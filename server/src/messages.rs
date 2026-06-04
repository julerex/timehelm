//! WebSocket message types for cell-grid multiplayer.

use crate::db::GridEntity;
use crate::game::{Activity, PlayerMeta};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum GameMessage {
    Join {
        player_id: String,
        username: String,
        #[serde(default)]
        spawn_z: i32,
    },
    Leave {
        player_id: String,
    },
    Move {
        entity_id: i64,
        to_x: i32,
        to_y: i32,
        to_z: i32,
    },
    SetActivity {
        player_id: String,
        activity: Activity,
    },
    ActivityChanged {
        player_id: String,
        activity: Activity,
    },
    WorldState {
        entities: Vec<GridEntity>,
        players: Vec<PlayerMeta>,
        #[serde(default)]
        game_time_seconds: i64,
    },
    TimeSync {
        game_time_seconds: i64,
    },
}
