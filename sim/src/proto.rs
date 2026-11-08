//! Messages shared by the server and the 3D viewer.

use serde::{Deserialize, Serialize};

use crate::command::{authorize, Command, Role, ViewOnly};
use crate::kinds::{Activity, Good, RoomKind};
use crate::world::{PowerReport, World};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct WorldSnapshot {
    pub available: bool,
    pub tick_unix: i64,
    pub clock_utc: String,
    pub decks: Vec<DeckView>,
    pub rooms: Vec<RoomView>,
    pub crew: Vec<CrewView>,
    pub stocks: Vec<StockView>,
    pub totals: CommunityTotals,
    pub power: PowerReport,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct WorldDelta {
    pub tick_unix: i64,
    pub clock_utc: String,
    pub crew: Vec<CrewView>,
    pub stocks: Vec<StockView>,
    pub totals: CommunityTotals,
    pub power: PowerReport,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DeckView {
    pub index: u8,
    pub name: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RoomView {
    pub id: u32,
    pub deck: u8,
    pub kind: RoomKind,
    pub name: String,
    pub polygon: Vec<[f32; 2]>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CrewView {
    pub id: u32,
    pub name: String,
    pub deck: u8,
    pub x: f32,
    pub y: f32,
    pub room_id: u32,
    pub activity: Activity,
    pub hunger: f64,
    pub health: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct StockView {
    pub room_id: u32,
    pub good: Good,
    pub qty: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CommunityTotals {
    pub crew: u32,
    pub meal_kg: f64,
    pub water_l: f64,
    pub edible_kg: f64,
    pub food_days: f64,
    pub water_days: f64,
    pub power_supply_kwh: f64,
    pub power_demand_kwh: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ServerMessage {
    Snapshot(WorldSnapshot),
    Delta(WorldDelta),
    CommandRejected { reason: String },
    Unavailable { reason: String },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClientMessage {
    Command { command: Command },
}

pub fn format_unix_utc(unix: i64) -> String {
    let Some(dt) = chrono::DateTime::from_timestamp(unix, 0) else {
        return format!("unix {unix}");
    };
    dt.format("%Y-%m-%d %H:%M:%S UTC").to_string()
}

impl World {
    pub fn snapshot(&self) -> WorldSnapshot {
        WorldSnapshot {
            available: true,
            tick_unix: self.last_tick_unix,
            clock_utc: format_unix_utc(self.last_tick_unix),
            decks: self
                .decks
                .iter()
                .map(|deck| DeckView {
                    index: deck.index,
                    name: deck.name.clone(),
                })
                .collect(),
            rooms: self
                .rooms
                .iter()
                .map(|room| RoomView {
                    id: room.id,
                    deck: room.deck,
                    kind: room.kind,
                    name: room.name.clone(),
                    polygon: room.polygon.clone(),
                })
                .collect(),
            crew: self.crew_views(),
            stocks: self.stock_views(),
            totals: self.totals(),
            power: self.power.clone(),
        }
    }

    pub fn delta(&self) -> WorldDelta {
        WorldDelta {
            tick_unix: self.last_tick_unix,
            clock_utc: format_unix_utc(self.last_tick_unix),
            crew: self.crew_views(),
            stocks: self.stock_views(),
            totals: self.totals(),
            power: self.power.clone(),
        }
    }

    fn crew_views(&self) -> Vec<CrewView> {
        self.crew
            .iter()
            .map(|crew| CrewView {
                id: crew.id,
                name: crew.name.clone(),
                deck: crew.deck,
                x: crew.x,
                y: crew.y,
                room_id: crew.room_id,
                activity: crew.activity,
                hunger: crew.hunger,
                health: crew.health,
            })
            .collect()
    }

    fn stock_views(&self) -> Vec<StockView> {
        self.stocks
            .iter()
            .map(|stock| StockView {
                room_id: stock.room_id,
                good: stock.good,
                qty: stock.qty,
            })
            .collect()
    }

    fn totals(&self) -> CommunityTotals {
        let meal_kg = self.total(Good::MealKg);
        let water_l = self.total(Good::WaterL);
        let edible_kg = meal_kg
            + self.total(Good::BreadKg)
            + self.total(Good::VegetablesKg)
            + self.total(Good::FishKg);
        let crew_n = self.crew.len().max(1) as f64;
        CommunityTotals {
            crew: self.crew.len() as u32,
            meal_kg,
            water_l,
            edible_kg,
            food_days: edible_kg / (crew_n * 0.6),
            water_days: water_l / (crew_n * 3.0),
            power_supply_kwh: self.power.supply_kwh,
            power_demand_kwh: self.power.demand_kwh,
        }
    }
}

/// Apply a viewer message. Mutations come back as [`ServerMessage::CommandRejected`].
pub fn respond_to_client(role: Role, message: &ClientMessage) -> ServerMessage {
    match message {
        ClientMessage::Command { command } => match authorize(role, command) {
            Err(ViewOnly) => ServerMessage::CommandRejected {
                reason: "viewers cannot change the simulation".to_string(),
            },
            Ok(()) => ServerMessage::CommandRejected {
                reason: "no participant role is enabled".to_string(),
            },
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::command::{Command, Role};

    #[test]
    fn viewer_command_round_trips_and_is_rejected() {
        let message = ClientMessage::Command {
            command: Command::SetStationRecipe {
                station: 1,
                recipe: 0,
            },
        };
        let json = serde_json::to_string(&message).expect("command json");
        assert_eq!(
            json,
            r#"{"type":"command","command":{"type":"set_station_recipe","station":1,"recipe":0}}"#
        );
        let parsed: ClientMessage = serde_json::from_str(&json).expect("parse command");
        assert_eq!(parsed, message);
        assert_eq!(
            respond_to_client(Role::Viewer, &parsed),
            ServerMessage::CommandRejected {
                reason: "viewers cannot change the simulation".to_string(),
            }
        );
    }
}
