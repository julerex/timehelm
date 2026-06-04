//! Build [`DeckLayouts`] from server HTTP API.

use crate::cell::{Cell, FloorMaterial, SideMaterial};
use crate::cell_box::{CellBox, CellIndex};
use crate::deck_layout::{DeckLayouts, DeckMeta, NUM_DECKS};
use crate::protocol::{AllCellsResponse, CellApiRow};

fn parse_wall(name: &str) -> SideMaterial {
    match name {
        "open" => SideMaterial::Open,
        "marine_panel" => SideMaterial::MarinePanel,
        "door" => SideMaterial::Door,
        "window" => SideMaterial::Window,
        _ => SideMaterial::MarinePanel,
    }
}

fn parse_floor(name: &str) -> FloorMaterial {
    match name {
        "wood" => FloorMaterial::Wood,
        _ => FloorMaterial::Carpet,
    }
}

fn cell_from_api(row: &CellApiRow) -> Cell {
    Cell {
        side1: parse_wall(&row.bow_wall),
        side2: parse_wall(&row.port_wall),
        side3: parse_wall(&row.stern_wall),
        side4: parse_wall(&row.starboard_wall),
        floor: parse_floor(&row.floor),
        fixtures: Vec::new(),
    }
}

/// Merge one deck's API cells into a [`CellBox`].
pub fn apply_deck_cells(cell_box: &mut CellBox, cells: &[CellApiRow]) {
    for row in cells {
        if let Some(idx) = CellIndex::new(row.x as u16, row.y as u16, row.z as u8) {
            cell_box.insert(idx, cell_from_api(row));
        }
    }
}

pub fn layouts_from_deck_cells(deck_z: u8, cells: Vec<CellApiRow>) -> DeckLayouts {
    let mut cell_box = CellBox::new();
    apply_deck_cells(&mut cell_box, &cells);
    DeckLayouts {
        cells: cell_box,
        decks: (0..NUM_DECKS).map(|_| DeckMeta {}).collect(),
        entities: Default::default(),
    }
}

pub fn parse_all_cells_response(json: &str) -> Result<AllCellsResponse, serde_json::Error> {
    serde_json::from_str(json)
}
