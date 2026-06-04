//! Ephemeral multiplayer connection state (positions live in PostgreSQL).

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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

/// Connected player metadata (grid position is on the linked entity in DB).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PlayerMeta {
    pub id: String,
    pub username: String,
    pub entity_id: i64,
    #[serde(default)]
    pub is_moving: bool,
    #[serde(default)]
    pub activity: Activity,
}

pub struct GameState {
    pub players: HashMap<String, PlayerMeta>,
}

impl GameState {
    pub fn new() -> Self {
        Self {
            players: HashMap::new(),
        }
    }

    pub fn add_player(&mut self, meta: PlayerMeta) {
        self.players.insert(meta.id.clone(), meta);
    }

    pub fn remove_player(&mut self, player_id: &str) -> Option<PlayerMeta> {
        self.players.remove(player_id)
    }

    pub fn get_player(&self, player_id: &str) -> Option<&PlayerMeta> {
        self.players.get(player_id)
    }

    pub fn get_player_mut(&mut self, player_id: &str) -> Option<&mut PlayerMeta> {
        self.players.get_mut(player_id)
    }

    pub fn get_all_players(&self) -> Vec<PlayerMeta> {
        self.players.values().cloned().collect()
    }

    pub fn update_activity(&mut self, player_id: &str, activity: Activity) {
        if let Some(p) = self.players.get_mut(player_id) {
            p.activity = activity;
        }
    }

    pub fn set_moving(&mut self, player_id: &str, is_moving: bool) {
        if let Some(p) = self.players.get_mut(player_id) {
            p.is_moving = is_moving;
        }
    }
}
