//! Procedural deck grid and tile zoning shared by 3D and 2D ship views (+Y bow).

use crate::ship_hull::{
    deck_tile_centers, deck_tile_centers_upper, FIRST_UPPER_DECK_STYLE_INDEX, SHIP_BEAM_M,
    SHIP_LENGTH_M,
};
use bevy::prelude::*;
use std::collections::HashSet;

/// Number of simulated decks (0-based indices).
pub const NUM_DECKS: usize = 20;

/// Square deck cell size (m). **1.2 m** balances detail vs WASM after mesh batching.
pub const TILE_CELL_M: f32 = 1.2;

/// Tile inset versus cell (matches 3D slab footprint).
pub const TILE_VISUAL_SCALE: f32 = 0.92;

#[derive(Resource, Clone)]
pub struct DeckLayouts(pub Vec<DeckTiles>);

#[derive(Clone)]
pub struct DeckTiles {
    pub centers: Vec<Vec2>,
    pub occupied: HashSet<(i32, i32)>,
}

#[derive(Clone, Copy)]
struct DeckProfile {
    half_beam_scale: f32,
    y_aft: f32,
    y_fwd: f32,
    bow_taper: f32,
    stern_taper: f32,
    courtyard_half_width: f32,
    courtyard_y_aft: f32,
    courtyard_y_fwd: f32,
}

/// Rough zones inspired by the reference floorplan (outer yellow cabins, inner pink block).
fn outer_cabin_zone(p: Vec2) -> bool {
    p.x.abs() > SHIP_BEAM_M * 0.32 && p.y > -SHIP_LENGTH_M * 0.36 && p.y < SHIP_LENGTH_M * 0.26
}

fn inner_cabin_zone(p: Vec2) -> bool {
    p.x.abs() < SHIP_BEAM_M * 0.24 && p.y > -SHIP_LENGTH_M * 0.32 && p.y < SHIP_LENGTH_M * 0.22
}

fn window_strip_zone(p: Vec2) -> bool {
    p.y > SHIP_LENGTH_M * 0.12 && p.x.abs() > SHIP_BEAM_M * 0.34
}

