//! Load ship layout and connect WebSocket from the game server (WASM).

use crate::deck_layout::DeckLayouts;
use crate::grid_state::ServerGridState;
use crate::load_screen::GamePhase;
use crate::world_client::{apply_deck_cells, parse_all_cells_response};
use crate::ws_client::{send_join_with_inbox, WsClientState, WsInbox};
use bevy::prelude::*;
use bevy::tasks::{AsyncComputeTaskPool, Task};

#[derive(Resource, Default)]
pub struct ServerWorldLoad {
    pub task: Option<Task<Result<DeckLayouts, String>>>,
    pub done: bool,
}

pub struct ServerWorldPlugin;

impl Plugin for ServerWorldPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ServerWorldLoad>()
            .add_systems(Startup, start_server_world_load)
            .add_systems(Update, poll_server_world_load);
    }
}

fn api_cells_url() -> String {
    #[cfg(target_arch = "wasm32")]
    {
        let window = web_sys::window().expect("window");
        let origin = window.location().origin().unwrap_or_default();
        format!("{origin}/api/cells")
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        "http://localhost:8080/api/cells".to_string()
    }
}

fn start_server_world_load(mut load: ResMut<ServerWorldLoad>) {
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = &mut load;
        return;
    }
    #[cfg(target_arch = "wasm32")]
    {
        let url = api_cells_url();
        load.task =
            Some(AsyncComputeTaskPool::get().spawn(async move { fetch_cells_json(&url).await }));
    }
}

#[cfg(target_arch = "wasm32")]
async fn fetch_cells_json(url: &str) -> Result<DeckLayouts, String> {
    use wasm_bindgen::JsCast;
    use wasm_bindgen_futures::JsFuture;
    let window = web_sys::window().ok_or("no window")?;
    let resp_val = JsFuture::from(window.fetch_with_str(url))
        .await
        .map_err(|e| format!("fetch failed: {e:?}"))?;
    let resp: web_sys::Response = resp_val.dyn_into().map_err(|_| "not Response")?;
    if !resp.ok() {
        return Err(format!("HTTP {}", resp.status()));
    }
    let text_val = JsFuture::from(resp.text().map_err(|_| "text() failed")?)
        .await
        .map_err(|e| format!("text: {e:?}"))?;
    let text = text_val.as_string().ok_or("body not string")?;
    let parsed = parse_all_cells_response(&text).map_err(|e| format!("json: {e}"))?;
    let mut cell_box = crate::cell_box::CellBox::new();
    apply_deck_cells(&mut cell_box, &parsed.cells);
    Ok(DeckLayouts {
        cells: cell_box,
        decks: (0..crate::deck_layout::NUM_DECKS)
            .map(|_| crate::deck_layout::DeckMeta {})
            .collect(),
        entities: Default::default(),
    })
}

fn poll_server_world_load(
    mut load: ResMut<ServerWorldLoad>,
    mut layouts: ResMut<DeckLayouts>,
    mut phase: ResMut<NextState<GamePhase>>,
    mut grid: ResMut<ServerGridState>,
    mut ws: ResMut<WsClientState>,
    inbox: Res<WsInbox>,
) {
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (
            &mut load,
            &mut layouts,
            &mut phase,
            &mut grid,
            &mut ws,
            &inbox,
        );
        return;
    }
    #[cfg(target_arch = "wasm32")]
    {
        if load.done {
            return;
        }
        let Some(task) = load.task.as_mut() else {
            return;
        };
        if let Some(result) = futures_lite::future::block_on(futures_lite::future::poll_once(task))
        {
            load.task = None;
            match result {
                Ok(deck_layouts) => {
                    *layouts = deck_layouts;
                    grid.local_player_id = format!("player_{}", js_player_nonce());
                    send_join_with_inbox(&grid, crate::deck_layout::SIM_DECK_INDEX as i32, &inbox);
                    ws.join_sent = true;
                    load.done = true;
                    phase.set(GamePhase::InGame);
                }
                Err(e) => {
                    tracing::warn!("server world load failed: {e}");
                }
            }
        }
    }
}

#[cfg(target_arch = "wasm32")]
fn js_player_nonce() -> u32 {
    (js_sys::Math::random() * 1_000_000.0) as u32
}
