use axum::{
    extract::{State, WebSocketUpgrade},
    response::Response,
    routing::get,
    Router,
};
use sqlx::PgPool;
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_http::{cors::CorsLayer, services::ServeDir};

// mod auth;  // Commented out - users/sessions tables not in use
mod db;
mod game;
mod websocket;

use db::{create_pool, get_game_time_minutes, set_game_time_minutes};
use game::GameState;
use websocket::handle_websocket;

#[derive(Clone)]
pub struct AppState {
    pub game: Arc<RwLock<GameState>>,
    pub db: PgPool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();
    tracing_subscriber::fmt::init();

    // Connect to database
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = create_pool(&database_url).await?;
    tracing::info!("Connected to database");

    // Initialize game state with time from database
    let mut game_state = GameState::new();
    if let Ok(saved_time) = get_game_time_minutes(&pool).await {
        game_state.init_game_time(saved_time);
        tracing::info!("Loaded game time from database: {} minutes", saved_time);
    }
    let game_state = Arc::new(RwLock::new(game_state));

    let app_state = AppState {
        game: game_state.clone(),
        db: pool.clone(),
    };

    // Background task to persist game time every 30 seconds
    let persist_game_state = game_state.clone();
    let persist_pool = pool.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(30));
        loop {
            interval.tick().await;
            let game_time = persist_game_state.read().await.get_game_time_minutes();
            if let Err(e) = set_game_time_minutes(&persist_pool, game_time).await {
                tracing::error!("Failed to persist game time: {}", e);
            } else {
                tracing::debug!("Persisted game time: {} minutes", game_time);
            }
        }
    });

    let app = Router::new()
        .route("/ws", get(websocket_handler))
        // Auth routes commented out - users/sessions tables not in use
        // .route("/auth/twitter", get(auth::twitter_login))
        // .route("/auth/twitter/callback", get(auth::twitter_callback))
        // .route("/api/me", get(auth::get_current_user))
        .fallback_service(ServeDir::new("client/dist").append_index_html_on_directories(true))
        .layer(CorsLayer::permissive())
        .with_state(app_state);

    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse::<u16>()?;

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port)).await?;
    tracing::info!("Server listening on port {}", port);

    axum::serve(listener, app).await?;

    Ok(())
}

async fn websocket_handler(ws: WebSocketUpgrade, State(state): State<AppState>) -> Response {
    ws.on_upgrade(|socket| handle_websocket(socket, state))
}