fn deck_profile(deck_index: usize) -> DeckProfile {
    // Hand-authored deck-by-deck silhouettes inspired by the reference plans.
    const P: [DeckProfile; NUM_DECKS] = [
        DeckProfile {
            half_beam_scale: 0.42,
            y_aft: -SHIP_LENGTH_M * 0.43,
            y_fwd: SHIP_LENGTH_M * 0.14,
            bow_taper: 0.55,
            stern_taper: 0.35,
            courtyard_half_width: 0.0,
            courtyard_y_aft: 0.0,
            courtyard_y_fwd: 0.0,
        }, // 1
        DeckProfile {
            half_beam_scale: 0.52,
            y_aft: -SHIP_LENGTH_M * 0.45,
            y_fwd: SHIP_LENGTH_M * 0.20,
            bow_taper: 0.48,
            stern_taper: 0.30,
            courtyard_half_width: 0.0,
            courtyard_y_aft: 0.0,
            courtyard_y_fwd: 0.0,
        }, // 2
        DeckProfile {
            half_beam_scale: 0.70,
            y_aft: -SHIP_LENGTH_M * 0.47,
            y_fwd: SHIP_LENGTH_M * 0.26,
            bow_taper: 0.44,
            stern_taper: 0.25,
            courtyard_half_width: 0.0,
            courtyard_y_aft: 0.0,
            courtyard_y_fwd: 0.0,
        }, // 3
        DeckProfile {
            half_beam_scale: 0.88,
            y_aft: -SHIP_LENGTH_M * 0.49,
            y_fwd: SHIP_LENGTH_M * 0.38,
            bow_taper: 0.32,
            stern_taper: 0.18,
            courtyard_half_width: 0.0,
            courtyard_y_aft: 0.0,
            courtyard_y_fwd: 0.0,
        }, // 4
        DeckProfile {
            half_beam_scale: 0.96,
            y_aft: -SHIP_LENGTH_M * 0.50,
            y_fwd: SHIP_LENGTH_M * 0.44,
            bow_taper: 0.26,
            stern_taper: 0.12,
            courtyard_half_width: 0.0,
            courtyard_y_aft: 0.0,
            courtyard_y_fwd: 0.0,
        }, // 5
        DeckProfile {
            half_beam_scale: 0.98,
            y_aft: -SHIP_LENGTH_M * 0.50,
            y_fwd: SHIP_LENGTH_M * 0.46,
            bow_taper: 0.24,
            stern_taper: 0.11,
            courtyard_half_width: 0.0,
            courtyard_y_aft: 0.0,
            courtyard_y_fwd: 0.0,
        }, // 6
        DeckProfile {
            half_beam_scale: 0.98,
            y_aft: -SHIP_LENGTH_M * 0.50,
            y_fwd: SHIP_LENGTH_M * 0.47,
            bow_taper: 0.23,
            stern_taper: 0.10,
            courtyard_half_width: 0.0,
            courtyard_y_aft: 0.0,
            courtyard_y_fwd: 0.0,
        }, // 7
        DeckProfile {
            half_beam_scale: 0.97,
            y_aft: -SHIP_LENGTH_M * 0.50,
            y_fwd: SHIP_LENGTH_M * 0.47,
            bow_taper: 0.23,
            stern_taper: 0.10,
            courtyard_half_width: 0.0,
            courtyard_y_aft: 0.0,
            courtyard_y_fwd: 0.0,
        }, // 8
        DeckProfile {
            half_beam_scale: 0.96,
            y_aft: -SHIP_LENGTH_M * 0.50,
            y_fwd: SHIP_LENGTH_M * 0.47,
            bow_taper: 0.24,
            stern_taper: 0.10,
            courtyard_half_width: 0.0,
            courtyard_y_aft: 0.0,
            courtyard_y_fwd: 0.0,
        }, // 9
        DeckProfile {
            half_beam_scale: 0.94,
            y_aft: -SHIP_LENGTH_M * 0.49,
            y_fwd: SHIP_LENGTH_M * 0.46,
            bow_taper: 0.25,
            stern_taper: 0.16,
            courtyard_half_width: 9.0,
            courtyard_y_aft: -SHIP_LENGTH_M * 0.26,
            courtyard_y_fwd: SHIP_LENGTH_M * 0.21,
        }, // 10
        DeckProfile {
            half_beam_scale: 0.93,
            y_aft: -SHIP_LENGTH_M * 0.49,
            y_fwd: SHIP_LENGTH_M * 0.45,
            bow_taper: 0.26,
            stern_taper: 0.17,
            courtyard_half_width: 9.5,
            courtyard_y_aft: -SHIP_LENGTH_M * 0.26,
            courtyard_y_fwd: SHIP_LENGTH_M * 0.21,
        }, // 11
        DeckProfile {
            half_beam_scale: 0.92,
            y_aft: -SHIP_LENGTH_M * 0.48,
            y_fwd: SHIP_LENGTH_M * 0.44,
            bow_taper: 0.27,
            stern_taper: 0.18,
            courtyard_half_width: 10.0,
            courtyard_y_aft: -SHIP_LENGTH_M * 0.25,
            courtyard_y_fwd: SHIP_LENGTH_M * 0.20,
        }, // 12
        DeckProfile {
            half_beam_scale: 0.89,
            y_aft: -SHIP_LENGTH_M * 0.47,
            y_fwd: SHIP_LENGTH_M * 0.42,
            bow_taper: 0.29,
            stern_taper: 0.20,
            courtyard_half_width: 9.2,
            courtyard_y_aft: -SHIP_LENGTH_M * 0.23,
            courtyard_y_fwd: SHIP_LENGTH_M * 0.18,
        }, // 13
        DeckProfile {
            half_beam_scale: 0.86,
            y_aft: -SHIP_LENGTH_M * 0.46,
            y_fwd: SHIP_LENGTH_M * 0.40,
            bow_taper: 0.31,
            stern_taper: 0.22,
            courtyard_half_width: 8.2,
            courtyard_y_aft: -SHIP_LENGTH_M * 0.20,
            courtyard_y_fwd: SHIP_LENGTH_M * 0.16,
        }, // 14
        DeckProfile {
            half_beam_scale: 0.82,
            y_aft: -SHIP_LENGTH_M * 0.45,
            y_fwd: SHIP_LENGTH_M * 0.38,
            bow_taper: 0.32,
            stern_taper: 0.23,
            courtyard_half_width: 7.4,
            courtyard_y_aft: -SHIP_LENGTH_M * 0.18,
            courtyard_y_fwd: SHIP_LENGTH_M * 0.14,
        }, // 15
        DeckProfile {
            half_beam_scale: 0.78,
            y_aft: -SHIP_LENGTH_M * 0.43,
            y_fwd: SHIP_LENGTH_M * 0.36,
            bow_taper: 0.34,
            stern_taper: 0.24,
            courtyard_half_width: 6.2,
            courtyard_y_aft: -SHIP_LENGTH_M * 0.15,
            courtyard_y_fwd: SHIP_LENGTH_M * 0.12,
        }, // 16
        DeckProfile {
            half_beam_scale: 0.73,
            y_aft: -SHIP_LENGTH_M * 0.40,
            y_fwd: SHIP_LENGTH_M * 0.34,
            bow_taper: 0.35,
            stern_taper: 0.25,
            courtyard_half_width: 4.8,
            courtyard_y_aft: -SHIP_LENGTH_M * 0.13,
            courtyard_y_fwd: SHIP_LENGTH_M * 0.10,
        }, // 17
        DeckProfile {
            half_beam_scale: 0.68,
            y_aft: -SHIP_LENGTH_M * 0.37,
            y_fwd: SHIP_LENGTH_M * 0.31,
            bow_taper: 0.37,
            stern_taper: 0.27,
            courtyard_half_width: 0.0,
            courtyard_y_aft: 0.0,
            courtyard_y_fwd: 0.0,
        }, // 18
        DeckProfile {
            half_beam_scale: 0.63,
            y_aft: -SHIP_LENGTH_M * 0.34,
            y_fwd: SHIP_LENGTH_M * 0.28,
            bow_taper: 0.40,
            stern_taper: 0.30,
            courtyard_half_width: 0.0,
            courtyard_y_aft: 0.0,
            courtyard_y_fwd: 0.0,
        }, // 19
        DeckProfile {
            half_beam_scale: 0.56,
            y_aft: -SHIP_LENGTH_M * 0.30,
            y_fwd: SHIP_LENGTH_M * 0.24,
            bow_taper: 0.44,
            stern_taper: 0.34,
            courtyard_half_width: 0.0,
            courtyard_y_aft: 0.0,
            courtyard_y_fwd: 0.0,
        }, // 20
    ];
    P[deck_index.min(NUM_DECKS - 1)]
}

