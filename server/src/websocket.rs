//! WebSocket handler — grid moves and world snapshots from PostgreSQL.

use axum::body::Bytes;
use axum::extract::ws::{Message, WebSocket};
use futures_util::{SinkExt, StreamExt};
use tokio::sync::mpsc;

use crate::db::{
    create_entity, delete_entity, fetch_entities, find_spawn_cell, get_game_time_seconds,
    move_entity,
};
use crate::game::{Activity, PlayerMeta};
use crate::messages::GameMessage;
use crate::AppState;
use std::sync::Arc;
use tokio::sync::RwLock;

const DEFAULT_SPAWN_DECK: i32 = 4;

pub async fn handle_websocket(socket: WebSocket, state: AppState) {
    let (mut sender, mut receiver) = socket.split();
    let mut player_id: Option<String> = None;
    let mut entity_id: Option<i64> = None;

    let (tx, mut rx) = mpsc::channel::<String>(32);
    let mut broadcast_rx = state.broadcast_tx.subscribe();

    if let Some(pool) = &state.db {
        if let Ok(t) = get_game_time_seconds(pool).await {
            let time_sync = GameMessage::TimeSync {
                game_time_seconds: t,
            };
            if let Ok(json) = serde_json::to_string(&time_sync) {
                let _ = tx.send(json).await;
            }
        }
    }

    let sender_task = tokio::spawn(async move {
        let mut ping_interval = tokio::time::interval(tokio::time::Duration::from_secs(30));
        loop {
            tokio::select! {
                msg = rx.recv() => {
                    match msg {
                        Some(text) => {
                            if sender.send(Message::Text(text.into())).await.is_err() {
                                break;
                            }
                        }
                        None => break,
                    }
                }
                broadcast_msg = broadcast_rx.recv() => {
                    match broadcast_msg {
                        Ok(text) => {
                            if sender.send(Message::Text(text.into())).await.is_err() {
                                break;
                            }
                        }
                        Err(_) => break,
                    }
                }
                _ = ping_interval.tick() => {
                    if sender.send(Message::Ping(Bytes::new())).await.is_err() {
                        break;
                    }
                }
            }
        }
    });

    let rx_task = tokio::spawn(async move {
        while let Some(msg) = receiver.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    let message: Result<GameMessage, _> = serde_json::from_str(&text);
                    match message {
                        Ok(GameMessage::Join {
                            player_id: pid,
                            username,
                            spawn_z,
                        }) => {
                            let Some(pool) = &state.db else {
                                tracing::warn!("Join without database");
                                continue;
                            };
                            let deck = if (0..=19).contains(&spawn_z) {
                                spawn_z
                            } else {
                                DEFAULT_SPAWN_DECK
                            };
                            let spawn = match find_spawn_cell(pool, deck).await {
                                Ok(Some(s)) => s,
                                Ok(None) => {
                                    tracing::warn!("No spawn cell on deck {deck}");
                                    continue;
                                }
                                Err(e) => {
                                    tracing::error!("find_spawn_cell: {e}");
                                    continue;
                                }
                            };
                            let eid = match create_entity(pool, "human", spawn.0, spawn.1, spawn.2)
                                .await
                            {
                                Ok(id) => id,
                                Err(e) => {
                                    tracing::error!("create_entity: {e}");
                                    continue;
                                }
                            };
                            player_id = Some(pid.clone());
                            entity_id = Some(eid);
                            {
                                let mut game = state.game.write().await;
                                game.add_player(PlayerMeta {
                                    id: pid.clone(),
                                    username,
                                    entity_id: eid,
                                    is_moving: false,
                                    activity: Activity::default(),
                                });
                            }
                            if let Ok(world) = build_world_state(pool, &state.game).await {
                                if let Ok(json) = serde_json::to_string(&world) {
                                    let _ = tx.send(json).await;
                                }
                            }
                        }
                        Ok(GameMessage::Move {
                            entity_id: eid,
                            to_x,
                            to_y,
                            to_z,
                        }) => {
                            let Some(pool) = &state.db else {
                                continue;
                            };
                            if let Err(e) = move_entity(pool, eid, to_x, to_y, to_z).await {
                                tracing::debug!("move_entity failed: {e}");
                            } else if let Some(pid) = &player_id {
                                let mut game = state.game.write().await;
                                game.set_moving(pid, true);
                            }
                        }
                        Ok(GameMessage::SetActivity {
                            player_id: pid,
                            activity,
                        }) => {
                            let mut game = state.game.write().await;
                            game.update_activity(&pid, activity);
                        }
                        Err(e) => tracing::error!("Failed to parse message: {e:?}"),
                        _ => {}
                    }
                }
                Ok(Message::Close(_)) => break,
                Err(e) => {
                    tracing::error!("WebSocket error: {e:?}");
                    break;
                }
                _ => {}
            }
        }

        if let (Some(pid), Some(eid), Some(pool)) = (player_id, entity_id, &state.db) {
            let _ = delete_entity(pool, eid).await;
            let mut game = state.game.write().await;
            game.remove_player(&pid);
        }
    });

    tokio::select! {
        _ = sender_task => {}
        _ = rx_task => {}
    }
}

pub async fn build_world_state(
    pool: &sqlx::PgPool,
    game: &Arc<RwLock<crate::game::GameState>>,
) -> anyhow::Result<GameMessage> {
    let entities = fetch_entities(pool).await?;
    let game_time_seconds = get_game_time_seconds(pool).await.unwrap_or(0);
    let players = game.read().await.get_all_players();
    Ok(GameMessage::WorldState {
        entities,
        players,
        game_time_seconds,
    })
}
