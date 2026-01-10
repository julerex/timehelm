//! WebSocket message types for client-server communication.
//!
//! All messages use tagged JSON serialization with a "type" field
//! to enable polymorphic message handling.

use crate::game::{Activity, Entity, Player, Position};
use serde::{Deserialize, Serialize};

/// WebSocket message types exchanged between client and server.
///
/// Uses tagged serialization (`#[serde(tag = "type")]`) so messages
/// can be deserialized based on the "type" field.
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum GameMessage {
    /// Client -> Server: Player joining the game
    Join {
        /// Player data for the joining player
        player: Player,
    },
    /// Server -> Client: Player leaving the game
    Leave {
        /// ID of the player who left
        player_id: String,
    },
    /// Client -> Server: Player movement update
    Move {
        /// ID of the moving player
        player_id: String,
        /// New position
        position: Position,
        /// New rotation (Y-axis, radians)
        rotation: f32,
        /// Whether the player is currently moving
        #[serde(default)]
        is_moving: bool,
    },
    /// Client -> Server: Set player activity
    SetActivity {
        /// ID of the player
        player_id: String,
        /// New activity
        activity: Activity,
    },
    /// Server -> Client: Player activity changed
    ActivityChanged {
        /// ID of the player
        player_id: String,
        /// New activity
        activity: Activity,
    },
    /// Server -> Client: Complete world state snapshot
    ///
    /// Sent periodically (10 FPS) to all clients to keep them synchronized.
    WorldState {
        /// All players in the game
        players: Vec<Player>,
        /// All entities in the game
        entities: Vec<Entity>,
    },
    /// Server -> Client: Game time synchronization
    ///
    /// Sent when client connects to sync game time.
    /// Game time is in minutes (Unix seconds, where 1 real second = 1 game minute).
    TimeSync {
        /// Current game time in minutes
        game_time_minutes: i64,
    },
}
