use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Daily routine activities that characters can be engaged in
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "snake_case")]
pub enum Activity {
    #[default]
    Idle,
    Sleeping,
    Eating,
    Cooking,
    Working,
    Exercising,
    Socializing,
    Shopping,
    Cleaning,
    Bathing,
    Reading,
    WatchingTv,
    Gaming,
    Commuting,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Player {
    pub id: String,
    pub username: String,
    pub position: Position,
    pub rotation: f32,
    #[serde(default)]
    pub is_moving: bool,
    #[serde(default)]
    pub activity: Activity,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Position {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[derive(Clone, Debug)]
pub struct GameState {
    pub players: HashMap<String, Player>,
}

impl GameState {
    pub fn new() -> Self {
        Self {
            players: HashMap::new(),
        }
    }

    /// Get the current game time in minutes, derived from Unix time.
    /// Game time = Unix seconds (1 real second = 1 game minute).
    pub fn get_game_time_minutes() -> i64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64
    }

    pub fn add_player(&mut self, player: Player) {
        self.players.insert(player.id.clone(), player);
    }

    pub fn remove_player(&mut self, player_id: &str) {
        self.players.remove(player_id);
    }

    pub fn update_player_position(
        &mut self,
        player_id: &str,
        position: Position,
        rotation: f32,
        is_moving: bool,
    ) {
        if let Some(player) = self.players.get_mut(player_id) {
            player.position = position;
            player.rotation = rotation;
            player.is_moving = is_moving;
        }
    }

    pub fn update_player_activity(&mut self, player_id: &str, activity: Activity) {
        if let Some(player) = self.players.get_mut(player_id) {
            player.activity = activity;
        }
    }

    pub fn get_all_players(&self) -> Vec<Player> {
        self.players.values().cloned().collect()
    }
}

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
    },
    /// Server -> Client: Sync game time (in game minutes, where 0 = midnight)
    TimeSync {
        game_time_minutes: i64,
    },
}
