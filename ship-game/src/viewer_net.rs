//! WebSocket client. The simulation never runs here.

use bevy::prelude::*;
use sim::{ServerMessage, WorldSnapshot};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::Mutex;

#[derive(Resource)]
pub struct NetInbox {
    rx: Mutex<Receiver<String>>,
}

#[derive(Resource, Default)]
pub struct LiveWorld {
    pub snapshot: Option<WorldSnapshot>,
    pub unavailable: Option<String>,
    pub received_at: f32,
    pub ever_connected: bool,
}

pub fn start_inbox() -> NetInbox {
    let (tx, rx) = mpsc::channel();
    spawn_socket(tx);
    NetInbox { rx: Mutex::new(rx) }
}

pub fn drain_inbox(time: Res<Time>, inbox: Res<NetInbox>, mut live: ResMut<LiveWorld>) {
    let Ok(rx) = inbox.rx.lock() else {
        return;
    };
    while let Ok(text) = rx.try_recv() {
        if text.is_empty() {
            continue;
        }
        match serde_json::from_str::<ServerMessage>(&text) {
            Ok(ServerMessage::Snapshot(snapshot)) => {
                live.received_at = time.elapsed_secs();
                live.ever_connected = true;
                live.unavailable = None;
                live.snapshot = Some(snapshot);
            }
            Ok(ServerMessage::Delta(delta)) => {
                live.received_at = time.elapsed_secs();
                live.ever_connected = true;
                if let Some(snapshot) = live.snapshot.as_mut() {
                    snapshot.tick_unix = delta.tick_unix;
                    snapshot.clock_utc = delta.clock_utc;
                    snapshot.crew = delta.crew;
                    snapshot.stocks = delta.stocks;
                    snapshot.totals = delta.totals;
                    snapshot.power = delta.power;
                }
            }
            Ok(ServerMessage::Unavailable { reason }) => {
                live.unavailable = Some(reason);
                live.snapshot = None;
            }
            Ok(ServerMessage::CommandRejected { reason }) => {
                info!("viewer command rejected: {reason}");
            }
            Err(error) => {
                debug!("ignored socket payload: {error}");
            }
        }
    }
}

fn spawn_socket(tx: Sender<String>) {
    #[cfg(target_arch = "wasm32")]
    connect_wasm(tx);
    #[cfg(not(target_arch = "wasm32"))]
    {
        std::thread::spawn(move || native_loop(tx));
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn native_loop(tx: Sender<String>) {
    let url = std::env::var("TIMEHELM_WS").unwrap_or_else(|_| "ws://127.0.0.1:8080/ws".to_string());
    loop {
        match tungstenite::connect(&url) {
            Ok((mut socket, _)) => {
                while let Ok(message) = socket.read() {
                    match message {
                        tungstenite::Message::Text(text) => {
                            if tx.send(text.to_string()).is_err() {
                                return;
                            }
                        }
                        tungstenite::Message::Ping(payload) => {
                            let _ = socket.send(tungstenite::Message::Pong(payload));
                        }
                        tungstenite::Message::Close(_) => break,
                        _ => {}
                    }
                }
            }
            Err(error) => {
                eprintln!("ship socket: {error}");
            }
        }
        std::thread::sleep(std::time::Duration::from_secs(2));
    }
}

#[cfg(target_arch = "wasm32")]
fn connect_wasm(tx: Sender<String>) {
    use wasm_bindgen::closure::Closure;
    use wasm_bindgen::JsCast;
    use web_sys::{CloseEvent, MessageEvent, WebSocket};

    let url = ws_url();
    let Ok(socket) = WebSocket::new(&url) else {
        schedule_retry(tx);
        return;
    };
    let tx_msg = tx.clone();
    let on_message = Closure::wrap(Box::new(move |event: MessageEvent| {
        if let Some(text) = event.data().as_string() {
            let _ = tx_msg.send(text);
        }
    }) as Box<dyn FnMut(MessageEvent)>);
    socket.set_onmessage(Some(on_message.as_ref().unchecked_ref()));
    on_message.forget();

    let on_close = Closure::wrap(Box::new(move |_event: CloseEvent| {
        schedule_retry(tx.clone());
    }) as Box<dyn FnMut(CloseEvent)>);
    socket.set_onclose(Some(on_close.as_ref().unchecked_ref()));
    on_close.forget();
    std::mem::forget(socket);
}

#[cfg(target_arch = "wasm32")]
fn schedule_retry(tx: Sender<String>) {
    use wasm_bindgen::closure::Closure;
    use wasm_bindgen::JsCast;

    let Some(window) = web_sys::window() else {
        return;
    };
    let retry = Closure::once(move || connect_wasm(tx));
    let _ = window.set_timeout_with_callback_and_timeout_and_arguments_0(
        retry.as_ref().unchecked_ref(),
        2_000,
    );
    retry.forget();
}

#[cfg(target_arch = "wasm32")]
fn ws_url() -> String {
    let Some(window) = web_sys::window() else {
        return "ws://127.0.0.1:8080/ws".to_string();
    };
    let location = window.location();
    let protocol = location.protocol().unwrap_or_else(|_| "http:".to_string());
    let host = location
        .host()
        .unwrap_or_else(|_| "127.0.0.1:8080".to_string());
    let scheme = if protocol == "https:" { "wss" } else { "ws" };
    format!("{scheme}://{host}/ws")
}
