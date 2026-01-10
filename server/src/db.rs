//! Database operations module.
//!
//! Handles PostgreSQL connection pooling and entity persistence.

use sqlx::{postgres::PgPoolOptions, PgPool};
use std::time::Duration;
use uuid::Uuid;

/// Create a PostgreSQL connection pool.
///
/// # Arguments
/// * `database_url` - PostgreSQL connection string (e.g., `postgresql://user:pass@host/db`)
///
/// # Returns
/// Connection pool with max 10 connections and 5-second acquire timeout
pub async fn create_pool(database_url: &str) -> anyhow::Result<PgPool> {
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .acquire_timeout(Duration::from_secs(5))
        .connect(database_url)
        .await?;

    Ok(pool)
}

/// Update the game time in minutes in the database.
///
/// Persists the current game time to the `game_state` table.
/// Game time is derived from Unix timestamp (1 real second = 1 game minute).
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `game_time_minutes` - Current game time in minutes
pub async fn set_game_time_minutes(pool: &PgPool, game_time_minutes: i64) -> anyhow::Result<()> {
    sqlx::query("UPDATE game_state SET game_time_minutes = $1 WHERE id = 1")
        .bind(game_time_minutes as i32)
        .execute(pool)
        .await?;

    Ok(())
}

/// Get entity type ID by name from the database.
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `name` - Entity type name (e.g., "human", "ball")
///
/// # Returns
/// Entity type ID from the `entity_types` table
pub async fn get_entity_type_id(pool: &PgPool, name: &str) -> anyhow::Result<i32> {
    let id: (i32,) = sqlx::query_as("SELECT id FROM entity_types WHERE name = $1")
        .bind(name)
        .fetch_one(pool)
        .await?;
    Ok(id.0)
}

/// Entity data structure for database operations.
///
/// Contains entity information in a format suitable for database storage.
/// Positions and rotations are stored as integers (centimeters).
pub struct EntityData {
    /// Entity identifier (can be UUID string or any string)
    pub id: String,
    /// Entity type name (e.g., "human", "ball")
    pub entity_type_name: String,
    /// X position in centimeters
    pub position_x: i32,
    /// Y position in centimeters
    pub position_y: i32,
    /// Z position in centimeters
    pub position_z: i32,
    /// X rotation in radians (stored as integer)
    pub rotation_x: i32,
    /// Y rotation in radians (stored as integer)
    pub rotation_y: i32,
    /// Z rotation in radians (stored as integer)
    pub rotation_z: i32,
}

/// Upsert (insert or update) an entity in the database.
///
/// If the entity ID already exists, the entity is updated.
/// If it doesn't exist, a new entity is inserted.
///
/// Entity IDs can be UUID strings or any string identifier.
/// Non-UUID strings are converted to deterministic UUID v5 for storage.
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `data` - Entity data to save
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

/// Save all entities to the database.
///
/// Converts game entities to database format and upserts them.
/// Called periodically (every 60 seconds) to persist game state.
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `entities` - Slice of game entities to save
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
