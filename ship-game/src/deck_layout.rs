//! Procedural deck grid and cell zoning shared by 3D and 2D ship views (+Y bow).

use crate::cell::{Cell, Material, RoomCatalog, RoomCategory, RoomId};
use crate::cell_box::{CellBox, CellIndex, PlanKey};
use crate::ship_hull::{
    deck_cell_centers, deck_cell_centers_upper, deck_hull_polygon_upper,
    FIRST_UPPER_DECK_STYLE_INDEX, SHIP_BEAM_M, SHIP_LENGTH_M,
};
use bevy::prelude::*;
use std::collections::{HashMap, HashSet};

/// Number of simulated decks (0-based indices).
pub const NUM_DECKS: usize = 20;

/// Square deck cell size (m). **1 m** grid spacing in plan view and 3D deck slabs.
pub const CELL_SIZE_M: f32 = 1.0;

/// Cell inset versus grid spacing (matches 3D slab footprint).
pub const CELL_VISUAL_SCALE: f32 = 1.0;

#[derive(Resource, Clone)]
pub struct DeckLayouts {
    pub cells: CellBox,
    pub decks: Vec<DeckMeta>,
}

#[derive(Clone)]
pub struct DeckMeta {
    pub rooms: RoomCatalog,
}

/// Per-deck metadata and cell access (backed by [`DeckLayouts::cells`]).
#[derive(Clone, Copy)]
pub struct DeckCells<'a> {
    pub deck: u8,
    pub cells: &'a CellBox,
    pub rooms: &'a RoomCatalog,
}

impl DeckLayouts {
    #[must_use]
    pub fn deck(&self, deck_index: usize) -> DeckCells<'_> {
        DeckCells {
            deck: deck_index as u8,
            cells: &self.cells,
            rooms: &self.decks[deck_index].rooms,
        }
    }

    pub fn cell_mut(&mut self, index: CellIndex) -> Option<&mut Cell> {
        self.cells.get_mut(index)
    }
}

impl<'a> DeckCells<'a> {
    pub fn centers(&self) -> Vec<Vec2> {
        self.cells
            .iter_deck(self.deck)
            .map(|(idx, _)| idx.to_world_xy())
            .collect()
    }

    pub fn perimeter(&self) -> HashSet<PlanKey> {
        let occupied: HashSet<_> = self.plan_keys().collect();
        occupied
            .iter()
            .copied()
            .filter(|cell| is_perimeter_cell(*cell, &occupied))
            .collect()
    }

    pub fn cell_coords(p: Vec2) -> Option<PlanKey> {
        CellIndex::from_world_xy_deck(p, 0).map(CellIndex::plan)
    }

    pub fn cell_coords_deck(p: Vec2, deck: u8) -> Option<PlanKey> {
        CellIndex::from_world_xy_deck(p, deck).map(CellIndex::plan)
    }

    pub fn index(&self, plan: PlanKey) -> CellIndex {
        CellIndex {
            x: plan.0,
            y: plan.1,
            z: self.deck,
        }
    }

    pub fn get(&self, plan: PlanKey) -> Option<&Cell> {
        self.cells.get(self.index(plan))
    }

    pub fn plan_keys(&self) -> impl Iterator<Item = PlanKey> + 'a {
        self.cells.iter_deck(self.deck).map(|(idx, _)| idx.plan())
    }

    pub fn iter_cells(&self) -> impl Iterator<Item = (PlanKey, &'a Cell)> + 'a {
        self.cells
            .iter_deck(self.deck)
            .map(|(idx, cell)| (idx.plan(), cell))
    }
}

fn plan_key_from_world(p: Vec2, deck: u8) -> Option<PlanKey> {
    CellIndex::from_world_xy_deck(p, deck).map(CellIndex::plan)
}

fn neighbor_plan(plan: PlanKey, wall_idx: usize) -> Option<PlanKey> {
    let idx = CellIndex {
        x: plan.0,
        y: plan.1,
        z: 0,
    };
    idx.neighbor(wall_idx).map(CellIndex::plan)
}

/// One deck while procedural layout runs; merged into [`CellBox`] when complete.
struct DeckBuild {
    cells: HashMap<PlanKey, Cell>,
    rooms: RoomCatalog,
}

