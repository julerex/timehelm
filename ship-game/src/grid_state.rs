//! Server-synchronized entity positions (cell grid).

use crate::protocol::GridEntity;
use bevy::prelude::*;
use std::collections::HashMap;

#[derive(Resource, Default)]
pub struct ServerGridState {
    pub entities: Vec<GridEntity>,
    pub game_time_seconds: i64,
    pub local_entity_id: Option<i64>,
    pub local_player_id: String,
}

impl ServerGridState {
    pub fn entity_map(&self) -> HashMap<i64, &GridEntity> {
        self.entities.iter().map(|e| (e.id, e)).collect()
    }
}

#[derive(Component)]
pub struct ServerEntityVisual {
    pub entity_id: i64,
}
