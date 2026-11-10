//! PostgreSQL pool, migrations, and the ship-world repository.

use anyhow::Context;
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::Value;
use sim::{
    Activity, CrewMember, Crop, Deck, Department, Door, Good, Job, PowerReport, RecipeId, Room,
    Skill, Station, Stock, World,
};
use sqlx::{postgres::PgPoolOptions, PgPool};
use std::time::Duration;

pub async fn create_pool(database_url: &str) -> anyhow::Result<PgPool> {
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .acquire_timeout(Duration::from_secs(5))
        .connect(database_url)
        .await?;
    Ok(pool)
}

pub async fn migrate(pool: &PgPool) -> anyhow::Result<()> {
    let migrator = sqlx::migrate!("./migrations");
    let expected: Vec<(i64, &[u8])> = migrator
        .migrations
        .iter()
        .map(|migration| (migration.version, migration.checksum.as_ref()))
        .collect();
    match classify_migration_history(read_migration_history(pool).await?.as_deref(), &expected) {
        MigrationHistory::Current => {
            tracing::info!("SQL migrations already applied");
            Ok(())
        }
        MigrationHistory::Diverged(detail) => {
            anyhow::bail!("SQL migration history does not match this server: {detail}")
        }
        MigrationHistory::Missing | MigrationHistory::Pending => {
            apply_migrations(&migrator, pool).await
        }
    }
}

async fn read_migration_history(pool: &PgPool) -> anyhow::Result<Option<Vec<MigrationRow>>> {
    match sqlx::query_as::<_, MigrationRow>(
        "SELECT version, checksum, success FROM _sqlx_migrations ORDER BY version",
    )
    .fetch_all(pool)
    .await
    {
        Ok(rows) => Ok(Some(rows)),
        Err(error) if is_postgres_code(&error, "42P01") => Ok(None),
        Err(error) => Err(error).context("read migration history"),
    }
}

async fn apply_migrations(migrator: &sqlx::migrate::Migrator, pool: &PgPool) -> anyhow::Result<()> {
    match migrator.run(pool).await {
        Ok(()) => Ok(()),
        Err(error) if is_postgres_code(&error, "42501") => {
            anyhow::bail!(
                "apply sql migrations: this database user cannot change the schema. Apply server/migrations as schema_admin from an operator machine, then restart"
            )
        }
        Err(error) => Err(error).context("apply sql migrations"),
    }
}

