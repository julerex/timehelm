use crate::game::{Activity, Entity, Player, Position};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum GameMessage {
    Join {
        player: Player,
    },
    Leave {
        player_id: String,
    },
    Move {
        player_id: String,
        position: Position,
        rotation: f32,
        #[serde(default)]
        is_moving: bool,
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
        players: Vec<Player>,
        entities: Vec<Entity>,
    },
    /// Server -> Client: Sync game time (in game minutes, where 0 = midnight)
    TimeSync {
        game_time_minutes: i64,
    },
}
