//! WebSocket connection handler.
//!
//! Manages WebSocket connections, message routing, and player lifecycle.

use axum::body::Bytes;
use axum::extract::ws::{Message, WebSocket};
use futures_util::{SinkExt, StreamExt};
use tokio::sync::mpsc;

use crate::messages::GameMessage;
use crate::AppState;

/// Handle a WebSocket connection from a game client.
///
/// Sets up bidirectional communication:
/// - Receives messages from client (Join, Move, SetActivity)
/// - Sends messages to client (WorldState, TimeSync, PlayerJoin/Leave)
/// - Subscribes to broadcast channel for world state updates
/// - Sends periodic ping messages to keep connection alive
///
/// # Arguments
/// * `socket` - WebSocket connection
/// * `state` - Application state (game state, database, broadcast channel)
pub async fn handle_websocket(socket: WebSocket, state: AppState) {
    // Split WebSocket into sender and receiver for concurrent handling
    let (mut sender, mut receiver) = socket.split();
    // Track player ID for cleanup on disconnect
    let mut player_id: Option<String> = None;

    // Create channel for sending messages directly to this client
    // Capacity: 32 messages
    let (tx, mut rx) = mpsc::channel::<String>(32);

    // Subscribe to broadcast channel for world state updates
    // This client will receive periodic world state broadcasts
    let mut broadcast_rx = state.broadcast_tx.subscribe();

    // Send initial time sync message to client
    // Game time is derived from Unix timestamp (1 real second = 1 game minute)
    let game_time = crate::game::GameState::get_game_time_minutes();
    let time_sync = crate::messages::GameMessage::TimeSync {
        game_time_minutes: game_time,
    };
    if let Ok(json) = serde_json::to_string(&time_sync) {
        let _ = tx.send(json).await;
    }

    // Spawn task to handle outgoing messages to the client
    // Handles:
    // - Direct messages via channel (tx/rx)
    // - Broadcast world state updates
    // - Periodic ping messages (every 30 seconds) to keep connection alive
    let sender_task = tokio::spawn(async move {
        let mut ping_interval = tokio::time::interval(tokio::time::Duration::from_secs(30));
        loop {
            tokio::select! {
                // Direct message from channel
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
                // Broadcast world state update
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
                // Periodic ping to keep connection alive
                _ = ping_interval.tick() => {
                    if sender.send(Message::Ping(Bytes::new())).await.is_err() {
                        break;
                    }
                }
            }
        }
    });

    // Handle incoming messages from the client
    let rx_task = tokio::spawn(async move {
        while let Some(msg) = receiver.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    // Parse JSON message
                    let message: Result<GameMessage, _> = serde_json::from_str(&text);
                    match message {
                        // Player joining the game
                        Ok(GameMessage::Join { player }) => {
                            player_id = Some(player.id.clone());
                            let mut game = state.game.write().await;
                            game.add_player(player.clone());

                            // Send complete world state to the newly joined player
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
                        // Player movement update
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

                            // Note: Movement updates are broadcast via periodic WorldState messages
                            // (10 FPS) rather than individual Move messages for efficiency
                            let move_msg = GameMessage::Move {
                                player_id: pid.clone(),
                                position: position.clone(),
                                rotation,
                                is_moving,
                            };
                            let _move_json = serde_json::to_string(&move_msg).unwrap();
                            // In a real implementation, broadcast to all connected clients
                        }
                        // Player activity change
                        Ok(GameMessage::SetActivity {
                            player_id: pid,
                            activity,
                        }) => {
                            let mut game = state.game.write().await;
                            game.update_player_activity(&pid, activity.clone());

                            // Note: Activity changes could be broadcast, but currently
                            // they're included in periodic WorldState messages
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
                    // Client closed connection
                    break;
                }
                Err(e) => {
                    tracing::error!("WebSocket error: {:?}", e);
                    break;
                }
                _ => {}
            }
        }

        // Clean up on disconnect: remove player from game state
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