fn is_postgres_code(error: &(dyn std::error::Error + 'static), code: &str) -> bool {
    let mut current = Some(error);
    while let Some(err) = current {
        if let Some(sqlx_error) = err.downcast_ref::<sqlx::Error>() {
            if sqlx_error
                .as_database_error()
                .and_then(|db| db.code())
                .is_some_and(|db_code| db_code == code)
            {
                return true;
            }
        }
        current = err.source();
    }
    false
}

#[derive(Debug, PartialEq, Eq)]
enum MigrationHistory {
    Missing,
    Current,
    Pending,
    Diverged(String),
}

#[derive(sqlx::FromRow)]
struct MigrationRow {
    version: i64,
    checksum: Vec<u8>,
    success: bool,
}

fn classify_migration_history(
    recorded: Option<&[MigrationRow]>,
    expected: &[(i64, &[u8])],
) -> MigrationHistory {
    let Some(recorded) = recorded else {
        return MigrationHistory::Missing;
    };

    if let Some(failed) = recorded.iter().find(|row| !row.success) {
        return MigrationHistory::Diverged(format!(
            "migration {} is recorded as failed",
            failed.version
        ));
    }

    for row in recorded {
        match expected.iter().find(|(version, _)| *version == row.version) {
            Some((_, checksum)) if *checksum == row.checksum.as_slice() => {}
            Some(_) => {
                return MigrationHistory::Diverged(format!(
                    "migration {} checksum does not match this server",
                    row.version
                ));
            }
            None => {
                return MigrationHistory::Diverged(format!(
                    "migration {} is recorded but is not in this server",
                    row.version
                ));
            }
        }
    }

    if expected.iter().all(|(version, _)| {
        recorded
            .iter()
            .any(|row| row.version == *version && row.success)
    }) {
        MigrationHistory::Current
    } else {
        MigrationHistory::Pending
    }
}

/// Load the singleton world. `Ok(None)` means the schema is empty and should be seeded.
pub async fn load_world(pool: &PgPool) -> anyhow::Result<Option<World>> {
    let meta = sqlx::query_as::<_, MetaRow>(
        "SELECT seed, last_tick_unix, solar_kw, next_job_id, power_json FROM world_meta WHERE id = 1",
    )
    .fetch_optional(pool)
    .await?;
    let Some(meta) = meta else {
        return Ok(None);
    };

    let decks = sqlx::query_as::<_, DeckRow>("SELECT index, name FROM decks ORDER BY index")
        .fetch_all(pool)
        .await?;
    let rooms = sqlx::query_as::<_, RoomRow>(
        "SELECT id, deck, kind, name, polygon::text FROM rooms ORDER BY id",
    )
    .fetch_all(pool)
    .await?;
    let doors =
        sqlx::query_as::<_, DoorRow>("SELECT id, from_room, to_room FROM doors ORDER BY id")
            .fetch_all(pool)
            .await?;
    let stations =
        sqlx::query_as::<_, StationRow>("SELECT id, room_id, recipe FROM stations ORDER BY id")
            .fetch_all(pool)
            .await?;
    let crew = sqlx::query_as::<_, CrewRow>(
        "SELECT id, name, department, skill, room_id, x, y, deck, hunger, thirst, fatigue, health,
                activity, berth_id, station_id, watch_offset_min, companionship
         FROM crew ORDER BY id",
    )
    .fetch_all(pool)
    .await?;
    let stocks = sqlx::query_as::<_, StockRow>(
        "SELECT room_id, good, qty FROM stocks ORDER BY room_id, good",
    )
    .fetch_all(pool)
    .await?;
    let jobs = sqlx::query_as::<_, JobRow>(
        "SELECT id, station_id, recipe, elapsed_seconds FROM jobs ORDER BY id",
    )
    .fetch_all(pool)
    .await?;
    let crops = sqlx::query_as::<_, CropRow>(
        "SELECT room_id, progress_days, area_m2 FROM crops ORDER BY room_id",
    )
    .fetch_all(pool)
    .await?;

    decode_world(StoredWorld {
        meta,
        decks,
        rooms,
        doors,
        stations,
        crew,
        stocks,
        jobs,
        crops,
    })
    .map(Some)
}

pub async fn save_world(pool: &PgPool, world: &World) -> anyhow::Result<()> {
    let stored = encode_world(world);
    let mut tx = pool.begin().await?;
    sqlx::query("DELETE FROM jobs").execute(&mut *tx).await?;
    sqlx::query("DELETE FROM stocks").execute(&mut *tx).await?;
    sqlx::query("DELETE FROM crops").execute(&mut *tx).await?;
    sqlx::query("DELETE FROM crew").execute(&mut *tx).await?;
    sqlx::query("DELETE FROM stations")
        .execute(&mut *tx)
        .await?;
    sqlx::query("DELETE FROM doors").execute(&mut *tx).await?;
    sqlx::query("DELETE FROM rooms").execute(&mut *tx).await?;
    sqlx::query("DELETE FROM decks").execute(&mut *tx).await?;

    for deck in &stored.decks {
        sqlx::query("INSERT INTO decks (index, name) VALUES ($1, $2)")
            .bind(deck.index)
            .bind(&deck.name)
            .execute(&mut *tx)
            .await?;
    }
    for room in &stored.rooms {
        sqlx::query(
            "INSERT INTO rooms (id, deck, kind, name, polygon) VALUES ($1, $2, $3, $4, $5::jsonb)",
        )
        .bind(room.id)
        .bind(room.deck)
        .bind(&room.kind)
        .bind(&room.name)
        .bind(&room.polygon)
        .execute(&mut *tx)
        .await?;
    }
    for door in &stored.doors {
        sqlx::query("INSERT INTO doors (id, from_room, to_room) VALUES ($1, $2, $3)")
            .bind(door.id)
            .bind(door.from_room)
            .bind(door.to_room)
            .execute(&mut *tx)
            .await?;
    }
    for station in &stored.stations {
        sqlx::query("INSERT INTO stations (id, room_id, recipe) VALUES ($1, $2, $3)")
            .bind(station.id)
            .bind(station.room_id)
            .bind(&station.recipe)
            .execute(&mut *tx)
            .await?;
    }
    for member in &stored.crew {
        sqlx::query(
            "INSERT INTO crew (
                id, name, department, skill, room_id, x, y, deck, hunger, thirst, fatigue, health,
                activity, berth_id, station_id, watch_offset_min, companionship
             ) VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17
             )",
        )
        .bind(member.id)
        .bind(&member.name)
        .bind(&member.department)
        .bind(&member.skill)
        .bind(member.room_id)
        .bind(member.x)
        .bind(member.y)
        .bind(member.deck)
        .bind(member.hunger)
        .bind(member.thirst)
        .bind(member.fatigue)
        .bind(member.health)
        .bind(&member.activity)
        .bind(member.berth_id)
        .bind(member.station_id)
        .bind(member.watch_offset_min)
        .bind(member.companionship)
        .execute(&mut *tx)
        .await?;
    }
    for stock in &stored.stocks {
        sqlx::query("INSERT INTO stocks (room_id, good, qty) VALUES ($1, $2, $3)")
            .bind(stock.room_id)
            .bind(&stock.good)
            .bind(stock.qty)
            .execute(&mut *tx)
            .await?;
    }
    for job in &stored.jobs {
        sqlx::query(
            "INSERT INTO jobs (id, station_id, recipe, elapsed_seconds) VALUES ($1, $2, $3, $4)",
        )
        .bind(job.id)
        .bind(job.station_id)
        .bind(&job.recipe)
        .bind(job.elapsed_seconds)
        .execute(&mut *tx)
        .await?;
    }
    for crop in &stored.crops {
        sqlx::query("INSERT INTO crops (room_id, progress_days, area_m2) VALUES ($1, $2, $3)")
            .bind(crop.room_id)
            .bind(crop.progress_days)
            .bind(crop.area_m2)
            .execute(&mut *tx)
            .await?;
    }
    sqlx::query(
        "INSERT INTO world_meta (id, seed, last_tick_unix, solar_kw, next_job_id, power_json)
         VALUES (1, $1, $2, $3, $4, $5)
         ON CONFLICT (id) DO UPDATE SET
            seed = EXCLUDED.seed,
            last_tick_unix = EXCLUDED.last_tick_unix,
            solar_kw = EXCLUDED.solar_kw,
            next_job_id = EXCLUDED.next_job_id,
            power_json = EXCLUDED.power_json,
            updated_at = NOW()",
    )
    .bind(stored.meta.seed)
    .bind(stored.meta.last_tick_unix)
    .bind(stored.meta.solar_kw)
    .bind(stored.meta.next_job_id)
    .bind(&stored.meta.power_json)
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(())
}

