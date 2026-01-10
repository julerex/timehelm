//! Game state management module.
//!
//! Handles player and entity state, game time, and physics integration.

use crate::physics::PhysicsWorld;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Daily routine activities that characters can be engaged in.
///
/// Activities represent what a player character is currently doing,
/// which may affect behavior, animations, or game mechanics.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "snake_case")]
pub enum Activity {
    /// Character is idle (default state)
    #[default]
    Idle,
    /// Character is sleeping
    Sleeping,
    /// Character is eating
    Eating,
    /// Character is cooking
    Cooking,
    /// Character is working
    Working,
    /// Character is exercising
    Exercising,
    /// Character is socializing with others
    Socializing,
    /// Character is shopping
    Shopping,
    /// Character is cleaning
    Cleaning,
    /// Character is bathing
    Bathing,
    /// Character is reading
    Reading,
    /// Character is watching TV
    WatchingTv,
    /// Character is gaming
    Gaming,
    /// Character is commuting/traveling
    Commuting,
}

/// Represents a player in the game world.
///
/// Contains player identity, position, orientation, movement state, and current activity.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Player {
    /// Unique player identifier
    pub id: String,
    /// Player's display username
    pub username: String,
    /// Current position in 3D space (units are centimeters)
    pub position: Position,
    /// Rotation around Y-axis (in radians)
    pub rotation: f32,
    /// Whether the player is currently moving
    #[serde(default)]
    pub is_moving: bool,
    /// Current activity the player is engaged in
    #[serde(default)]
    pub activity: Activity,
}

/// 3D position in the game world.
///
/// Units are in centimeters (1 unit = 1 cm).
/// Y-axis is vertical (height).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Position {
    /// X coordinate (horizontal, east-west)
    pub x: f32,
    /// Y coordinate (vertical, height)
    pub y: f32,
    /// Z coordinate (horizontal, north-south)
    pub z: f32,
}

/// Type of entity in the game world.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum EntityType {
    /// Human entity (player character)
    Human,
    /// Ball entity (physics object)
    Ball,
}

impl EntityType {
    /// Get the string representation of the entity type.
    pub fn as_str(&self) -> &'static str {
        match self {
            EntityType::Human => "human",
            EntityType::Ball => "ball",
        }
    }
}

/// Represents a game entity (non-player object).
///
/// Entities can be physics objects like balls, or other interactive objects.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Entity {
    /// Unique entity identifier
    pub id: String,
    /// Type of entity
    pub entity_type: EntityType,
    /// Current position in 3D space (units are centimeters)
    pub position: Position,
    /// Rotation in Euler angles (radians)
    pub rotation: Rotation,
}

/// 3D rotation represented as Euler angles.
///
/// Angles are in radians.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Rotation {
    /// Rotation around X-axis (pitch)
    pub x: f32,
    /// Rotation around Y-axis (yaw)
    pub y: f32,
    /// Rotation around Z-axis (roll)
    pub z: f32,
}

/// Main game state container.
///
/// Manages all players, entities, and the physics simulation.
/// Thread-safe access is provided via `Arc<RwLock<GameState>>`.
pub struct GameState {
    /// Map of player ID to Player data
    pub players: HashMap<String, Player>,
    /// Map of entity ID to Entity data
    pub entities: HashMap<String, Entity>,
    /// Physics simulation world
    pub physics: PhysicsWorld,
}

