//! WebSocket client for server world state (WASM).

use crate::grid_state::ServerGridState;
use crate::protocol::{ClientMessage, GameMessage};
use bevy::prelude::*;
use std::sync::{Arc, Mutex};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
#[cfg(target_arch = "wasm32")]
use web_sys::{MessageEvent, WebSocket};

#[derive(Resource, Default)]
pub struct WsClientState {
    pub connected: bool,
    pub join_sent: bool,
}

#[derive(Resource, Default)]
pub struct WsInbox {
    messages: Arc<Mutex<Vec<String>>>,
    #[cfg(target_arch = "wasm32")]
    socket: Option<WebSocket>,
}

pub struct WsClientPlugin;

impl Plugin for WsClientPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WsClientState>()
            .init_resource::<WsInbox>()
            .init_resource::<ServerGridState>()
            .add_systems(Startup, connect_ws)
            .add_systems(Update, drain_ws_messages);
    }
}

#[cfg(target_arch = "wasm32")]
fn ws_url() -> String {
    let window = web_sys::window().expect("window");
    let host = window.location().host().unwrap_or_default();
    let protocol = window
        .location()
        .protocol()
        .unwrap_or_else(|_| "http:".to_string());
    let ws_proto = if protocol.starts_with("https") {
        "wss"
    } else {
        "ws"
    };
    format!("{ws_proto}://{host}/ws")
}

#[cfg(target_arch = "wasm32")]
fn connect_ws(mut inbox: ResMut<WsInbox>, mut ws: ResMut<WsClientState>) {
    let url = ws_url();
    let Ok(socket) = WebSocket::new(&url) else {
        return;
    };
    let queue = Arc::new(Mutex::new(Vec::<String>::new()));
    let queue_clone = queue.clone();
    let onmessage = Closure::<dyn FnMut(MessageEvent)>::new(move |e: MessageEvent| {
        if let Ok(text) = e.data().dyn_into::<js_sys::JsString>() {
            if let Ok(mut q) = queue_clone.lock() {
                q.push(String::from(text));
            }
        }
    });
    socket.set_onmessage(Some(onmessage.as_ref().unchecked_ref()));
    onmessage.forget();
    inbox.messages = queue;
    inbox.socket = Some(socket);
    ws.connected = true;
}

#[cfg(not(target_arch = "wasm32"))]
fn connect_ws(_inbox: ResMut<WsInbox>, _ws: ResMut<WsClientState>) {}

fn drain_ws_messages(
    inbox: Res<WsInbox>,
    mut grid: ResMut<ServerGridState>,
    mut ws: ResMut<WsClientState>,
) {
    let Ok(mut messages) = inbox.messages.lock() else {
        return;
    };
    for text in messages.drain(..) {
        let Ok(msg) = serde_json::from_str::<GameMessage>(&text) else {
            continue;
        };
        match msg {
            GameMessage::WorldState {
                entities,
                game_time_seconds,
                players,
            } => {
                grid.entities = entities;
                grid.game_time_seconds = game_time_seconds;
                if grid.local_entity_id.is_none() {
                    if let Some(p) = players.iter().find(|p| p.id == grid.local_player_id) {
                        grid.local_entity_id = Some(p.entity_id);
                    }
                }
            }
            GameMessage::TimeSync { game_time_seconds } => {
                grid.game_time_seconds = game_time_seconds;
            }
        }
    }
}

#[cfg(target_arch = "wasm32")]
fn send_raw(json: &str, inbox: &WsInbox) {
    if let Some(socket) = &inbox.socket {
        let _ = socket.send_with_str(json);
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn send_raw(_json: &str, _inbox: &WsInbox) {}

pub fn send_join_with_inbox(grid: &ServerGridState, spawn_z: i32, inbox: &WsInbox) {
    let msg = ClientMessage::Join {
        player_id: grid.local_player_id.clone(),
        username: "player".to_string(),
        spawn_z,
    };
    if let Ok(json) = serde_json::to_string(&msg) {
        send_raw(&json, inbox);
    }
}

pub fn send_move_with_inbox(entity_id: i64, to_x: i32, to_y: i32, to_z: i32, inbox: &WsInbox) {
    let msg = ClientMessage::Move {
        entity_id,
        to_x,
        to_y,
        to_z,
    };
    if let Ok(json) = serde_json::to_string(&msg) {
        send_raw(&json, inbox);
    }
}