struct StoredWorld {
    meta: MetaRow,
    decks: Vec<DeckRow>,
    rooms: Vec<RoomRow>,
    doors: Vec<DoorRow>,
    stations: Vec<StationRow>,
    crew: Vec<CrewRow>,
    stocks: Vec<StockRow>,
    jobs: Vec<JobRow>,
    crops: Vec<CropRow>,
}

#[derive(sqlx::FromRow)]
struct MetaRow {
    seed: i64,
    last_tick_unix: i64,
    solar_kw: f64,
    next_job_id: i32,
    power_json: String,
}

#[derive(sqlx::FromRow)]
struct DeckRow {
    index: i16,
    name: String,
}

#[derive(sqlx::FromRow)]
struct RoomRow {
    id: i32,
    deck: i16,
    kind: String,
    name: String,
    polygon: String,
}

#[derive(sqlx::FromRow)]
struct DoorRow {
    id: i32,
    from_room: i32,
    to_room: i32,
}

#[derive(sqlx::FromRow)]
struct StationRow {
    id: i32,
    room_id: i32,
    recipe: Option<String>,
}

#[derive(sqlx::FromRow)]
struct CrewRow {
    id: i32,
    name: String,
    department: String,
    skill: String,
    room_id: i32,
    x: f64,
    y: f64,
    deck: i16,
    hunger: f64,
    thirst: f64,
    fatigue: f64,
    health: f64,
    activity: String,
    berth_id: i32,
    station_id: Option<i32>,
    watch_offset_min: i32,
    companionship: f64,
}

#[derive(sqlx::FromRow)]
struct StockRow {
    room_id: i32,
    good: String,
    qty: f64,
}

#[derive(sqlx::FromRow)]
struct JobRow {
    id: i32,
    station_id: i32,
    recipe: String,
    elapsed_seconds: f64,
}

