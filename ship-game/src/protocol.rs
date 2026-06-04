//! JSON protocol shared with the Time Helm server (cell grid).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct CellApiRow {
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub bow_wall: String,
    pub stern_wall: String,
    pub port_wall: String,
    pub starboard_wall: String,
    pub floor: String,
    pub ceiling: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CellsResponse {
    pub deck: i32,
    pub cells: Vec<CellApiRow>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AllCellsResponse {
    pub cells: Vec<CellApiRow>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GridEntity {
    pub id: i64,
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub entity_type: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EntitiesResponse {
    pub entities: Vec<GridEntity>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PlayerMeta {
    pub id: String,
    pub username: String,
    pub entity_id: i64,
    #[serde(default)]
    pub is_moving: bool,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type")]
pub enum GameMessage {
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

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type")]
pub enum ClientMessage {
    Join {
        player_id: String,
        username: String,
        #[serde(default)]
        spawn_z: i32,
    },
    Move {
        entity_id: i64,
        to_x: i32,
        to_y: i32,
        to_z: i32,
    },
}