fn profile_allows_tile(deck_index: usize, p: Vec2) -> bool {
    let profile = deck_profile(deck_index);
    if p.y < profile.y_aft || p.y > profile.y_fwd {
        return false;
    }

    let fwd_span = (SHIP_LENGTH_M * 0.5 - profile.y_fwd).max(1.0);
    let aft_span = (profile.y_aft + SHIP_LENGTH_M * 0.5).max(1.0);
    let fwd_t = ((p.y - profile.y_fwd) / fwd_span).clamp(0.0, 1.0);
    let aft_t = ((profile.y_aft - p.y) / aft_span).clamp(0.0, 1.0);
    let taper = 1.0 - profile.bow_taper * fwd_t * fwd_t - profile.stern_taper * aft_t * aft_t;
    let beam_limit = SHIP_BEAM_M * 0.5 * profile.half_beam_scale * taper.max(0.2);
    if p.x.abs() > beam_limit {
        return false;
    }

    if profile.courtyard_half_width > 0.0
        && p.y > profile.courtyard_y_aft
        && p.y < profile.courtyard_y_fwd
        && p.x.abs() < profile.courtyard_half_width
    {
        return false;
    }

    // Upper leisure decks: emulate split stern terraces and side tapering.
    if deck_index >= 17 && p.y < -SHIP_LENGTH_M * 0.20 && p.x.abs() < SHIP_BEAM_M * 0.14 {
        return false;
    }
    if deck_index >= 18 && p.y > SHIP_LENGTH_M * 0.15 && p.x.abs() > SHIP_BEAM_M * 0.22 {
        return false;
    }

    true
}

fn fallback_deck_tiles(deck_index: usize, step_m: f32) -> DeckTiles {
    let centers = if deck_index >= FIRST_UPPER_DECK_STYLE_INDEX {
        deck_tile_centers_upper(step_m)
    } else {
        deck_tile_centers(step_m)
    };
    let occupied = centers
        .iter()
        .map(|c| ((c.x / step_m).round() as i32, (c.y / step_m).round() as i32))
        .collect::<HashSet<_>>();
    DeckTiles { centers, occupied }
}

