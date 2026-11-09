//! Time Helm server.
//!
//! Serves the 3D viewer and owns the persistent ship simulation. The simulation
//! advances in real time and is stored in PostgreSQL. Viewers cannot change it.

use axum::extract::{State, WebSocketUpgrade};
use axum::response::{IntoResponse, Redirect, Response};
use axum::routing::get;
use axum::Json;
use axum::Router;
use sqlx::PgPool;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;

mod db;
mod websocket;

use db::{bootstrap_world, create_pool, save_world};
use sim::ServerMessage;
use websocket::handle_websocket;

/// In-memory copy of the database world. `world` is absent when persistence is unavailable.
pub struct SimState {
    pub world: Option<sim::World>,
    pub unavailable_reason: String,
}

#[derive(Clone)]
pub struct AppState {
    pub sim: Arc<RwLock<SimState>>,
    pub db: Option<PgPool>,
    pub broadcast_tx: broadcast::Sender<String>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();
    tracing_subscriber::fmt::init();

    let (broadcast_tx, _) = broadcast::channel::<String>(32);
    let (pool, setup_reason) = match std::env::var("DATABASE_URL") {
        Ok(database_url) => match create_pool(&database_url).await {
            Ok(pool) => {
                tracing::info!("Connected to database");
                (Some(pool), String::new())
            }
            Err(error) => {
                let reason = format!("Database connection failed: {error}");
                tracing::error!("{reason}");
                (None, reason)
            }
        },
        Err(_) => {
            let reason = "DATABASE_URL is not set; the persistent world is unavailable".to_string();
            tracing::error!("{reason}");
            (None, reason)
        }
    };

    let sim_state = match &pool {
        Some(pool) => match bootstrap_world(pool).await {
            Ok(world) => {
                tracing::info!(
                    "Ship world ready at unix {} with {} crew",
                    world.last_tick_unix,
                    world.crew.len()
                );
                SimState {
                    world: Some(world),
                    unavailable_reason: String::new(),
                }
            }
            Err(error) => {
                let reason = format!("Could not load the persistent world: {error}");
                tracing::error!("{reason}");
                SimState {
                    world: None,
                    unavailable_reason: reason,
                }
            }
        },
        None => SimState {
            world: None,
            unavailable_reason: setup_reason,
        },
    };

    let app_state = AppState {
        sim: Arc::new(RwLock::new(sim_state)),
        db: pool,
        broadcast_tx: broadcast_tx.clone(),
    };

    if app_state.db.is_some() && app_state.sim.read().await.world.is_some() {
        let tick_state = app_state.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(1));
            loop {
                interval.tick().await;
                if let Err(error) = tick_once(&tick_state).await {
                    tracing::error!("Simulation tick failed: {error}");
                }
            }
        });
    }

    let app = Router::new()
        .route("/", get(|| async { Redirect::temporary("/3d/") }))
        .route("/3d", get(|| async { Redirect::permanent("/3d/") }))
        .route("/seacells", get(|| async { Redirect::permanent("/3d/") }))
        .route("/seacells/", get(|| async { Redirect::permanent("/3d/") }))
        .route("/2d", get(|| async { Redirect::permanent("/3d/") }))
        .route("/2d/", get(|| async { Redirect::permanent("/3d/") }))
        .route("/ws", get(websocket_handler))
        .route("/api/world", get(world_json))
        .route(
            "/favicon.ico",
            get(|| async { Redirect::temporary("/favicon.svg") }),
        )
        .fallback_service(ServeDir::new(static_dir()).append_index_html_on_directories(true))
        .layer(CorsLayer::permissive())
        .with_state(app_state.clone());

    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse::<u16>()?;
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{port}")).await?;
    tracing::info!("Server listening on 0.0.0.0:{port} — open http://localhost:{port}/");

    axum::serve(listener, app)
        .with_graceful_shutdown(async {
            let _ = tokio::signal::ctrl_c().await;
        })
        .await?;

    if let (Some(pool), Some(world)) = {
        let sim = app_state.sim.read().await;
        (app_state.db.clone(), sim.world.clone())
    } {
        if let Err(error) = save_world(&pool, &world).await {
            tracing::error!("Shutdown save failed: {error}");
        }
    }

    Ok(())
}

async fn tick_once(state: &AppState) -> anyhow::Result<()> {
    let Some(pool) = state.db.clone() else {
        return Ok(());
    };
    let now = chrono::Utc::now().timestamp();
    let saved = {
        let mut sim = state.sim.write().await;
        let Some(world) = sim.world.as_mut() else {
            return Ok(());
        };
        let before = world.last_tick_unix;
        world.catch_up_to(now);
        if world.last_tick_unix == before {
            return Ok(());
        }
        world.clone()
    };
    let message = ServerMessage::Delta(saved.delta());
    if let Ok(json) = serde_json::to_string(&message) {
        let _ = state.broadcast_tx.send(json);
    }
    save_world(&pool, &saved).await
}

async fn world_json(State(state): State<AppState>) -> impl IntoResponse {
    let sim = state.sim.read().await;
    let message = match &sim.world {
        Some(world) => ServerMessage::Snapshot(world.snapshot()),
        None => ServerMessage::Unavailable {
            reason: sim.unavailable_reason.clone(),
        },
    };
    Json(message)
}

fn static_dir() -> std::path::PathBuf {
    let candidates = ["client/public", "../client/public"];
    for path in candidates {
        let candidate = std::path::Path::new(path);
        if candidate.join("index.html").exists() {
            return candidate.to_path_buf();
        }
    }
    candidates[0].into()
}

async fn websocket_handler(ws: WebSocketUpgrade, State(state): State<AppState>) -> Response {
    ws.on_upgrade(|socket| handle_websocket(socket, state))
}
