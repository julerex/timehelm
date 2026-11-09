//! Server-authoritative ship simulation.
//!
//! The world is a deck of rooms in metres, not a 1 m cell lattice. Time advances
//! one sim second per real second. Callers persist [`World`] and only the server
//! should call [`World::step`].

mod command;
mod geom;
mod hull;
mod kinds;
mod layout;
mod proto;
mod recipes;
mod world;

pub use command::{authorize, Command, Role, ViewOnly};
pub use geom::{polygon_area, polygon_centroid};
pub use hull::{
    deck_hull_polygon, deck_hull_polygon_upper, hull_for_deck, point_in_polygon, SHIP_BEAM_M,
    SHIP_LENGTH_M, UPPER_DECK_START,
};
pub use kinds::{Activity, Department, Good, PowerClass, RoomKind, Skill};
pub use layout::{DECK_COUNT, DECK_HEIGHT_M, DECK_NAMES};
pub use proto::{
    format_unix_utc, respond_to_client, ClientMessage, CommunityTotals, CrewView, RoomView,
    ServerMessage, StockView, WorldDelta, WorldSnapshot,
};
pub use recipes::{recipe, Recipe, RecipeId, ENVIRONMENTAL_SINK, ENVIRONMENTAL_SOURCE};
pub use world::{
    allocate_power, CrewMember, Crop, Deck, Door, Job, MemoryStore, PowerReport, Room, Station,
    Stock, World, STEP_SECONDS, WALK_METRES_PER_SECOND,
};