impl DeckBuild {
    fn plan_world(plan: PlanKey, deck: u8) -> Vec2 {
        CellIndex::with_plan(deck, plan)
            .expect("plan in range")
            .to_world_xy()
    }
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

/// Half-width (positive starboard extent) allowed by the deck profile at `y`, before courtyard / carve rules.
fn sim_half_beam_limit(deck_index: usize, y: f32) -> Option<f32> {
    let profile = deck_profile(deck_index);
    if y < profile.y_aft || y > profile.y_fwd {
        return None;
    }

    let fwd_span = (SHIP_LENGTH_M * 0.5 - profile.y_fwd).max(1.0);
    let aft_span = (profile.y_aft + SHIP_LENGTH_M * 0.5).max(1.0);
    let fwd_t = ((y - profile.y_fwd) / fwd_span).clamp(0.0, 1.0);
    let aft_t = ((profile.y_aft - y) / aft_span).clamp(0.0, 1.0);
    let taper = 1.0 - profile.bow_taper * fwd_t * fwd_t - profile.stern_taper * aft_t * aft_t;
    Some(SHIP_BEAM_M * 0.5 * profile.half_beam_scale * taper.max(0.2))
}

fn profile_allows_cell(deck_index: usize, p: Vec2) -> bool {
    let Some(beam_limit) = sim_half_beam_limit(deck_index, p.y) else {
        return false;
    };
    if p.x.abs() > beam_limit {
        return false;
    }

    let profile = deck_profile(deck_index);
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

fn deck_lower_profile_outline(deck_index: usize) -> Vec<Vec2> {
    const STEPS: usize = 72;
    let profile = deck_profile(deck_index);
    let y_aft = profile.y_aft;
    let y_fwd = profile.y_fwd;
    let hb_fwd = sim_half_beam_limit(deck_index, y_fwd).unwrap_or(0.0);
    let hb_aft = sim_half_beam_limit(deck_index, y_aft).unwrap_or(0.0);

    let mut poly = Vec::with_capacity(STEPS * 2 + 4);
    poly.push(Vec2::new(hb_fwd, y_fwd));
    for i in 1..STEPS - 1 {
        let t = i as f32 / (STEPS - 1) as f32;
        let y = y_fwd + (y_aft - y_fwd) * t;
        if let Some(hb) = sim_half_beam_limit(deck_index, y) {
            poly.push(Vec2::new(hb, y));
        }
    }
    poly.push(Vec2::new(hb_aft, y_aft));
    poly.push(Vec2::new(-hb_aft, y_aft));
    for i in (1..STEPS - 1).rev() {
        let t = i as f32 / (STEPS - 1) as f32;
        let y = y_fwd + (y_aft - y_fwd) * t;
        if let Some(hb) = sim_half_beam_limit(deck_index, y) {
            poly.push(Vec2::new(-hb, y));
        }
    }
    poly.push(Vec2::new(-hb_fwd, y_fwd));
    poly
}

/// Closed deck boundary matching the **simulated cell footprint** (profile clipping). Upper decks use the
/// courtyard hull polygon so coarse LOD / 2D fills align with fine cells instead of the full reference hull.
pub fn deck_sim_footprint_polygon(deck_index: usize) -> Vec<Vec2> {
    if deck_index >= FIRST_UPPER_DECK_STYLE_INDEX {
        deck_hull_polygon_upper()
    } else {
        deck_lower_profile_outline(deck_index)
    }
}

fn fallback_deck_cell_centers(deck_index: usize, step_m: f32) -> Vec<Vec2> {
    if deck_index >= FIRST_UPPER_DECK_STYLE_INDEX {
        deck_cell_centers_upper(step_m)
    } else {
        deck_cell_centers(step_m)
    }
}

const CABIN_WIDTH_CELLS: i32 = 3;
const CABIN_LENGTH_CELLS: i32 = 6;
const CABIN_COLUMN_WIDTH_M: f32 = 3.0;
const NUM_CABIN_COLUMNS: usize = 6;
const NUM_INTERIOR_SPINES: usize = 3;
const CABIN_COLUMNS_TOTAL_WIDTH_M: f32 = CABIN_COLUMN_WIDTH_M * NUM_CABIN_COLUMNS as f32;

fn cabin_room_key(plan: PlanKey) -> (i32, i32) {
    (
        (plan.1 as i32).div_euclid(CABIN_WIDTH_CELLS),
        (plan.0 as i32).div_euclid(CABIN_LENGTH_CELLS),
    )
}

fn cabin_local(plan: PlanKey) -> (i32, i32) {
    (
        (plan.1 as i32).rem_euclid(CABIN_WIDTH_CELLS),
        (plan.0 as i32).rem_euclid(CABIN_LENGTH_CELLS),
    )
}

fn is_cabin_interior(lx: i32, ly: i32) -> bool {
    lx == 1 && (1..=4).contains(&ly)
}

fn effective_beam_m(deck_index: usize, y: f32) -> Option<f32> {
    sim_half_beam_limit(deck_index, y).map(|hb| hb * 2.0)
}

fn spine_width_m(effective_beam_m: f32) -> f32 {
    ((effective_beam_m - CABIN_COLUMNS_TOTAL_WIDTH_M) / NUM_INTERIOR_SPINES as f32).max(1.0)
}

/// Seven bands port→stbd: cabin, spine, cabin, spine, cabin, spine, cabin (metres, signed `x`).
fn cross_section_bands(effective_beam_m: f32) -> [(f32, f32); 7] {
    let spine_w = spine_width_m(effective_beam_m);
    let total = CABIN_COLUMNS_TOTAL_WIDTH_M + NUM_INTERIOR_SPINES as f32 * spine_w;
    let x0 = -total * 0.5;
    let widths = [
        CABIN_COLUMN_WIDTH_M,
        spine_w,
        CABIN_COLUMN_WIDTH_M,
        spine_w,
        CABIN_COLUMN_WIDTH_M,
        spine_w,
        CABIN_COLUMN_WIDTH_M,
    ];
    let mut bands = [(0.0, 0.0); 7];
    let mut x = x0;
    for (band, &w) in bands.iter_mut().zip(widths.iter()) {
        *band = (x, x + w);
        x += w;
    }
    bands
}

fn band_index_at_x(x: f32, bands: &[(f32, f32); 7]) -> Option<usize> {
    if x < bands[0].0 {
        return Some(0);
    }
    if x >= bands[6].1 {
        return Some(6);
    }
    for (i, &(lo, hi)) in bands.iter().enumerate() {
        if i == bands.len() - 1 {
            if x >= lo && x <= hi {
                return Some(i);
            }
        } else if x >= lo && x < hi {
            return Some(i);
        }
    }
    None
}

fn is_spine_band(band_index: usize) -> bool {
    band_index % 2 == 1
}

fn module_x_span(module_cells: &[PlanKey]) -> (u16, u16) {
    let min_x = module_cells.iter().map(|p| p.0).min().unwrap();
    let max_x = module_cells.iter().map(|p| p.0).max().unwrap();
    (min_x, max_x)
}

fn is_module_fore_aft_door_wall(wall_idx: usize, _lx: i32, x: u16, min_x: u16, max_x: u16) -> bool {
    if wall_idx != 1 && wall_idx != 3 {
        return false;
    }
    (wall_idx == 3 && x == min_x) || (wall_idx == 1 && x == max_x)
}

/// Inboard direction (±1 in `ix`) from a cabin column band toward the nearest interior spine.
fn cabin_band_inboard_x(band_index: usize) -> i32 {
    match band_index {
        0 | 2 => 1,
        4 | 6 => -1,
        _ => 0,
    }
}

fn room_category_at(
    catalog: &RoomCatalog,
    room_map: &HashMap<PlanKey, RoomId>,
    coord: PlanKey,
) -> Option<RoomCategory> {
    room_map.get(&coord).and_then(|&id| catalog.category(id))
}

fn cabin_edge_wall(
    occupied: &HashSet<PlanKey>,
    from_room: RoomId,
    to: PlanKey,
    room_map: &HashMap<PlanKey, RoomId>,
    catalog: &RoomCatalog,
) -> Material {
    if !occupied.contains(&to) {
        return Material::Hull;
    }
    match room_map.get(&to) {
        Some(&r) if r == from_room => Material::Open,
        Some(&r) if catalog.category(r) == Some(RoomCategory::Corridor) => Material::MarinePanel,
        Some(_) => Material::MarinePanel,
        None => Material::Open,
    }
}

/// Longitudinal corridor spines from the six-column / three-spine cross-section (all decks).
fn build_cabin_spine_layout(centers: &[Vec2], deck_index: usize) -> HashSet<PlanKey> {
    let deck = deck_index as u8;
    let mut corridor = HashSet::new();
    for &p in centers {
        let Some(beam) = effective_beam_m(deck_index, p.y) else {
            continue;
        };
        let bands = cross_section_bands(beam);
        let Some(bi) = band_index_at_x(p.x, &bands) else {
            continue;
        };
        if is_spine_band(bi) {
            if let Some(plan) = plan_key_from_world(p, deck) {
                corridor.insert(plan);
            }
        }
    }
    corridor
}

fn link_stub_to_spine(
    corridor: &mut HashSet<PlanKey>,
    occupied: &HashSet<PlanKey>,
    stub: PlanKey,
    inboard_dy: i32,
    deck_index: usize,
) {
    let deck = deck_index as u8;
    let y_m = DeckBuild::plan_world(stub, deck).y;
    let Some(beam) = effective_beam_m(deck_index, y_m) else {
        corridor.insert(stub);
        return;
    };
    let bands = cross_section_bands(beam);
    let mut y = stub.1 as i32;
    corridor.insert(stub);
    loop {
        let next_y = y + inboard_dy;
        if next_y < 0 || next_y >= crate::cell_box::BEAM as i32 {
            break;
        }
        let next = (stub.0, next_y as u16);
        let x_m = DeckBuild::plan_world(next, deck).x;
        let Some(bi) = band_index_at_x(x_m, &bands) else {
            break;
        };
        if !occupied.contains(&next) {
            break;
        }
        corridor.insert(next);
        if is_spine_band(bi) {
            break;
        }
        y = next_y;
    }
}

fn inboard_column_lx(band_index: usize) -> i32 {
    match band_index {
        0 | 2 => CABIN_WIDTH_CELLS - 1,
        4 | 6 => 0,
        _ => 1,
    }
}

/// Fore/aft door stubs (full 3 m row) plus inboard links from each cabin module to its nearest spine.
fn extend_corridor_door_connectors(
    corridor: &mut HashSet<PlanKey>,
    occupied: &HashSet<PlanKey>,
    deck_index: usize,
) {
    let deck = deck_index as u8;
    let mut module_anchors: HashSet<PlanKey> = HashSet::new();
    for &coord in occupied.iter() {
        if corridor.contains(&coord) {
            continue;
        }
        let ox = (coord.0 as i32).div_euclid(CABIN_LENGTH_CELLS) as u16 * CABIN_LENGTH_CELLS as u16;
        let oy = (coord.1 as i32).div_euclid(CABIN_WIDTH_CELLS) as u16 * CABIN_WIDTH_CELLS as u16;
        module_anchors.insert((ox, oy));
    }

    for (ox, oy) in module_anchors {
        let anchor = (ox, oy);
        let mut module_cells: Vec<PlanKey> = Vec::new();
        for &coord in occupied.iter() {
            if corridor.contains(&coord) {
                continue;
            }
            if cabin_room_key(coord) == cabin_room_key(anchor) {
                module_cells.push(coord);
            }
        }
        if module_cells.is_empty() {
            continue;
        }
        let (min_x, max_x) = module_x_span(&module_cells);
        let y_m = DeckBuild::plan_world(((min_x + max_x) / 2, oy), deck).y;
        let Some(beam) = effective_beam_m(deck_index, y_m) else {
            continue;
        };
        let bands = cross_section_bands(beam);
        let across_y = oy.saturating_add(1);
        let x_m = DeckBuild::plan_world((ox, across_y), deck).x;
        let Some(band) = band_index_at_x(x_m, &bands) else {
            continue;
        };
        if is_spine_band(band) {
            continue;
        }
        let inboard_dy = cabin_band_inboard_x(band);
        if inboard_dy == 0 {
            continue;
        }
        let inboard_ly = inboard_column_lx(band);

        let door_specs: [(u16, usize); 2] = [(min_x, 3), (max_x, 1)];
        let mut ends = door_specs.to_vec();
        let module_mid_x = (min_x + max_x) / 2;
        ends.sort_by_key(|(x, _)| x.abs_diff(module_mid_x));

        let mut linked = false;
        for (end_x, door_wall) in ends {
            let door_cell = (end_x, across_y);
            if !occupied.contains(&door_cell) {
                continue;
            }
            let Some(stub) = neighbor_plan(door_cell, door_wall) else {
                continue;
            };
            if !occupied.contains(&stub) {
                continue;
            }
            corridor.insert(stub);
            let link_y = oy.saturating_add(inboard_ly as u16);
            let link_stub = (stub.0, link_y);
            if occupied.contains(&link_stub) {
                link_stub_to_spine(corridor, occupied, link_stub, inboard_dy, deck_index);
                linked = true;
            }
            break;
        }

        if linked {
            continue;
        }

        for &(carve_x, _) in &[(min_x, 3usize), (max_x, 1usize)] {
            let mut row_ok = true;
            for ly in 0..CABIN_WIDTH_CELLS {
                let cell = (carve_x, oy.saturating_add(ly as u16));
                if !occupied.contains(&cell) {
                    row_ok = false;
                    break;
                }
            }
            if !row_ok {
                continue;
            }
            for ly in 0..CABIN_WIDTH_CELLS {
                corridor.insert((carve_x, oy.saturating_add(ly as u16)));
            }
            let link_stub = (carve_x, oy.saturating_add(inboard_ly as u16));
            link_stub_to_spine(corridor, occupied, link_stub, inboard_dy, deck_index);
            break;
        }
    }
}

/// Ensure every cabin cell has a corridor neighbor (inboard toward spine, or fore/aft stub).
fn ensure_cabin_corridor_touch(
    corridor: &mut HashSet<PlanKey>,
    occupied: &HashSet<PlanKey>,
    deck_index: usize,
) {
    for _ in 0..4 {
        let cabin_coords: Vec<_> = occupied
            .iter()
            .copied()
            .filter(|c| !corridor.contains(c))
            .collect();

        let mut changed = false;
        for coord in cabin_coords {
            let has_corridor =
                (0..4).any(|wi| neighbor_plan(coord, wi).is_some_and(|nb| corridor.contains(&nb)));
            if has_corridor {
                continue;
            }

            let world = DeckBuild::plan_world(coord, deck_index as u8);
            let Some(beam) = effective_beam_m(deck_index, world.y) else {
                continue;
            };
            let bands = cross_section_bands(beam);
            let band = band_index_at_x(world.x, &bands);
            if band.is_some_and(is_spine_band) {
                continue;
            }

            let inboard_dy = band.map(cabin_band_inboard_x).unwrap_or(1);
            if inboard_dy == 0 {
                continue;
            }
            let inboard_y = coord.1 as i32 + inboard_dy;
            if (0..crate::cell_box::BEAM as i32).contains(&inboard_y) {
                let inboard = (coord.0, inboard_y as u16);
                if occupied.contains(&inboard) {
                    changed |= corridor.insert(inboard);
                }
            }

            let fore_aft_has_corridor = [3, 1]
                .iter()
                .any(|&wi| neighbor_plan(coord, wi).is_some_and(|nb| corridor.contains(&nb)));
            if fore_aft_has_corridor {
                continue;
            }

            for wi in [3usize, 1] {
                if let Some(nb) = neighbor_plan(coord, wi) {
                    if occupied.contains(&nb) {
                        changed |= corridor.insert(nb);
                    }
                }
            }
        }
        if !changed {
            break;
        }
    }
}

fn corridor_floor_material() -> Material {
    Material::Corridor
}

fn room_for_cabin(
    catalog: &mut RoomCatalog,
    cabin_rooms: &mut HashMap<(i32, i32), RoomId>,
    deck: u8,
    plan: PlanKey,
) -> RoomId {
    *cabin_rooms
        .entry(cabin_room_key(plan))
        .or_insert_with(|| catalog.insert("Cabin", deck, RoomCategory::Cabin))
}

fn edge_wall(
    occupied: &HashSet<PlanKey>,
    from_room: RoomId,
    plan: PlanKey,
    wall_idx: usize,
    room_map: &HashMap<PlanKey, RoomId>,
    rooms: &RoomCatalog,
) -> Material {
    match neighbor_plan(plan, wall_idx) {
        Some(nb) => cabin_edge_wall(occupied, from_room, nb, room_map, rooms),
        None => Material::Hull,
    }
}

fn assign_non_cabin_walls(deck: &mut DeckBuild) {
    let occupied: HashSet<_> = deck.cells.keys().copied().collect();
    let room_map: HashMap<PlanKey, RoomId> =
        deck.cells.iter().map(|(&c, cell)| (c, cell.room)).collect();
    let coords: Vec<PlanKey> = deck.cells.keys().copied().collect();
    for plan in coords {
        let from_room = room_map[&plan];
        if deck.rooms.category(from_room) == Some(RoomCategory::Cabin) {
            continue;
        }
        let cell = deck.cells.get_mut(&plan).expect("cell");
        for wi in 0..4 {
            let wall = edge_wall(&occupied, from_room, plan, wi, &room_map, &deck.rooms);
            *wall_material_mut(cell, wi) = wall;
        }
    }
}

fn assign_cabin_module_walls(deck: &mut DeckBuild) {
    let occupied: HashSet<_> = deck.cells.keys().copied().collect();
    let room_map: HashMap<PlanKey, RoomId> =
        deck.cells.iter().map(|(&c, cell)| (c, cell.room)).collect();
    let coords: Vec<PlanKey> = deck.cells.keys().copied().collect();
    for plan in coords {
        let from_room = room_map[&plan];
        if deck.rooms.category(from_room) != Some(RoomCategory::Cabin) {
            continue;
        }
        let (lx, ly) = cabin_local(plan);
        let cell = deck.cells.get_mut(&plan).expect("cell");
        if is_cabin_interior(lx, ly) {
            cell.wall1 = Material::Open;
            cell.wall2 = Material::Open;
            cell.wall3 = Material::Open;
            cell.wall4 = Material::Open;
        } else {
            for wi in 0..4 {
                *wall_material_mut(cell, wi) =
                    edge_wall(&occupied, from_room, plan, wi, &room_map, &deck.rooms);
            }
        }
    }
}

fn wall_material_mut(cell: &mut Cell, wall_idx: usize) -> &mut Material {
    match wall_idx {
        0 => &mut cell.wall1,
        1 => &mut cell.wall2,
        2 => &mut cell.wall3,
        _ => &mut cell.wall4,
    }
}

fn wall_material(cell: &Cell, wall_idx: usize) -> Material {
    match wall_idx {
        0 => cell.wall1,
        1 => cell.wall2,
        2 => cell.wall3,
        _ => cell.wall4,
    }
}

fn opposite_wall(wall_idx: usize) -> usize {
    match wall_idx {
        0 => 2,
        1 => 3,
        2 => 0,
        _ => 1,
    }
}

fn is_corridor_cell(
    coord: PlanKey,
    corridor_cells: &HashSet<PlanKey>,
    catalog: &RoomCatalog,
    room_map: &HashMap<PlanKey, RoomId>,
) -> bool {
    corridor_cells.contains(&coord)
        || room_category_at(catalog, room_map, coord) == Some(RoomCategory::Corridor)
}

fn deck_has_corridor_at(
    deck: &DeckBuild,
    coord: PlanKey,
    room_map: &HashMap<PlanKey, RoomId>,
) -> bool {
    room_map
        .get(&coord)
        .and_then(|&id| deck.rooms.category(id))
        .is_some_and(|c| c == RoomCategory::Corridor)
}

/// Ensure fore/aft faces toward corridor cells are `MarinePanel` before door placement.
fn prime_fore_aft_walls_toward_corridor(deck: &mut DeckBuild) {
    let room_map: HashMap<PlanKey, RoomId> =
        deck.cells.iter().map(|(&c, cell)| (c, cell.room)).collect();
    let mut by_room: HashMap<RoomId, Vec<PlanKey>> = HashMap::new();
    for (&coord, &room) in &room_map {
        if deck.rooms.category(room) == Some(RoomCategory::Cabin) {
            by_room.entry(room).or_default().push(coord);
        }
    }
    for cells in by_room.values() {
        let (min_x, max_x) = module_x_span(cells);
        for &plan in cells {
            let wall_idx = if plan.0 == min_x {
                3
            } else if plan.0 == max_x {
                1
            } else {
                continue;
            };
            let Some(nb) = neighbor_plan(plan, wall_idx) else {
                continue;
            };
            if deck_has_corridor_at(deck, nb, &room_map) {
                let cell = deck.cells.get_mut(&plan).expect("cell");
                let w = wall_material_mut(cell, wall_idx);
                if *w != Material::Door {
                    *w = Material::MarinePanel;
                }
            }
        }
    }
}

fn assign_cabin_openings(deck: &mut DeckBuild, _corridor_cells: &HashSet<PlanKey>) {
    let room_map: HashMap<PlanKey, RoomId> =
        deck.cells.iter().map(|(&c, cell)| (c, cell.room)).collect();
    let occupied: HashSet<_> = deck.cells.keys().copied().collect();

    let mut cabin_rooms: HashSet<RoomId> = HashSet::new();
    for (&coord, &room) in &room_map {
        if deck.rooms.category(room) == Some(RoomCategory::Cabin) {
            cabin_rooms.insert(room);
        }
        let _ = coord;
    }

    for room in cabin_rooms {
        let mut module_cells = Vec::new();
        for (&coord, &r) in &room_map {
            if r == room {
                module_cells.push(coord);
            }
        }

        let (min_x, max_x) = module_x_span(&module_cells);
        let mut door_candidates: Vec<(PlanKey, usize)> = Vec::new();
        let mut window_candidates: Vec<(PlanKey, usize)> = Vec::new();
        let mut has_hull_edge = false;

        for &plan in &module_cells {
            let (lx, ly) = cabin_local(plan);
            if is_cabin_interior(lx, ly) {
                continue;
            }
            for wall_idx in 0..4 {
                let nb = neighbor_plan(plan, wall_idx);
                let wall = wall_material(deck.cells.get(&plan).expect("cell"), wall_idx);
                if nb.is_some_and(|nb| deck_has_corridor_at(deck, nb, &room_map))
                    && is_module_fore_aft_door_wall(wall_idx, lx, plan.0, min_x, max_x)
                    && (wall == Material::MarinePanel || wall == Material::Open)
                {
                    door_candidates.push((plan, wall_idx));
                }
                if wall == Material::Hull && nb.is_none_or(|nb| !occupied.contains(&nb)) {
                    has_hull_edge = true;
                    window_candidates.push((plan, wall_idx));
                }
            }
        }

        let door =
            pick_door(&door_candidates, &module_cells).or_else(|| door_candidates.first().copied());
        let window = if has_hull_edge {
            pick_window(&window_candidates, door)
        } else {
            None
        };

        if let Some((coord, wall_idx)) = door {
            let cell = deck.cells.get_mut(&coord).expect("cell");
            let wall = wall_material_mut(cell, wall_idx);
            if *wall != Material::Door {
                *wall = Material::Door;
            }
        }
        if let Some((coord, wall_idx)) = window {
            *wall_material_mut(deck.cells.get_mut(&coord).expect("cell"), wall_idx) =
                Material::Window;
        }
    }

    force_cabin_doors_on_fore_aft(deck);
}

/// Last pass: every cabin room with fore/aft corridor access gets a door on the 3 m bulkhead.
fn force_cabin_doors_on_fore_aft(deck: &mut DeckBuild) {
    let room_map: HashMap<PlanKey, RoomId> =
        deck.cells.iter().map(|(&c, cell)| (c, cell.room)).collect();

    let mut by_room: HashMap<RoomId, Vec<PlanKey>> = HashMap::new();
    for (&coord, &room) in &room_map {
        if deck.rooms.category(room) == Some(RoomCategory::Cabin) {
            by_room.entry(room).or_default().push(coord);
        }
    }

    for (_room, cells) in by_room {
        if cells.len() < 4 {
            continue;
        }
        if cells
            .iter()
            .any(|&coord| deck.cells.get(&coord).is_some_and(cell_has_door_material))
        {
            continue;
        }

        let (min_x, max_x) = module_x_span(&cells);
        for (end_x, wall_idx) in [(min_x, 3usize), (max_x, 1usize)] {
            for &plan in &cells {
                if plan.0 != end_x {
                    continue;
                }
                let Some(nb) = neighbor_plan(plan, wall_idx) else {
                    continue;
                };
                if deck_has_corridor_at(deck, nb, &room_map) {
                    *wall_material_mut(deck.cells.get_mut(&plan).expect("cell"), wall_idx) =
                        Material::Door;
                    break;
                }
            }
        }
    }
}

fn cell_has_door_material(cell: &Cell) -> bool {
    [cell.wall1, cell.wall2, cell.wall3, cell.wall4]
        .iter()
        .any(|&w| w == Material::Door)
}

fn pick_door(
    candidates: &[(PlanKey, usize)],
    module_cells: &[PlanKey],
) -> Option<(PlanKey, usize)> {
    if candidates.is_empty() {
        return None;
    }
    let mut best = candidates[0];
    let mut best_score = door_candidate_score(best, module_cells);
    for &cand in &candidates[1..] {
        let score = door_candidate_score(cand, module_cells);
        if score > best_score {
            best = cand;
            best_score = score;
        }
    }
    Some(best)
}

fn door_candidate_score((plan, wall_idx): (PlanKey, usize), module_cells: &[PlanKey]) -> i32 {
    let (lx, _) = cabin_local(plan);
    let (min_x, max_x) = module_x_span(module_cells);
    let mut score = 0;
    if is_module_fore_aft_door_wall(wall_idx, lx, plan.0, min_x, max_x) {
        score += 20;
    }
    if lx == 1 {
        score += 5;
    }
    let module_mid_x = (min_x + max_x) / 2;
    score -= plan.0.abs_diff(module_mid_x) as i32;
    score
}

fn is_opposite_cabin_end(plan1: PlanKey, plan2: PlanKey) -> bool {
    let (lx1, ly1) = cabin_local(plan1);
    let (lx2, ly2) = cabin_local(plan2);
    lx1 == 1 && lx2 == 1 && ((ly1 <= 1 && ly2 >= 4) || (ly1 >= 4 && ly2 <= 1))
}

fn pick_window(
    candidates: &[(PlanKey, usize)],
    door: Option<(PlanKey, usize)>,
) -> Option<(PlanKey, usize)> {
    let filtered: Vec<_> = candidates
        .iter()
        .copied()
        .filter(|&(coord, _)| {
            door.map(|(door_coord, _)| !is_opposite_cabin_end(door_coord, coord))
                .unwrap_or(true)
        })
        .collect();
    if filtered.is_empty() {
        return None;
    }
    let pool = &filtered;
    let mut best = pool[0];
    let mut best_score = window_candidate_score(best);
    for &cand in &pool[1..] {
        let score = window_candidate_score(cand);
        if score > best_score {
            best = cand;
            best_score = score;
        }
    }
    Some(best)
}

fn window_candidate_score((plan, _wall_idx): (PlanKey, usize)) -> i32 {
    let (lx, ly) = cabin_local(plan);
    let mut score = plan.1 as i32;
    if lx == 1 || ly == 3 {
        score += 10;
    }
    let _ = ly;
    score
}

/// All deck occupancy grids at `step_m` cell spacing, stored in a shared [`CellBox`].
pub fn deck_cell_layouts(step_m: f32) -> DeckLayouts {
    let mut cell_box = CellBox::new();
    let mut decks = Vec::with_capacity(NUM_DECKS);
    for deck_i in 0..NUM_DECKS {
        let deck_z = deck_i as u8;
        let centers = fallback_deck_cell_centers(deck_i, step_m)
            .into_iter()
            .filter(|p| profile_allows_cell(deck_i, *p))
            .collect::<Vec<_>>();
        let occupied: HashSet<_> = centers
            .iter()
            .filter_map(|&p| plan_key_from_world(p, deck_z))
            .collect();
        let mut corridor_cells = build_cabin_spine_layout(&centers, deck_i);
        extend_corridor_door_connectors(&mut corridor_cells, &occupied, deck_i);
        ensure_cabin_corridor_touch(&mut corridor_cells, &occupied, deck_i);

        let mut rooms = RoomCatalog::default();
        let mut cabin_rooms = HashMap::new();
        let corridor_room = rooms.insert("Corridor", deck_z, RoomCategory::Corridor);
        let corridor_floor = corridor_floor_material();

        let mut cells = HashMap::new();
        for &p in &centers {
            let Some(coord) = plan_key_from_world(p, deck_z) else {
                continue;
            };
            if corridor_cells.contains(&coord) {
                cells.insert(coord, Cell::new(corridor_floor, corridor_room));
                continue;
            }
            let Some(beam) = effective_beam_m(deck_i, p.y) else {
                continue;
            };
            let bands = cross_section_bands(beam);
            let Some(band) = band_index_at_x(p.x, &bands) else {
                continue;
            };
            if is_spine_band(band) {
                cells.insert(coord, Cell::new(corridor_floor, corridor_room));
            } else {
                let room = room_for_cabin(&mut rooms, &mut cabin_rooms, deck_z, coord);
                cells.insert(coord, Cell::new(Material::CabinPartition, room));
            }
        }

        let mut build = DeckBuild { cells, rooms };
        assign_cabin_module_walls(&mut build);
        assign_non_cabin_walls(&mut build);
        prime_fore_aft_walls_toward_corridor(&mut build);
        assign_cabin_openings(&mut build, &corridor_cells);

        for (plan, cell) in build.cells {
            let index = CellIndex::with_plan(deck_z, plan).expect("plan in box range");
            cell_box.insert(index, cell);
        }
        decks.push(DeckMeta { rooms: build.rooms });
    }
    DeckLayouts {
        cells: cell_box,
        decks,
    }
}

/// Alias retained for callers migrating from the old name.
pub fn deck_layouts(step_m: f32) -> DeckLayouts {
    deck_cell_layouts(step_m)
}

fn is_perimeter_cell(cell: PlanKey, occupied: &HashSet<PlanKey>) -> bool {
    for wall_idx in 0..4 {
        let Some(nb) = neighbor_plan(cell, wall_idx) else {
            return true;
        };
        if !occupied.contains(&nb) {
            return true;
        }
    }
    false
}

// TODO: update for `CellBox` / `PlanKey` (disabled until migrated).
#[cfg(any())]
mod layout_tests {
    use super::*;
    use crate::cell::Material;

    fn deck_five(layouts: &DeckLayouts) -> DeckCells<'_> {
        layouts.deck(4)
    }

    #[test]
    fn deck_cell_layouts_builds_walls_and_floors() {
        let layouts = deck_cell_layouts(CELL_SIZE_M);
        let deck = deck_five(&layouts);
        let cell = deck.iter_cells().next().expect("cell").1;
        assert_ne!(cell.floor, Material::Open);
    }

    #[test]
    fn deck_five_has_no_lattice_holes_anywhere() {
        let layouts = deck_cell_layouts(CELL_SIZE_M);
        let deck = deck_five(&layouts);
        let occupied: HashSet<_> = deck.plan_keys().collect();
        for plan in &occupied {
            for wall_idx in 0..4 {
                let Some(mid) = neighbor_plan(*plan, wall_idx) else {
                    continue;
                };
                let Some(far) = neighbor_plan(mid, wall_idx) else {
                    continue;
                };
                if occupied.contains(&far) && !occupied.contains(&mid) {
                    panic!("deck 5 lattice hole: {mid:?} between {plan:?} and {far:?}");
                }
            }
        }
    }

    fn all_walls_open(cell: &Cell) -> bool {
        [cell.wall1, cell.wall2, cell.wall3, cell.wall4]
            .iter()
            .all(|&w| w == Material::Open)
    }

    fn room_category(deck: DeckCells<'_>, room: RoomId) -> RoomCategory {
        deck.rooms.category(room).expect("room")
    }

    fn cell_has_door(cell: &Cell) -> Option<usize> {
        [cell.wall1, cell.wall2, cell.wall3, cell.wall4]
            .iter()
            .position(|&w| w == Material::Door)
    }

    fn cell_has_window(cell: &Cell) -> Option<usize> {
        [cell.wall1, cell.wall2, cell.wall3, cell.wall4]
            .iter()
            .position(|&w| w == Material::Window)
    }

    fn neighbor_is_corridor(deck: DeckCells<'_>, coord: PlanKey, wall_idx: usize) -> bool {
        let Some(nb) = neighbor_plan(coord, wall_idx) else {
            return false;
        };
        deck.get(nb)
            .is_some_and(|c| room_category(deck, c.room) == RoomCategory::Corridor)
    }

    #[test]
    fn deck_five_has_corridor_rooms() {
        let layouts = deck_cell_layouts(CELL_SIZE_M);
        let deck = deck_five(&layouts);
        let corridor_cells: Vec<_> = deck
            .iter_cells()
            .filter(|(_, c)| room_category(deck, c.room) == RoomCategory::Corridor)
            .collect();
        assert!(
            !corridor_cells.is_empty(),
            "expected corridor cells on deck 5"
        );
        for (_, cell) in &corridor_cells {
            assert_eq!(room_category(deck, cell.room), RoomCategory::Corridor);
        }
    }

    fn midship_y_for_deck(deck_index: usize) -> f32 {
        let profile = deck_profile(deck_index);
        (profile.y_aft + profile.y_fwd) * 0.5
    }

    fn corridor_y_runs_at_x(deck: DeckCells<'_>, along_x: u16) -> Vec<Vec<u16>> {
        let mut ys: Vec<u16> = deck
            .iter_cells()
            .filter(|(p, c)| {
                p.0 == along_x && room_category(deck, c.room) == RoomCategory::Corridor
            })
            .map(|(p, _)| p.1)
            .collect();
        ys.sort_unstable();
        ys.dedup();
        let mut runs = Vec::new();
        let mut current = Vec::new();
        for &y in &ys {
            if current.is_empty() || y == *current.last().unwrap() + 1 {
                current.push(y);
            } else {
                runs.push(current);
                current = vec![y];
            }
        }
        if !current.is_empty() {
            runs.push(current);
        }
        runs
    }

    #[test]
    fn midship_has_six_cabin_columns() {
        let layouts = deck_cell_layouts(CELL_SIZE_M);
        let deck = deck_five(&layouts);
        let mid_y = midship_y_for_deck(4);
        let mid_x = CellIndex::from_world_xy_deck(Vec2::new(0.0, mid_y), 4)
            .expect("midship")
            .x;
        let mut cabin_ys = HashSet::new();
        for (plan, cell) in deck.iter_cells() {
            if plan.0 != mid_x {
                continue;
            }
            if room_category(deck, cell.room) == RoomCategory::Cabin {
                cabin_ys.insert(plan.1);
            }
        }
        assert!(
            cabin_ys.len() >= 6,
            "expected at least six cabin column cells at midship x={mid_x}, got {}",
            cabin_ys.len()
        );
    }

    #[test]
    fn midship_has_three_spine_corridors() {
        let layouts = deck_cell_layouts(CELL_SIZE_M);
        let deck = deck_five(&layouts);
        let mid_y = midship_y_for_deck(4);
        let mid_x = CellIndex::from_world_xy_deck(Vec2::new(0.0, mid_y), 4)
            .expect("midship")
            .x;
        let runs = corridor_y_runs_at_x(deck, mid_x);
        let spine_runs: Vec<_> = runs.iter().filter(|run| run.len() >= 2).collect();
        assert!(
            spine_runs.len() >= 3,
            "expected three interior spine runs at midship, got {spine_runs:?}"
        );
    }

    #[test]
    fn spine_width_fills_beam() {
        let deck_index = 4;
        let mid_y = midship_y_for_deck(deck_index);
        let beam = effective_beam_m(deck_index, mid_y).expect("beam");
        let spine_w = spine_width_m(beam);
        let total = CABIN_COLUMNS_TOTAL_WIDTH_M + NUM_INTERIOR_SPINES as f32 * spine_w;
        assert!(
            (total - beam).abs() < 1.0,
            "expected 6×3m + 3×spine ≈ beam ({total} vs {beam})"
        );
    }

    #[test]
    fn doors_on_three_metre_walls_only() {
        let layouts = deck_cell_layouts(CELL_SIZE_M);
        let deck = deck_five(&layouts);
        let mut cabin_rooms: HashMap<RoomId, Vec<PlanKey>> = HashMap::new();
        for (coord, cell) in deck.iter_cells() {
            if room_category(deck, cell.room) == RoomCategory::Cabin {
                cabin_rooms.entry(cell.room).or_default().push(coord);
            }
        }
        for (plan, cell) in deck.iter_cells() {
            let Some(wi) = cell_has_door(cell) else {
                continue;
            };
            let (lx, _) = cabin_local(plan);
            let cells = cabin_rooms.get(&cell.room).expect("cabin cells");
            let (min_x, max_x) = module_x_span(cells);
            assert!(
                is_module_fore_aft_door_wall(wi, lx, plan.0, min_x, max_x),
                "door at {plan:?} wall{wi} must be on 3m fore/aft bulkhead"
            );
            assert!(
                wi == 1 || wi == 3,
                "door wall index must be wall2 or wall4, got {wi}"
            );
        }
    }

    fn module_borders_corridor(deck: DeckCells<'_>, ox: u16, oy: u16) -> bool {
        for ly in 0..CABIN_WIDTH_CELLS {
            for lx in 0..CABIN_LENGTH_CELLS {
                let coord = (ox.saturating_add(lx as u16), oy.saturating_add(ly as u16));
                for wall_idx in 0..4 {
                    if neighbor_is_corridor(deck, coord, wall_idx) {
                        return true;
                    }
                }
            }
        }
        false
    }

    fn cabin_room_has_door(deck: DeckCells<'_>, room: RoomId) -> bool {
        deck.iter_cells()
            .any(|(_, cell)| cell.room == room && cell_has_door(cell).is_some())
    }

    #[test]
    fn every_cabin_room_has_door_to_corridor() {
        let layouts = deck_cell_layouts(CELL_SIZE_M);
        let deck = deck_five(&layouts);
        let mut cabin_rooms = HashSet::new();
        for (_, cell) in deck.iter_cells() {
            if room_category(deck, cell.room) == RoomCategory::Cabin {
                cabin_rooms.insert(cell.room);
            }
        }
        for room in cabin_rooms {
            let cell_count = deck.iter_cells().filter(|(_, c)| c.room == room).count();
            if cell_count < 4 {
                continue;
            }
            assert!(
                cabin_room_has_door(deck, room),
                "cabin room {:?} ({cell_count} cells) should have a door",
                room
            );
        }
    }

    fn room_has_hull_edge(deck: DeckCells<'_>, room: RoomId) -> bool {
        let occupied: HashSet<_> = deck.plan_keys().collect();
        deck.iter_cells().any(|(plan, cell)| {
            if cell.room != room {
                return false;
            }
            (0..4).any(|wi| neighbor_plan(plan, wi).is_none_or(|nb| !occupied.contains(&nb)))
        })
    }

    fn room_window_and_door(
        deck: DeckCells<'_>,
        room: RoomId,
    ) -> (Option<(PlanKey, usize)>, Option<(PlanKey, usize)>) {
        let mut door = None;
        let mut window = None;
        for (coord, cell) in deck.iter_cells() {
            if cell.room != room {
                continue;
            }
            if let Some(wi) = cell_has_door(cell) {
                door = Some((coord, wi));
            }
            if let Some(wi) = cell_has_window(cell) {
                window = Some((coord, wi));
            }
        }
        (door, window)
    }

    #[test]
    fn full_cabin_module_has_perimeter_walls_and_door_window() {
        let deck = &deck_cell_layouts(CELL_SIZE_M)[4];
        let mut found_module = false;
        for &(ix, iy) in deck.cells.keys() {
            let ox = ix.div_euclid(CABIN_WIDTH_CELLS) * CABIN_WIDTH_CELLS;
            let oy = iy.div_euclid(CABIN_LENGTH_CELLS) * CABIN_LENGTH_CELLS;
            if (ix, iy) != (ox, oy) {
                continue;
            }
            let room = {
                let mut cabin_cells = 0u32;
                let mut room_id = None;
                for lx in 0..CABIN_WIDTH_CELLS {
                    for ly in 0..CABIN_LENGTH_CELLS {
                        if let Some(cell) = deck.cells.get(&(ox + lx, oy + ly)) {
                            if room_category(deck, cell.room) == RoomCategory::Cabin {
                                cabin_cells += 1;
                                room_id = Some(cell.room);
                            }
                        }
                    }
                }
                if cabin_cells < 15 {
                    continue;
                }
                room_id
            };
            let Some(room) = room else {
                continue;
            };
            if room_category(deck, room) != RoomCategory::Cabin {
                continue;
            }
            if !module_borders_corridor(deck, ox, oy) {
                continue;
            }
            let (door, _) = room_window_and_door(deck, room);
            if door.is_none() {
                continue;
            }
            found_module = true;

            let mut interior_open = 0u32;
            let mut perimeter_with_wall = 0u32;
            for lx in 0..CABIN_WIDTH_CELLS {
                for ly in 0..CABIN_LENGTH_CELLS {
                    let cell = deck.cells.get(&(ox + lx, oy + ly)).expect("cell");
                    if is_cabin_interior(lx, ly) {
                        assert!(
                            all_walls_open(cell),
                            "interior ({lx},{ly}) should have all Open walls"
                        );
                        interior_open += 1;
                    } else if !all_walls_open(cell) {
                        perimeter_with_wall += 1;
                    }
                }
            }
            assert_eq!(interior_open, 4, "expected four interior cells");
            assert_eq!(
                perimeter_with_wall, 14,
                "expected fourteen perimeter cells with walls"
            );
            break;
        }
        assert!(
            found_module,
            "expected at least one full 3×6 cabin module bordering corridor on deck 5"
        );

        let mut found_exterior = false;
        let mut found_interior = false;
        let mut cabin_rooms = HashSet::new();
        for cell in deck.cells.values() {
            if room_category(deck, cell.room) == RoomCategory::Cabin {
                cabin_rooms.insert(cell.room);
            }
        }
        let occupied: HashSet<_> = deck.cells.keys().copied().collect();
        for room in cabin_rooms {
            let exterior = room_has_hull_edge(deck, room);
            let (door, window) = room_window_and_door(deck, room);
            let (door_coord, door_wall) = door.expect("cabin room should have a door");
            assert!(
                neighbor_is_corridor(deck, door_coord, door_wall),
                "door must face a corridor cell"
            );
            if exterior {
                found_exterior = true;
                let (window_coord, window_wall) =
                    window.expect("exterior cabin room should have a window");
                let nb = neighbor_coord(window_coord, window_wall);
                assert!(!occupied.contains(&nb), "window must be on hull perimeter");
                assert!(
                    !is_opposite_cabin_end(door_coord, window_coord),
                    "window must not be on the cabin end opposite the door"
                );
            } else {
                found_interior = true;
                assert!(
                    window.is_none(),
                    "interior cabin room should not have a window"
                );
            }
        }
        assert!(
            found_exterior,
            "expected at least one exterior cabin room on deck 5"
        );
        assert!(
            found_interior,
            "expected at least one interior cabin room on deck 5"
        );
    }
}