/// All deck occupancy grids at `step_m` cell spacing.
pub fn deck_layouts(step_m: f32) -> Vec<DeckTiles> {
    let mut out = Vec::with_capacity(NUM_DECKS);
    for deck_i in 0..NUM_DECKS {
        let base = fallback_deck_tiles(deck_i, step_m);
        let centers = base
            .centers
            .into_iter()
            .filter(|p| profile_allows_tile(deck_i, *p))
            .collect::<Vec<_>>();
        let occupied = centers
            .iter()
            .map(|c| ((c.x / step_m).round() as i32, (c.y / step_m).round() as i32))
            .collect::<HashSet<_>>();
        out.push(DeckTiles { centers, occupied });
    }
    out
}

fn is_perimeter_cell(cell: (i32, i32), occupied: &HashSet<(i32, i32)>) -> bool {
    for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
        if !occupied.contains(&(cell.0 + dx, cell.1 + dy)) {
            return true;
        }
    }
    false
}

#[derive(Clone, Copy)]
pub enum DeckTileBucket {
    HullEdge,
    WindowStrip,
    InnerCabin,
    OuterCabin,
    PublicDeck,
    DeckBase,
}

impl DeckTileBucket {
    pub const COUNT: usize = 6;

    pub fn idx(self) -> usize {
        match self {
            DeckTileBucket::HullEdge => 0,
            DeckTileBucket::WindowStrip => 1,
            DeckTileBucket::InnerCabin => 2,
            DeckTileBucket::OuterCabin => 3,
            DeckTileBucket::PublicDeck => 4,
            DeckTileBucket::DeckBase => 5,
        }
    }

    /// Mirrors the classification order in legacy `setup` (per-tile spawn).
    pub fn classify(c: Vec2, cell: (i32, i32), occupied: &HashSet<(i32, i32)>) -> Self {
        let edge = is_perimeter_cell(cell, occupied);
        if edge {
            return if window_strip_zone(c) {
                DeckTileBucket::WindowStrip
            } else {
                DeckTileBucket::HullEdge
            };
        }
        if inner_cabin_zone(c) {
            return DeckTileBucket::InnerCabin;
        }
        if outer_cabin_zone(c) {
            return DeckTileBucket::OuterCabin;
        }
        if c.y < -SHIP_LENGTH_M * 0.28 {
            return DeckTileBucket::PublicDeck;
        }
        DeckTileBucket::DeckBase
    }
}

/// Optional axis-aligned amenity tint on top of base bucket colours (restaurant, pool, theatre, …).
pub fn amenity_overlay(deck_index: usize, p: Vec2) -> Option<Color> {
    // Forward theatre / aqua show — decks 5–10, centreline-forward
    if (5..=10).contains(&deck_index)
        && p.x.abs() < 17.5
        && p.y > SHIP_LENGTH_M * 0.36
        && p.y < SHIP_LENGTH_M * 0.49
    {
        return Some(Color::srgb(0.52, 0.36, 0.68));
    }
    // Main dining room — mid-aft hull
    if (4..=7).contains(&deck_index)
        && p.x.abs() < 17.5
        && p.y > -SHIP_LENGTH_M * 0.36
        && p.y < -SHIP_LENGTH_M * 0.10
    {
        return Some(Color::srgb(0.82, 0.48, 0.34));
    }
    // Buffet / speciality restaurant strip — upper lido bays
    if (8..=12).contains(&deck_index)
        && p.x.abs() > 9.0
        && p.x.abs() < 27.5
        && p.y > SHIP_LENGTH_M * 0.04
        && p.y < SHIP_LENGTH_M * 0.30
    {
        return Some(Color::srgb(0.42, 0.70, 0.50));
    }
    // Pool / sports — open forward-upper
    if (11..=16).contains(&deck_index)
        && p.y > SHIP_LENGTH_M * 0.16
        && p.x.abs() < SHIP_BEAM_M * 0.36
    {
        return Some(Color::srgb(0.32, 0.74, 0.88));
    }
    // Casino / nightclub — outboard promenade band
    if (6..=9).contains(&deck_index)
        && p.y > -SHIP_LENGTH_M * 0.06
        && p.y < SHIP_LENGTH_M * 0.14
        && p.x.abs() > 19.0
    {
        return Some(Color::srgb(0.72, 0.22, 0.42));
    }
    None
}
