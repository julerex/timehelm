//! Time Helm Server — PostgreSQL cell-grid simulation + WebSocket/HTTP API.

use axum::extract::WebSocketUpgrade;
use axum::{
    extract::State,
    response::{Redirect, Response},
    routing::get,
    Router,
};
use sqlx::PgPool;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::{broadcast, RwLock};
use tower_http::{cors::CorsLayer, services::ServeDir};

mod api;
mod db;
mod game;
mod messages;
mod websocket;

use db::{create_pool, fetch_entities, get_game_time_seconds, run_migrations, sim_tick};
use game::GameState;
use messages::GameMessage;
use websocket::{build_world_state, handle_websocket};

#[derive(Clone)]
pub struct AppState {
    pub game: Arc<RwLock<GameState>>,
    pub db: Option<PgPool>,
    pub broadcast_tx: broadcast::Sender<String>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();
    tracing_subscriber::fmt::init();

    let pool = match std::env::var("DATABASE_URL") {
        Ok(database_url) => {
            let pool = create_pool(&database_url).await?;
            run_migrations(&pool).await?;
            tracing::info!("Connected to database and ran migrations");
            Some(pool)
        }
        Err(_) => {
            tracing::warn!("DATABASE_URL is not set; API and simulation require a database");
            None
        }
    };

    let game_state = Arc::new(RwLock::new(GameState::new()));
    let (broadcast_tx, _) = broadcast::channel::<String>(100);

    let app_state = AppState {
        game: game_state,
        db: pool.clone(),
        broadcast_tx: broadcast_tx.clone(),
    };

    if let Some(pool) = pool.clone() {
        let game_for_tick = app_state.game.clone();
        let broadcast_for_tick = broadcast_tx.clone();
        tokio::spawn(async move {
            loop {
                let tick_start = Instant::now();
                match sim_tick(&pool).await {
                    Ok(t) => tracing::debug!("sim_tick game_time_seconds={t}"),
                    Err(e) => tracing::error!("sim_tick failed: {e}"),
                }
                if let Ok(world) = build_world_state(&pool, &game_for_tick).await {
                    if let Ok(json) = serde_json::to_string(&world) {
                        let _ = broadcast_for_tick.send(json);
                    }
                }
                let elapsed = tick_start.elapsed();
                if elapsed < tokio::time::Duration::from_secs(1) {
                    tokio::time::sleep(tokio::time::Duration::from_secs(1) - elapsed).await;
                } else {
                    tracing::warn!(
                        "sim_tick overran by {:?}",
                        elapsed.saturating_sub(tokio::time::Duration::from_secs(1))
                    );
                }
            }
        });
    }

    let game_for_broadcast = app_state.game.clone();
    let broadcast_tx_for_task = broadcast_tx.clone();
    let pool_for_broadcast = pool.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(100));
        loop {
            interval.tick().await;
            let Some(pool) = &pool_for_broadcast else {
                continue;
            };
            let entities = match fetch_entities(pool).await {
                Ok(e) => e,
                Err(e) => {
                    tracing::error!("fetch_entities: {e}");
                    continue;
                }
            };
            let game_time_seconds = get_game_time_seconds(pool).await.unwrap_or(0);
            let players = game_for_broadcast.read().await.get_all_players();
            let world_state = GameMessage::WorldState {
                entities,
                players,
                game_time_seconds,
            };
            if let Ok(world_json) = serde_json::to_string(&world_state) {
                let _ = broadcast_tx_for_task.send(world_json);
            }
        }
    });

    let app = Router::new()
        .route("/", get(|| async { Redirect::temporary("/3d/") }))
        .route(
            "/seacells",
            get(|| async { Redirect::permanent("/seacells/") }),
        )
        .route("/2d", get(|| async { Redirect::permanent("/seacells/") }))
        .route("/3d", get(|| async { Redirect::permanent("/3d/") }))
        .route("/ws", get(websocket_handler))
        .route("/api/cells", get(api::all_cells))
        .route("/api/decks/:z/cells", get(api::deck_cells))
        .route("/api/entities", get(api::entities))
        .route(
            "/favicon.ico",
            get(|| async { Redirect::temporary("/favicon.svg") }),
        )
        .fallback_service(ServeDir::new(static_dir()).append_index_html_on_directories(true))
        .layer(CorsLayer::permissive())
        .with_state(app_state);

    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse::<u16>()?;

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{port}")).await?;
    tracing::info!("Server listening on 0.0.0.0:{port} — open http://localhost:{port}/");

    axum::serve(listener, app).await?;

    Ok(())
}

fn static_dir() -> std::path::PathBuf {
    let candidates = ["client/public", "../client/public"];
    for path in candidates {
        let p = std::path::Path::new(path);
        if p.join("index.html").exists() {
            return p.to_path_buf();
        }
    }
    candidates[0].into()
}

async fn websocket_handler(ws: WebSocketUpgrade, State(state): State<AppState>) -> Response {
    ws.on_upgrade(|socket| handle_websocket(socket, state))
}