#[derive(sqlx::FromRow)]
struct CropRow {
    room_id: i32,
    progress_days: f64,
    area_m2: f64,
}

fn encode_world(world: &World) -> StoredWorld {
    StoredWorld {
        meta: MetaRow {
            seed: i64::try_from(world.seed).unwrap_or(i64::MAX),
            last_tick_unix: world.last_tick_unix,
            solar_kw: world.solar_kw,
            next_job_id: i32::try_from(world.next_job_id).unwrap_or(i32::MAX),
            power_json: serde_json::to_string(&world.power).expect("power report"),
        },
        decks: world
            .decks
            .iter()
            .map(|deck| DeckRow {
                index: i16::from(deck.index),
                name: deck.name.clone(),
            })
            .collect(),
        rooms: world
            .rooms
            .iter()
            .map(|room| RoomRow {
                id: room.id as i32,
                deck: i16::from(room.deck),
                kind: enum_str(&room.kind),
                name: room.name.clone(),
                polygon: serde_json::to_string(&room.polygon).expect("polygon"),
            })
            .collect(),
        doors: world
            .doors
            .iter()
            .map(|door| DoorRow {
                id: door.id as i32,
                from_room: door.from_room as i32,
                to_room: door.to_room as i32,
            })
            .collect(),
        stations: world
            .stations
            .iter()
            .map(|station| StationRow {
                id: station.id as i32,
                room_id: station.room_id as i32,
                recipe: station.recipe.map(|recipe| enum_str(&recipe)),
            })
            .collect(),
        crew: world
            .crew
            .iter()
            .map(|crew| CrewRow {
                id: crew.id as i32,
                name: crew.name.clone(),
                department: enum_str(&crew.department),
                skill: enum_str(&crew.skill),
                room_id: crew.room_id as i32,
                x: f64::from(crew.x),
                y: f64::from(crew.y),
                deck: i16::from(crew.deck),
                hunger: crew.hunger,
                thirst: crew.thirst,
                fatigue: crew.fatigue,
                health: crew.health,
                activity: enum_str(&crew.activity),
                berth_id: crew.berth_id as i32,
                station_id: crew.station_id.map(|id| id as i32),
                watch_offset_min: crew.watch_offset_min as i32,
                companionship: crew.companionship,
            })
            .collect(),
        stocks: world
            .stocks
            .iter()
            .map(|stock| StockRow {
                room_id: stock.room_id as i32,
                good: enum_str(&stock.good),
                qty: stock.qty,
            })
            .collect(),
        jobs: world
            .jobs
            .iter()
            .map(|job| JobRow {
                id: job.id as i32,
                station_id: job.station_id as i32,
                recipe: enum_str(&job.recipe),
                elapsed_seconds: job.elapsed_seconds,
            })
            .collect(),
        crops: world
            .crops
            .iter()
            .map(|crop| CropRow {
                room_id: crop.room_id as i32,
                progress_days: crop.progress_days,
                area_m2: crop.area_m2,
            })
            .collect(),
    }
}

