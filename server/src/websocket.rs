use axum::extract::ws::{Message, WebSocket};
use futures_util::{SinkExt, StreamExt};
use serde_json;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::game::GameMessage;
use crate::AppState;

pub async fn handle_websocket(socket: WebSocket, state: AppState) {
    let (mut sender, mut receiver) = socket.split();
    let mut player_id: Option<String> = None;

    // Handle incoming messages
    let mut rx = tokio::spawn(async move {
        while let Some(msg) = receiver.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    let message: Result<GameMessage, _> = serde_json::from_str(&text);
                    match message {
                        Ok(GameMessage::Join { player }) => {
                            player_id = Some(player.id.clone());
                            let mut game = state.game.write().await;
                            game.add_player(player.clone());

                                // Broadcast join to all players
                                let all_players = game.get_all_players();
                                let world_state = GameMessage::WorldState {
                                    players: all_players,
                                };
                                tracing::debug!("Player {} joined, total players: {}", player.id, all_players.len());
                            let world_json = serde_json::to_string(&world_state).unwrap();
                            // In a real implementation, broadcast to all connected clients
                        }
                        Ok(GameMessage::Move {
                            player_id: pid,
                            position,
                            rotation,
                        }) => {
                            let mut game = state.game.write().await;
                            game.update_player_position(&pid, position.clone(), rotation);

                            // Broadcast move to all players
                            let move_msg = GameMessage::Move {
                                player_id: pid.clone(),
                                position: position.clone(),
                                rotation,
                            };
                            let move_json = serde_json::to_string(&move_msg).unwrap();
                            // In a real implementation, broadcast to all connected clients
                        }
                        Err(e) => {
                            tracing::error!("Failed to parse message: {:?}", e);
                        }
                        _ => {}
                    }
                }
                Ok(Message::Close(_)) => {
                    break;
                }
                Err(e) => {
                    tracing::error!("WebSocket error: {:?}", e);
                    break;
                }
                _ => {}
            }
        }

        // Clean up on disconnect
        if let Some(pid) = player_id {
            let mut game = state.game.write().await;
            game.remove_player(&pid);
        }
    });

    // Keep connection alive
    let _ = tokio::spawn(async move {
        loop {
            if sender.send(Message::Ping(vec![])).await.is_err() {
                break;
            }
            tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
        }
    });

    rx.await.ok();
}

