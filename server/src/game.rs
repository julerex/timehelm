use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Player {
    pub id: String,
    pub username: String,
    pub position: Position,
    pub rotation: f32,
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

    pub fn add_player(&mut self, player: Player) {
        self.players.insert(player.id.clone(), player);
    }

    pub fn remove_player(&mut self, player_id: &str) {
        self.players.remove(player_id);
    }

    pub fn update_player_position(&mut self, player_id: &str, position: Position, rotation: f32) {
        if let Some(player) = self.players.get_mut(player_id) {
            player.position = position;
            player.rotation = rotation;
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
    },
    WorldState {
        players: Vec<Player>,
    },
}
