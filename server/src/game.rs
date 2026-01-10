use crate::physics::PhysicsWorld;
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

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum EntityType {
    Human,
    Ball,
}

impl EntityType {
    pub fn as_str(&self) -> &'static str {
        match self {
            EntityType::Human => "human",
            EntityType::Ball => "ball",
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Entity {
    pub id: String,
    pub entity_type: EntityType,
    pub position: Position,
    pub rotation: Rotation,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Rotation {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

pub struct GameState {
    pub players: HashMap<String, Player>,
    pub entities: HashMap<String, Entity>,
    pub physics: PhysicsWorld,
}

impl GameState {
    pub fn new() -> Self {
        let mut physics = PhysicsWorld::new();

        // Create initial bouncy ball
        // Pole is at (-500, -400), ball is 200 units away at (-300, -400)
        let ball_id = format!("ball_{}", uuid::Uuid::new_v4());
        physics.create_bouncy_ball(ball_id.clone(), -300.0, -400.0);

        let mut entities = HashMap::new();
        entities.insert(
            ball_id.clone(),
            Entity {
                id: ball_id,
                entity_type: EntityType::Ball,
                position: Position {
                    x: -300.0, // 200 units away from pole at (-500, -400)
                    y: 500.0,  // Start at 5 meters (more visible than 10m)
                    z: -400.0,
                },
                rotation: Rotation {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                },
            },
        );

        Self {
            players: HashMap::new(),
            entities,
            physics,
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
        // Also create entity for player
        let entity = self.player_to_entity(&player);
        self.add_entity(entity);
        self.players.insert(player.id.clone(), player);
    }

    pub fn remove_player(&mut self, player_id: &str) {
        let entity_id = format!("human_{}", player_id);
        self.entities.remove(&entity_id);
        self.physics.remove_entity(&entity_id);
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
            player.position = position.clone();
            player.rotation = rotation;
            player.is_moving = is_moving;

            // Update corresponding entity
            let entity_id = format!("human_{}", player_id);
            if let Some(entity) = self.entities.get_mut(&entity_id) {
                entity.position = position;
                entity.rotation = Rotation {
                    x: 0.0,
                    y: rotation,
                    z: 0.0,
                };
                self.physics.update_human_position(
                    &entity_id,
                    entity.position.x,
                    entity.position.y,
                    entity.position.z,
                );
            }
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

    pub fn add_entity(&mut self, entity: Entity) {
        match entity.entity_type {
            EntityType::Human => {
                self.physics.create_human(
                    entity.id.clone(),
                    entity.position.x,
                    entity.position.y,
                    entity.position.z,
                );
            }
            EntityType::Ball => {
                self.physics.create_bouncy_ball(
                    entity.id.clone(),
                    entity.position.x,
                    entity.position.z,
                );
            }
        }
        self.entities.insert(entity.id.clone(), entity);
    }

    pub fn update_entity_position(&mut self, entity_id: &str, position: Position) {
        if let Some(entity) = self.entities.get_mut(entity_id) {
            entity.position = position.clone();
            if matches!(entity.entity_type, EntityType::Human) {
                self.physics
                    .update_human_position(entity_id, position.x, position.y, position.z);
            }
        }
    }

    pub fn update_entity_rotation(&mut self, entity_id: &str, rotation: Rotation) {
        if let Some(entity) = self.entities.get_mut(entity_id) {
            entity.rotation = rotation;
        }
    }

    pub fn get_all_entities(&self) -> Vec<Entity> {
        self.entities.values().cloned().collect()
    }

    /// Step physics and update entity positions from physics
    pub fn step_physics(&mut self, dt: f64) {
        self.physics.step(dt);

        // Update entity positions from physics
        for entity in self.entities.values_mut() {
            if let Some((x, y, z)) = self.physics.get_entity_position(&entity.id) {
                entity.position = Position { x, y, z };
            }
            if let Some((rx, ry, rz)) = self.physics.get_entity_rotation(&entity.id) {
                entity.rotation = Rotation {
                    x: rx,
                    y: ry,
                    z: rz,
                };
            }
        }
    }

    /// Convert player to entity
    pub fn player_to_entity(&self, player: &Player) -> Entity {
        Entity {
            id: format!("human_{}", player.id),
            entity_type: EntityType::Human,
            position: player.position.clone(),
            rotation: Rotation {
                x: 0.0,
                y: player.rotation,
                z: 0.0,
            },
        }
    }
}
