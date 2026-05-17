//! Window and asset settings shared by 3D and 2D ship modes.

use crate::cell_box::CellIndex;
use crate::deck_layout::{DeckLayouts, NUM_DECKS};
use crate::ship_hull::{SHIP_BEAM_M, SHIP_LENGTH_M};
use bevy::camera::Viewport;
use bevy::prelude::*;

/// Top fraction of the window reserved for the HUD; the ship plan renders below.
pub const HUD_VIEWPORT_FRACTION: f32 = 0.5;

pub const CLIENT_VERSION: i64 = 128;

pub const DECK_NAMES: [&str; NUM_DECKS] = [
    "Engine Deck",
    "Orlop Deck",
    "Hold Deck",
    "Lower Deck",
    "Second Deck",
    "First Deck",
    "Main Deck",
    "Upper Deck",
    "Promenade Deck",
    "Lido Deck",
    "Boat Deck",
    "Bridge Deck",
    "Sports Deck",
    "Observation Deck",
    "Spa Deck",
    "Pool Deck",
    "Sky Deck",
    "Terrace Deck",
    "Crown Deck",
    "Sun Deck",
];

pub(crate) fn primary_window() -> Window {
    #[cfg(target_arch = "wasm32")]
    {
        Window {
            title: "Ship Game - Time Helm".into(),
            canvas: Some("#ship-game-canvas".into()),
            fit_canvas_to_parent: true,
            ..default()
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        Window {
            title: "Ship Game - Time Helm".into(),
            ..default()
        }
    }
}

pub(crate) fn asset_plugin() -> AssetPlugin {
    #[cfg(target_arch = "wasm32")]
    {
        // Skip HTTP fetches for `.meta` sidecars: static hosting has no meta files, and Bevy
        // would fall back to default meta anyway (`AssetReaderError::NotFound`).
        AssetPlugin {
            meta_check: bevy::asset::AssetMetaCheck::Never,
            ..default()
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        AssetPlugin {
            file_path: "../assets".into(),
            ..default()
        }
    }
}

pub fn game_camera_viewport(window: &Window) -> Viewport {
    let width = window.physical_width().max(1);
    let height = window.physical_height().max(1);
    let hud_h = (height as f32 * HUD_VIEWPORT_FRACTION).round() as u32;
    let view_h = height.saturating_sub(hud_h).max(1);
    Viewport {
        physical_position: UVec2::new(0, hud_h),
        physical_size: UVec2::new(width, view_h),
        ..default()
    }
}

/// Window logical cursor when inside the game camera viewport.
///
/// [`Camera::viewport_to_world_2d`] / [`Camera::viewport_to_world`] expect this absolute
/// position (they subtract [`Camera::logical_viewport_rect`] internally).
pub fn cursor_in_game_viewport(window: &Window, camera: &Camera) -> Option<Vec2> {
    let cursor = window.cursor_position()?;
    let logical = camera.logical_viewport_rect()?;
    logical.contains(cursor).then_some(cursor)
}

/// Hover HUD line from a point in hull plan metres (aft→fore X, starboard→port Y).
pub fn format_cell_hover_line(hull_xy: Vec2, current_deck: usize, layouts: &DeckLayouts) -> String {
    let deck = current_deck as u8;
    let Some(idx) = CellIndex::from_world_xy_deck(hull_xy, deck) else {
        return format!(
            "Hover: cursor ({:.1}, {:.1}) m · outside grid",
            hull_xy.x, hull_xy.y
        );
    };
    let plan = idx.plan();
    let centre = idx.to_world_xy();
    let deck_cells = layouts.deck(current_deck);
    let Some(cell) = deck_cells.get(plan) else {
        return format!(
            "Hover: cell ({}, {}) · centre ({:.1}, {:.1}) m · cursor ({:.1}, {:.1}) m · outside hull",
            plan.0, plan.1, centre.x, centre.y, hull_xy.x, hull_xy.y
        );
    };
    format!(
        "Hover: cell ({}, {}) · centre ({:.1}, {:.1}) m · cursor ({:.1}, {:.1}) m · floor {}",
        plan.0,
        plan.1,
        centre.x,
        centre.y,
        hull_xy.x,
        hull_xy.y,
        cell.floor.label(),
    )
}

pub fn deck_info_text_3d(deck_index: usize) -> String {
    format!(
        "Version {CLIENT_VERSION}\nDeck {}/{}: {} | hull {:.0} m × {:.0} m\nQ/E: orbit | WASD: pan | R/F: vertical | Z/X: zoom | RMB: orbit | MMB: pan | wheel: zoom | PgUp/PgDn: deck\nTile zones (fine LOD): hull edge · bow windows · inner/outer cabins · public aft · shell tint",
        deck_index + 1,
        NUM_DECKS,
        DECK_NAMES[deck_index],
        SHIP_LENGTH_M,
        SHIP_BEAM_M,
    )
}

pub fn deck_info_text_2d(deck_index: usize) -> String {
    format!(
        "Version {CLIENT_VERSION}\nDeck {}/{}: {} | hull {:.0} m × {:.0} m\nWASD: pan · wheel: zoom · PgUp/PgDn: switch deck · E: edit mode · hover ship for cell info",
        deck_index + 1,
        NUM_DECKS,
        DECK_NAMES[deck_index],
        SHIP_LENGTH_M,
        SHIP_BEAM_M,
    )
}