fn decode_world(stored: StoredWorld) -> anyhow::Result<World> {
    let power: PowerReport = serde_json::from_str(&stored.meta.power_json)?;
    Ok(World {
        seed: u64::try_from(stored.meta.seed).unwrap_or(0),
        last_tick_unix: stored.meta.last_tick_unix,
        solar_kw: stored.meta.solar_kw,
        next_job_id: u32::try_from(stored.meta.next_job_id).unwrap_or(0),
        power,
        decks: stored
            .decks
            .into_iter()
            .map(|deck| Deck {
                index: deck.index as u8,
                name: deck.name,
            })
            .collect(),
        rooms: stored
            .rooms
            .into_iter()
            .map(|room| {
                Ok(Room {
                    id: room.id as u32,
                    deck: room.deck as u8,
                    kind: enum_from(&room.kind)?,
                    name: room.name,
                    polygon: serde_json::from_str(&room.polygon)?,
                })
            })
            .collect::<anyhow::Result<Vec<_>>>()?,
        doors: stored
            .doors
            .into_iter()
            .map(|door| Door {
                id: door.id as u32,
                from_room: door.from_room as u32,
                to_room: door.to_room as u32,
            })
            .collect(),
        stations: stored
            .stations
            .into_iter()
            .map(|station| {
                Ok(Station {
                    id: station.id as u32,
                    room_id: station.room_id as u32,
                    recipe: station
                        .recipe
                        .as_deref()
                        .map(enum_from::<RecipeId>)
                        .transpose()?,
                })
            })
            .collect::<anyhow::Result<Vec<_>>>()?,
        crew: stored
            .crew
            .into_iter()
            .map(|crew| {
                Ok(CrewMember {
                    id: crew.id as u32,
                    name: crew.name,
                    department: enum_from::<Department>(&crew.department)?,
                    skill: enum_from::<Skill>(&crew.skill)?,
                    room_id: crew.room_id as u32,
                    x: crew.x as f32,
                    y: crew.y as f32,
                    deck: crew.deck as u8,
                    hunger: crew.hunger,
                    thirst: crew.thirst,
                    fatigue: crew.fatigue,
                    health: crew.health,
                    activity: enum_from::<Activity>(&crew.activity)?,
                    berth_id: crew.berth_id as u32,
                    station_id: crew.station_id.map(|id| id as u32),
                    watch_offset_min: crew.watch_offset_min as u32,
                    companionship: crew.companionship,
                })
            })
            .collect::<anyhow::Result<Vec<_>>>()?,
        stocks: stored
            .stocks
            .into_iter()
            .map(|stock| {
                Ok(Stock {
                    room_id: stock.room_id as u32,
                    good: enum_from::<Good>(&stock.good)?,
                    qty: stock.qty,
                })
            })
            .collect::<anyhow::Result<Vec<_>>>()?,
        jobs: stored
            .jobs
            .into_iter()
            .map(|job| {
                Ok(Job {
                    id: job.id as u32,
                    station_id: job.station_id as u32,
                    recipe: enum_from::<RecipeId>(&job.recipe)?,
                    elapsed_seconds: job.elapsed_seconds,
                })
            })
            .collect::<anyhow::Result<Vec<_>>>()?,
        crops: stored
            .crops
            .into_iter()
            .map(|crop| Crop {
                room_id: crop.room_id as u32,
                progress_days: crop.progress_days,
                area_m2: crop.area_m2,
            })
            .collect(),
    })
}

fn enum_str<T: Serialize>(value: &T) -> String {
    match serde_json::to_value(value).expect("enum") {
        Value::String(text) => text,
        other => other.to_string(),
    }
}

fn enum_from<T: DeserializeOwned>(text: &str) -> anyhow::Result<T> {
    serde_json::from_value(Value::String(text.to_string())).context("parse enum")
}

/// Open a saved world or seed one, then apply elapsed real time.
pub async fn bootstrap_world(pool: &PgPool) -> anyhow::Result<World> {
    migrate(pool).await?;
    let now = chrono::Utc::now().timestamp();
    let mut world = if let Some(world) = load_world(pool).await? {
        if saved_layout_matches(&world) {
            world
        } else {
            tracing::warn!(
                "Saved rooms do not match the Icon deck plan; generating a new ship from the stored seed"
            );
            let tick = world.last_tick_unix;
            let aligned = tick - tick.rem_euclid(60);
            World::generate(world.seed, aligned).map_err(|error| anyhow::anyhow!(error))?
        }
    } else {
        let aligned = now - now.rem_euclid(60);
        World::generate(WORLD_SEED, aligned).map_err(|error| anyhow::anyhow!(error))?
    };
    let from = world.last_tick_unix;
    world.catch_up_to(now);
    if world.last_tick_unix != from {
        tracing::info!(
            "Caught the ship up from unix {from} to {}",
            world.last_tick_unix
        );
    }
    save_world(pool, &world).await?;
    Ok(world)
}

/// Stable seed for the first persistent world. Layout does not roll this.
pub const WORLD_SEED: u64 = 0x7469_6D65_6865_6C6D;

