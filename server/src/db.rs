use sqlx::{postgres::PgPoolOptions, PgPool};
use std::time::Duration;
use uuid::Uuid;

pub async fn create_pool(database_url: &str) -> anyhow::Result<PgPool> {
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .acquire_timeout(Duration::from_secs(5))
        .connect(database_url)
        .await?;

    Ok(pool)
}

/// Update the game time in minutes in the database
pub async fn set_game_time_minutes(pool: &PgPool, game_time_minutes: i64) -> anyhow::Result<()> {
    sqlx::query("UPDATE game_state SET game_time_minutes = $1 WHERE id = 1")
        .bind(game_time_minutes as i32)
        .execute(pool)
        .await?;

    Ok(())
}

/// Get entity type ID by name
pub async fn get_entity_type_id(pool: &PgPool, name: &str) -> anyhow::Result<i32> {
    let id: (i32,) = sqlx::query_as("SELECT id FROM entity_types WHERE name = $1")
        .bind(name)
        .fetch_one(pool)
        .await?;
    Ok(id.0)
}

/// Entity data for database operations
pub struct EntityData {
    pub id: String,
    pub entity_type_name: String,
    pub position_x: i32,
    pub position_y: i32,
    pub position_z: i32,
    pub rotation_x: i32,
    pub rotation_y: i32,
    pub rotation_z: i32,
}

/// Upsert entity in database
/// entity_id can be a UUID string or any string identifier
pub async fn upsert_entity(pool: &PgPool, data: &EntityData) -> anyhow::Result<()> {
    let type_id = get_entity_type_id(pool, &data.entity_type_name).await?;
    // Try to parse as UUID, if it fails, generate a deterministic UUID v5 from the string
    let uuid_id = Uuid::parse_str(&data.id).unwrap_or_else(|_| {
        // Use a fixed namespace UUID for entity IDs
        let namespace = Uuid::parse_str("6ba7b810-9dad-11d1-80b4-00c04fd430c8").unwrap();
        Uuid::new_v5(&namespace, data.id.as_bytes())
    });

    sqlx::query(
        r#"
        INSERT INTO entities (id, entity_type_id, position_x, position_y, position_z, rotation_x, rotation_y, rotation_z)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        ON CONFLICT (id) DO UPDATE SET
            entity_type_id = EXCLUDED.entity_type_id,
            position_x = EXCLUDED.position_x,
            position_y = EXCLUDED.position_y,
            position_z = EXCLUDED.position_z,
            rotation_x = EXCLUDED.rotation_x,
            rotation_y = EXCLUDED.rotation_y,
            rotation_z = EXCLUDED.rotation_z,
            updated_at = NOW()
        "#,
    )
    .bind(uuid_id)
    .bind(type_id)
    .bind(data.position_x)
    .bind(data.position_y)
    .bind(data.position_z)
    .bind(data.rotation_x)
    .bind(data.rotation_y)
    .bind(data.rotation_z)
    .execute(pool)
    .await?;

    Ok(())
}

/// Save all entities to database
pub async fn save_all_entities(
    pool: &PgPool,
    entities: &[crate::game::Entity],
) -> anyhow::Result<()> {
    for entity in entities {
        let data = EntityData {
            id: entity.id.clone(),
            entity_type_name: entity.entity_type.as_str().to_string(),
            position_x: entity.position.x as i32,
            position_y: entity.position.y as i32,
            position_z: entity.position.z as i32,
            rotation_x: entity.rotation.x as i32,
            rotation_y: entity.rotation.y as i32,
            rotation_z: entity.rotation.z as i32,
        };
        upsert_entity(pool, &data).await?;
    }
    Ok(())
}
