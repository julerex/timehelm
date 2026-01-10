//! Time Helm Server
//!
//! Main entry point for the Time Helm game server.
//! Handles WebSocket connections, game state management, physics simulation,
//! and periodic persistence of game data to PostgreSQL.

use axum::{
    extract::{State, WebSocketUpgrade},
    response::Response,
    routing::get,
    Router,
};
use sqlx::PgPool;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use tower_http::{cors::CorsLayer, services::ServeDir};

// mod auth;  // Commented out - users/sessions tables not in use
mod db;
mod game;
mod messages;
mod physics;
mod websocket;

use db::{create_pool, save_all_entities, set_game_time_minutes};
use game::GameState;
use messages::GameMessage;
use websocket::handle_websocket;

/// Application state shared across all request handlers.
///
/// Contains:
/// - `game`: Thread-safe game state (players, entities, physics)
/// - `db`: PostgreSQL connection pool
/// - `broadcast_tx`: Channel for broadcasting world state updates to all connected clients
#[derive(Clone)]
pub struct AppState {
    /// Thread-safe game state containing players, entities, and physics simulation
    pub game: Arc<RwLock<GameState>>,
    /// PostgreSQL database connection pool
    pub db: PgPool,
    /// Broadcast channel sender for distributing world state updates to WebSocket clients
    pub broadcast_tx: broadcast::Sender<String>,
}

/// Main entry point for the Time Helm server.
///
/// Initializes:
/// 1. Database connection pool
/// 2. Game state (thread-safe)
/// 3. Background tasks for:
///    - Game time persistence (every 60 seconds)
///    - Entity persistence (every 60 seconds)
///    - Physics simulation (60 FPS)
///    - World state broadcasting (10 FPS)
/// 4. HTTP/WebSocket server
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load environment variables from .env file
    dotenv::dotenv().ok();
    // Initialize tracing for structured logging
    tracing_subscriber::fmt::init();

    // Connect to PostgreSQL database
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = create_pool(&database_url).await?;
    tracing::info!("Connected to database");

    // Initialize game state with thread-safe access
    let game_state = Arc::new(RwLock::new(GameState::new()));
    // Create broadcast channel for sending world state updates to all WebSocket clients
    // Channel capacity: 100 messages
    let (broadcast_tx, _) = broadcast::channel::<String>(100);

    let app_state = AppState {
        game: game_state,
        db: pool.clone(),
        broadcast_tx: broadcast_tx.clone(),
    };

    // Background task: Persist game time to database every real-world minute
    // Game time is derived from Unix timestamp (1 real second = 1 game minute)
    let persist_pool = pool.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(60));
        loop {
            interval.tick().await;
            let game_time = GameState::get_game_time_minutes();
            if let Err(e) = set_game_time_minutes(&persist_pool, game_time).await {
                tracing::error!("Failed to persist game time: {e}");
            } else {
                tracing::debug!("Persisted game time: {game_time} minutes");
            }
        }
    });

    // Background task: Persist all entities to database every real-world minute
    // This ensures entity positions and states are saved periodically
    let persist_pool_entities = pool.clone();
    let game_state_for_entities = app_state.game.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(60));
        loop {
            interval.tick().await;
            // Read lock to get all entities, then drop lock before database write
            let game = game_state_for_entities.read().await;
            let entities: Vec<_> = game.get_all_entities();
            drop(game);

            if let Err(e) = save_all_entities(&persist_pool_entities, &entities).await {
                tracing::error!("Failed to persist entities: {e}");
            } else {
                tracing::debug!("Persisted {} entities", entities.len());
            }
        }
    });

    // Background task: Physics simulation update loop running at 60 FPS
    // Updates physics world and syncs entity positions from physics simulation
    let game_state_for_physics = app_state.game.clone();
    tokio::spawn(async move {
        // 16,666,667 nanoseconds = ~16.67ms = ~60 FPS
        let mut interval = tokio::time::interval(tokio::time::Duration::from_nanos(16_666_667));
        loop {
            interval.tick().await;
            let mut game = game_state_for_physics.write().await;
            // Step physics with delta time of 1/60 second
            game.step_physics(1.0 / 60.0);
        }
    });

    // Background task: Broadcast world state updates to all connected clients
    // Runs at 10 FPS (every 100ms) for network efficiency
    // Sends complete world state (all players + all entities) to all WebSocket clients
    let game_state_for_broadcast = app_state.game.clone();
    let broadcast_tx_for_task = broadcast_tx.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(100)); // 10 FPS
        loop {
            interval.tick().await;
            // Read lock to get world state, then drop lock before serialization
            let game = game_state_for_broadcast.read().await;
            let all_players = game.get_all_players();
            let all_entities = game.get_all_entities();
            drop(game);

            let world_state = GameMessage::WorldState {
                players: all_players,
                entities: all_entities,
            };
            // Serialize and broadcast to all WebSocket clients
            if let Ok(world_json) = serde_json::to_string(&world_state) {
                let _ = broadcast_tx_for_task.send(world_json);
            }
        }
    });

    // Set up HTTP routes
    let app = Router::new()
        // WebSocket endpoint for game client connections
        .route("/ws", get(websocket_handler))
        // Auth routes commented out - users/sessions tables not in use
        // .route("/auth/twitter", get(auth::twitter_login))
        // .route("/auth/twitter/callback", get(auth::twitter_callback))
        // .route("/api/me", get(auth::get_current_user))
        // Serve static files from client/dist (fallback for all non-API routes)
        .fallback_service(ServeDir::new("client/dist").append_index_html_on_directories(true))
        // Enable CORS for all origins (development)
        .layer(CorsLayer::permissive())
        .with_state(app_state);

    // Get port from environment variable or default to 8080
    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse::<u16>()?;

    // Bind to all network interfaces on the specified port
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port)).await?;
    tracing::info!("Server listening on port {}", port);

    // Start the HTTP server
    axum::serve(listener, app).await?;

    Ok(())
}

/// WebSocket connection handler.
///
/// Upgrades HTTP connection to WebSocket and delegates to `handle_websocket`
/// for message processing and game state synchronization.
async fn websocket_handler(ws: WebSocketUpgrade, State(state): State<AppState>) -> Response {
    ws.on_upgrade(|socket| handle_websocket(socket, state))
}
