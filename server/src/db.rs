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

/// Get the current game time in minutes from the database
pub async fn get_game_time_minutes(pool: &PgPool) -> anyhow::Result<i64> {
    let row: (i32,) = sqlx::query_as("SELECT game_time_minutes FROM game_state WHERE id = 1")
        .fetch_one(pool)
        .await?;

    Ok(row.0 as i64)
}
