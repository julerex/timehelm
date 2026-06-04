//! PostgreSQL connection and cell-grid simulation API.

use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgPoolOptions, PgPool, Row};
use std::time::Duration;

/// Create a PostgreSQL connection pool.
pub async fn create_pool(database_url: &str) -> anyhow::Result<PgPool> {
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .acquire_timeout(Duration::from_secs(5))
        .connect(database_url)
        .await?;
    Ok(pool)
}

/// Run SQLx migrations from `server/migrations/`.
pub async fn run_migrations(pool: &PgPool) -> anyhow::Result<()> {
    sqlx::migrate!().run(pool).await?;
    Ok(())
}

/// Advance simulation by one game second (PL/pgSQL).
pub async fn sim_tick(pool: &PgPool) -> anyhow::Result<i64> {
    let row: (i64,) = sqlx::query_as("SELECT sim_tick()").fetch_one(pool).await?;
    Ok(row.0)
}

/// Current game time in seconds (starts at 0 on deploy).
pub async fn get_game_time_seconds(pool: &PgPool) -> anyhow::Result<i64> {
    let row: (i64,) = sqlx::query_as("SELECT get_game_time_seconds()")
        .fetch_one(pool)
        .await?;
    Ok(row.0)
}

/// Move an entity to a neighboring cell; returns false if blocked.
pub async fn move_entity(
    pool: &PgPool,
    entity_id: i64,
    to_x: i32,
    to_y: i32,
    to_z: i32,
) -> anyhow::Result<bool> {
    let row: (bool,) = sqlx::query_as("SELECT move_entity($1, $2, $3, $4)")
        .bind(entity_id)
        .bind(to_x)
        .bind(to_y)
        .bind(to_z)
        .fetch_one(pool)
        .await?;
    Ok(row.0)
}

/// Create an entity on a cell; returns new entity id.
pub async fn create_entity(
    pool: &PgPool,
    entity_type: &str,
    x: i32,
    y: i32,
    z: i32,
) -> anyhow::Result<i64> {
    let row: (i64,) = sqlx::query_as(
        r#"
        INSERT INTO entity (x, y, z, entity_type)
        VALUES ($1, $2, $3, $4)
        RETURNING id
        "#,
    )
    .bind(x)
    .bind(y)
    .bind(z)
    .bind(entity_type)
    .fetch_one(pool)
    .await?;
    Ok(row.0)
}

pub async fn delete_entity(pool: &PgPool, entity_id: i64) -> anyhow::Result<()> {
    sqlx::query("DELETE FROM entity WHERE id = $1")
        .bind(entity_id)
        .execute(pool)
        .await?;
    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct GridEntity {
    pub id: i64,
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub entity_type: String,
}

pub async fn fetch_entities(pool: &PgPool) -> anyhow::Result<Vec<GridEntity>> {
    let rows = sqlx::query_as::<_, GridEntity>(
        r#"
        SELECT id, x, y, z, entity_type
        FROM entity
        ORDER BY id
        "#,
    )
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

#[derive(Debug, Clone, Serialize)]
pub struct CellApiRow {
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub bow_wall: String,
    pub stern_wall: String,
    pub port_wall: String,
    pub starboard_wall: String,
    pub floor: String,
    pub ceiling: String,
}

pub async fn fetch_all_cells(pool: &PgPool) -> anyhow::Result<Vec<CellApiRow>> {
    fetch_cells_query(pool, None).await
}

pub async fn fetch_cells_on_deck(pool: &PgPool, deck_z: i32) -> anyhow::Result<Vec<CellApiRow>> {
    fetch_cells_query(pool, Some(deck_z)).await
}

async fn fetch_cells_query(pool: &PgPool, deck_z: Option<i32>) -> anyhow::Result<Vec<CellApiRow>> {
    let rows = sqlx::query(
        r#"
        SELECT
            c.x,
            c.y,
            c.z,
            bw.name AS bow_wall,
            sw.name AS stern_wall,
            pw.name AS port_wall,
            stw.name AS starboard_wall,
            fl.name AS floor,
            ce.name AS ceiling
        FROM cell c
        JOIN wall_material bw ON c.bow_wall = bw.id
        JOIN wall_material sw ON c.stern_wall = sw.id
        JOIN wall_material pw ON c.port_wall = pw.id
        JOIN wall_material stw ON c.starboard_wall = stw.id
        JOIN floor_material fl ON c.floor = fl.id
        JOIN ceiling_material ce ON c.ceiling = ce.id
        WHERE ($1::INT IS NULL OR c.z = $1)
        ORDER BY c.z, c.x, c.y
        "#,
    )
    .bind(deck_z)
    .fetch_all(pool)
    .await?;

    let mut out = Vec::with_capacity(rows.len());
    for row in rows {
        out.push(CellApiRow {
            x: row.get("x"),
            y: row.get("y"),
            z: row.get("z"),
            bow_wall: row.get("bow_wall"),
            stern_wall: row.get("stern_wall"),
            port_wall: row.get("port_wall"),
            starboard_wall: row.get("starboard_wall"),
            floor: row.get("floor"),
            ceiling: row.get("ceiling"),
        });
    }
    Ok(out)
}

/// Find a walkable spawn cell on deck `z` (corridor with open-ish neighbors).
pub async fn find_spawn_cell(
    pool: &PgPool,
    deck_z: i32,
) -> anyhow::Result<Option<(i32, i32, i32)>> {
    let row = sqlx::query_as::<_, (i32, i32, i32)>(
        r#"
        SELECT c.x, c.y, c.z
        FROM cell c
        JOIN floor_material fl ON c.floor = fl.id
        WHERE c.z = $1 AND fl.name = 'carpet'
        ORDER BY c.x, c.y
        LIMIT 1
        "#,
    )
    .bind(deck_z)
    .fetch_optional(pool)
    .await?;
    Ok(row)
}