impl GameState {
    /// Create a new game state with initial entities.
    ///
    /// Initializes physics world and creates a bouncy ball entity
    /// positioned near the pole at (-500, -400).
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
                    y: 500.0,  // Start at 5 meters (500cm) for visibility
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
    ///
    /// Game time = Unix seconds (1 real second = 1 game minute).
    /// This provides a persistent, server-authoritative time source.
    pub fn get_game_time_minutes() -> i64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64
    }

    /// Add a new player to the game state.
    ///
    /// Also creates a corresponding human entity for physics simulation.
    pub fn add_player(&mut self, player: Player) {
        // Create corresponding entity for player (for physics simulation)
        let entity = self.player_to_entity(&player);
        self.add_entity(entity);
        self.players.insert(player.id.clone(), player);
    }

    /// Remove a player from the game state.
    ///
    /// Removes player data, associated entity, and physics body.
    pub fn remove_player(&mut self, player_id: &str) {
        let entity_id = format!("human_{}", player_id);
        self.entities.remove(&entity_id);
        self.physics.remove_entity(&entity_id);
        self.players.remove(player_id);
    }

    /// Update a player's position, rotation, and movement state.
    ///
    /// Also updates the corresponding entity and physics body position.
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

            // Update corresponding entity and physics body
            let entity_id = format!("human_{}", player_id);
            if let Some(entity) = self.entities.get_mut(&entity_id) {
                entity.position = position;
                entity.rotation = Rotation {
                    x: 0.0,
                    y: rotation,
                    z: 0.0,
                };
                // Update physics body position (for collision detection)
                self.physics.update_human_position(
                    &entity_id,
                    entity.position.x,
                    entity.position.y,
                    entity.position.z,
                );
            }
        }
    }

    /// Update a player's current activity.
    pub fn update_player_activity(&mut self, player_id: &str, activity: Activity) {
        if let Some(player) = self.players.get_mut(player_id) {
            player.activity = activity;
        }
    }

    /// Get a copy of all players in the game.
    pub fn get_all_players(&self) -> Vec<Player> {
        self.players.values().cloned().collect()
    }

    /// Add a new entity to the game state.
    ///
    /// Creates the corresponding physics body based on entity type.
    pub fn add_entity(&mut self, entity: Entity) {
        match entity.entity_type {
            EntityType::Human => {
                // Create kinematic physics body for human (position-controlled)
                self.physics.create_human(
                    entity.id.clone(),
                    entity.position.x,
                    entity.position.y,
                    entity.position.z,
                );
            }
            EntityType::Ball => {
                // Create dynamic physics body for ball (physics-controlled)
                self.physics.create_bouncy_ball(
                    entity.id.clone(),
                    entity.position.x,
                    entity.position.z,
                );
            }
        }
        self.entities.insert(entity.id.clone(), entity);
    }

    /// Update an entity's position.
    ///
    /// Also updates physics body if the entity is a human.
    pub fn update_entity_position(&mut self, entity_id: &str, position: Position) {
        if let Some(entity) = self.entities.get_mut(entity_id) {
            entity.position = position.clone();
            // Update physics body for human entities
            if matches!(entity.entity_type, EntityType::Human) {
                self.physics
                    .update_human_position(entity_id, position.x, position.y, position.z);
            }
        }
    }

    /// Update an entity's rotation.
    pub fn update_entity_rotation(&mut self, entity_id: &str, rotation: Rotation) {
        if let Some(entity) = self.entities.get_mut(entity_id) {
            entity.rotation = rotation;
        }
    }

    /// Get a copy of all entities in the game.
    pub fn get_all_entities(&self) -> Vec<Entity> {
        self.entities.values().cloned().collect()
    }

    /// Step the physics simulation and sync entity positions/rotations from physics.
    ///
    /// This should be called every frame (60 FPS) to update physics simulation.
    /// After stepping physics, entity positions and rotations are updated from
    /// the physics world state.
    ///
    /// # Arguments
    /// * `dt` - Delta time in seconds (typically 1/60.0 for 60 FPS)
    pub fn step_physics(&mut self, dt: f64) {
        // Step physics simulation
        self.physics.step(dt);

        // Sync entity positions and rotations from physics world
        for entity in self.entities.values_mut() {
            // Update position from physics
            if let Some((x, y, z)) = self.physics.get_entity_position(&entity.id) {
                entity.position = Position { x, y, z };
            }
            // Update rotation from physics
            if let Some((rx, ry, rz)) = self.physics.get_entity_rotation(&entity.id) {
                entity.rotation = Rotation {
                    x: rx,
                    y: ry,
                    z: rz,
                };
            }
        }
    }

    /// Convert a player to an entity representation.
    ///
    /// Used when adding a player to create the corresponding physics entity.
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