/// The saved ship is the one this build generates. A renamed deck or a missing
/// Icon neighborhood means the stored rectangles are from an older plan.
fn saved_layout_matches(world: &World) -> bool {
    world.decks.len() == sim::DECK_COUNT
        && world
            .decks
            .iter()
            .zip(sim::DECK_NAMES)
            .all(|(deck, name)| deck.name == name)
        && world
            .rooms
            .iter()
            .any(|room| room.deck == 8 && room.name == "Central Park")
        && world
            .rooms
            .iter()
            .any(|room| room.deck == 0 && room.name == "Gas generator hall")
        && world
            .rooms
            .iter()
            .any(|room| room.deck == 1 && room.name == "Wastewater and incinerator")
        && world
            .rooms
            .iter()
            .any(|room| room.deck == 4 && room.name == "Casino Royale")
        && world.rooms.iter().any(|room| {
            room.deck == 9
                && room.kind == sim::RoomKind::Head
                && room
                    .polygon
                    .iter()
                    .any(|point| (point[0] - (40.0 + 5.6)).abs() < 0.2)
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use sim::MemoryStore;

    fn hex_encode(bytes: &[u8]) -> String {
        const HEX: &[u8; 16] = b"0123456789abcdef";
        let mut out = String::with_capacity(bytes.len() * 2);
        for byte in bytes {
            out.push(HEX[(byte >> 4) as usize] as char);
            out.push(HEX[(byte & 0x0f) as usize] as char);
        }
        out
    }

    #[test]
    fn a_saved_world_must_match_the_icon_deck_plan() {
        let world = World::generate(1, 0).expect("world");
        assert!(saved_layout_matches(&world));
        let mut stale = world;
        stale.decks[8].name = "Carpentry Deck".to_string();
        assert!(!saved_layout_matches(&stale));
    }

    #[test]
    fn embedded_migration_checksum_matches_the_sql_file() {
        let migrator = sqlx::migrate!("./migrations");
        let migration = migrator
            .migrations
            .iter()
            .find(|migration| migration.version == 1)
            .expect("version 1");
        assert_eq!(migration.description.as_ref(), "sim world");
        assert_eq!(
            hex_encode(migration.checksum.as_ref()),
            "aef2ddacb6e8bc1dd89506a3a910cdfe592b652f4a202ebc83ad40a85a33fb805da412a61799e6bc4842b74d1115ea54"
        );
        let companionship = migrator
            .migrations
            .iter()
            .find(|migration| migration.version == 2)
            .expect("version 2");
        assert_eq!(companionship.description.as_ref(), "companionship");
        assert_eq!(
            hex_encode(companionship.checksum.as_ref()),
            "4985ae9c4847fe7d1361e1dce7efbe680038ab7d7250d2dfc488d90b5ecbb154ca2ea4eb69edfe5d2a386d5c523a2e46"
        );
    }

    #[test]
    fn migration_history_classifies_recorded_rows() {
        let checksum = b"abc";
        let expected = [(1_i64, checksum.as_slice())];
        let applied = [MigrationRow {
            version: 1,
            checksum: checksum.to_vec(),
            success: true,
        }];
        assert_eq!(
            classify_migration_history(None, &expected),
            MigrationHistory::Missing
        );
        assert_eq!(
            classify_migration_history(Some(&[]), &expected),
            MigrationHistory::Pending
        );
        assert_eq!(
            classify_migration_history(Some(&applied), &expected),
            MigrationHistory::Current
        );
        assert_eq!(
            classify_migration_history(
                Some(&[MigrationRow {
                    version: 1,
                    checksum: b"nope".to_vec(),
                    success: true,
                }]),
                &expected
            ),
            MigrationHistory::Diverged(
                "migration 1 checksum does not match this server".to_string()
            )
        );
        assert_eq!(
            classify_migration_history(
                Some(&[MigrationRow {
                    version: 1,
                    checksum: checksum.to_vec(),
                    success: false,
                }]),
                &expected
            ),
            MigrationHistory::Diverged("migration 1 is recorded as failed".to_string())
        );
        assert_eq!(
            classify_migration_history(
                Some(&[MigrationRow {
                    version: 9,
                    checksum: checksum.to_vec(),
                    success: true,
                }]),
                &expected
            ),
            MigrationHistory::Diverged(
                "migration 9 is recorded but is not in this server".to_string()
            )
        );
    }

    #[test]
    fn encoded_rows_round_trip_and_memory_store() {
        let mut world = World::generate(WORLD_SEED, 1_700_000_000).expect("world");
        world.step();
        let loaded = decode_world(encode_world(&world)).expect("decode");
        assert_eq!(loaded, world);

        let mut memory = MemoryStore::default();
        memory.save(&loaded);
        assert_eq!(memory.load().as_ref(), Some(&loaded));
    }
}
