//! WebSocket fan-out for the read-only ship viewer.

use axum::body::Bytes;
use axum::extract::ws::{Message, WebSocket};
use futures_util::{SinkExt, StreamExt};
use sim::{respond_to_client, ClientMessage, Role, ServerMessage};
use tokio::sync::mpsc;

use crate::AppState;

pub async fn handle_websocket(socket: WebSocket, state: AppState) {
    let (mut sender, mut receiver) = socket.split();
    let (tx, mut rx) = mpsc::channel::<String>(32);
    let mut broadcast_rx = state.broadcast_tx.subscribe();

    let hello = {
        let sim = state.sim.read().await;
        match &sim.world {
            Some(world) => ServerMessage::Snapshot(world.snapshot()),
            None => ServerMessage::Unavailable {
                reason: sim.unavailable_reason.clone(),
            },
        }
    };
    if let Ok(json) = serde_json::to_string(&hello) {
        let _ = tx.send(json).await;
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
                        Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => {}
                        Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
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

    while let Some(msg) = receiver.next().await {
        let Ok(Message::Text(text)) = msg else {
            continue;
        };
        let Ok(client) = serde_json::from_str::<ClientMessage>(&text) else {
            tracing::debug!("ignored viewer message");
            continue;
        };
        let response = respond_to_client(Role::Viewer, &client);
        if let Ok(json) = serde_json::to_string(&response) {
            let _ = tx.send(json).await;
        }
    }

    sender_task.abort();
}
