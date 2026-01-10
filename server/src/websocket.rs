use axum::body::Bytes;
use axum::extract::ws::{Message, WebSocket};
use futures_util::{SinkExt, StreamExt};
use tokio::sync::mpsc;

use crate::game::GameMessage;
use crate::AppState;

pub async fn handle_websocket(socket: WebSocket, state: AppState) {
    let (mut sender, mut receiver) = socket.split();
    let mut player_id: Option<String> = None;

    // Create channel for sending messages to this client
    let (tx, mut rx) = mpsc::channel::<String>(32);

    // Send initial time sync - game time is derived from Unix time
    let game_time = crate::game::GameState::get_game_time_minutes();
    let time_sync = GameMessage::TimeSync {
        game_time_minutes: game_time,
    };
    if let Ok(json) = serde_json::to_string(&time_sync) {
        let _ = tx.send(json).await;
    }

    // Spawn task to forward messages to websocket and handle pings
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
                _ = ping_interval.tick() => {
                    if sender.send(Message::Ping(Bytes::new())).await.is_err() {
                        break;
                    }
                }
            }
        }
    });

    // Handle incoming messages
    let rx_task = tokio::spawn(async move {
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
                            let all_entities = game.get_all_entities();
                            let player_count = all_players.len();
                            let world_state = GameMessage::WorldState {
                                players: all_players,
                                entities: all_entities,
                            };
                            let player_id_ref = &player.id;
                            tracing::debug!(
                                "Player {player_id_ref} joined, total players: {player_count}"
                            );
                            if let Ok(world_json) = serde_json::to_string(&world_state) {
                                let _ = tx.send(world_json).await;
                            }
                        }
                        Ok(GameMessage::Move {
                            player_id: pid,
                            position,
                            rotation,
                            is_moving,
                        }) => {
                            let mut game = state.game.write().await;
                            game.update_player_position(
                                &pid,
                                position.clone(),
                                rotation,
                                is_moving,
                            );

                            // Broadcast move to all players
                            let move_msg = GameMessage::Move {
                                player_id: pid.clone(),
                                position: position.clone(),
                                rotation,
                                is_moving,
                            };
                            let _move_json = serde_json::to_string(&move_msg).unwrap();
                            // In a real implementation, broadcast to all connected clients
                        }
                        Ok(GameMessage::SetActivity {
                            player_id: pid,
                            activity,
                        }) => {
                            let mut game = state.game.write().await;
                            game.update_player_activity(&pid, activity.clone());

                            // Broadcast activity change to all players
                            let activity_msg = GameMessage::ActivityChanged {
                                player_id: pid.clone(),
                                activity,
                            };
                            let _activity_json = serde_json::to_string(&activity_msg).unwrap();
                            tracing::debug!("Player {pid} activity changed");
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

    // Wait for either task to complete
    tokio::select! {
        _ = sender_task => {}
        _ = rx_task => {}
    }
}
