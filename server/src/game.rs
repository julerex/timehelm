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
    /// Game time in minutes when the server started tracking (from DB)
    game_time_at_start: i64,
    /// Real time (ms since epoch) when the server started tracking
    real_time_at_start: i64,
}

impl GameState {
    pub fn new() -> Self {
        Self {
            players: HashMap::new(),
            game_time_at_start: 0,
            real_time_at_start: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as i64,
        }
    }

    /// Initialize game time from database value
    pub fn init_game_time(&mut self, game_time_minutes: i64) {
        self.game_time_at_start = game_time_minutes;
        self.real_time_at_start = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as i64;
    }

    /// Get the current game time in minutes (advances at 1 game minute per real second)
    pub fn get_game_time_minutes(&self) -> i64 {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as i64;
        let elapsed_real_ms = now - self.real_time_at_start;
        // 1 game minute = 1 real second (1000ms)
        let elapsed_game_minutes = elapsed_real_ms / 1000;
        // Wrap at 24 hours = 1440 minutes
        (self.game_time_at_start + elapsed_game_minutes) % 1440
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
