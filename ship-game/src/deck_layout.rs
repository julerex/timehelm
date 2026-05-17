//! Procedural deck grid and cell zoning shared by 3D and 2D ship views (+X aft→fore, corner origin).

use crate::cell::{
    Cell, Entity, EntityError, EntityId, EntityKind, FloorMaterial, Location, SideMaterial,
};
use crate::cell_box::{self, BEAM, LENGTH};
use crate::cell_box::{CellBox, CellIndex, PlanKey};
use crate::ship_hull::{
    deck_hull_polygon, deck_hull_polygon_upper, point_in_polygon, FIRST_UPPER_DECK_STYLE_INDEX,
    SHIP_BEAM_M, SHIP_LENGTH_M,
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
    pub entities: HashMap<EntityId, Entity>,
}

#[derive(Clone, Default)]
pub struct DeckMeta {}

/// Per-deck metadata and cell access (backed by [`DeckLayouts::cells`]).
#[derive(Clone, Copy)]
pub struct DeckCells<'a> {
    pub deck: u8,
    pub cells: &'a CellBox,
}

impl DeckLayouts {
    #[must_use]
    pub fn deck(&self, deck_index: usize) -> DeckCells<'_> {
        DeckCells {
            deck: deck_index as u8,
            cells: &self.cells,
        }
    }

    pub fn cell_mut(&mut self, index: CellIndex) -> Option<&mut Cell> {
        self.cells.get_mut(index)
    }

    pub fn insert_entity(
        &mut self,
        id: EntityId,
        kind: EntityKind,
        location: Location,
    ) -> Result<(), EntityError> {
        if self.entities.contains_key(&id) {
            return Err(EntityError::DuplicateId);
        }
        let index = CellIndex::from_location(location).ok_or(EntityError::InvalidLocation)?;
        if !self.cells.contains(index) {
            return Err(EntityError::InvalidLocation);
        }
        self.entities.insert(id, Entity { kind, location });
        Ok(())
    }

    pub fn remove_entity(&mut self, id: EntityId) -> Option<Entity> {
        self.entities.remove(&id)
    }

    pub fn set_entity_location(
        &mut self,
        id: EntityId,
        location: Location,
    ) -> Result<(), EntityError> {
        let index = CellIndex::from_location(location).ok_or(EntityError::InvalidLocation)?;
        if !self.cells.contains(index) {
            return Err(EntityError::InvalidLocation);
        }
        let entity = self.entities.get_mut(&id).ok_or(EntityError::UnknownId)?;
        entity.location = location;
        Ok(())
    }

    pub fn entities_at(&self, location: Location) -> impl Iterator<Item = &Entity> {
        self.entities
            .values()
            .filter(move |e| e.location == location)
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
}

impl DeckBuild {
    fn plan_world(plan: PlanKey, deck: u8) -> Vec2 {
        CellIndex::with_plan(deck, plan)
            .expect("plan in range")
            .to_world_xy()
    }
}

#[derive(Clone, Copy)]
struct DeckProfileCentered {
    half_beam_scale: f32,
    y_aft: f32,
    y_fwd: f32,
    bow_taper: f32,
    stern_taper: f32,
    courtyard_half_width: f32,
    courtyard_y_aft: f32,
    courtyard_y_fwd: f32,
}

#[derive(Clone, Copy)]
struct DeckProfile {
    half_beam_scale: f32,
    x_aft: f32,
    x_fwd: f32,
    bow_taper: f32,
    stern_taper: f32,
    courtyard_half_width: f32,
    courtyard_x_aft: f32,
    courtyard_x_fwd: f32,
}

#[must_use]
fn centered_length_y_to_x(y_centered: f32) -> f32 {
    SHIP_LENGTH_M * 0.5 + y_centered
}

impl From<DeckProfileCentered> for DeckProfile {
    fn from(c: DeckProfileCentered) -> Self {
        Self {
            half_beam_scale: c.half_beam_scale,
            x_aft: centered_length_y_to_x(c.y_aft),
            x_fwd: centered_length_y_to_x(c.y_fwd),
            bow_taper: c.bow_taper,
            stern_taper: c.stern_taper,
            courtyard_half_width: c.courtyard_half_width,
            courtyard_x_aft: centered_length_y_to_x(c.courtyard_y_aft),
            courtyard_x_fwd: centered_length_y_to_x(c.courtyard_y_fwd),
        }
    }
}

fn deck_profile(deck_index: usize) -> DeckProfile {
    // Hand-authored deck-by-deck silhouettes inspired by the reference plans.
    const P: [DeckProfileCentered; NUM_DECKS] = [
        DeckProfileCentered {
            half_beam_scale: 0.42,
            y_aft: -SHIP_LENGTH_M * 0.43,
            y_fwd: SHIP_LENGTH_M * 0.14,
            bow_taper: 0.55,
            stern_taper: 0.35,
            courtyard_half_width: 0.0,
            courtyard_y_aft: 0.0,
            courtyard_y_fwd: 0.0,
        }, // 1
        DeckProfileCentered {
            half_beam_scale: 0.52,
            y_aft: -SHIP_LENGTH_M * 0.45,
            y_fwd: SHIP_LENGTH_M * 0.20,
            bow_taper: 0.48,
            stern_taper: 0.30,
            courtyard_half_width: 0.0,
            courtyard_y_aft: 0.0,
            courtyard_y_fwd: 0.0,
        }, // 2
        DeckProfileCentered {
            half_beam_scale: 0.70,
            y_aft: -SHIP_LENGTH_M * 0.47,
            y_fwd: SHIP_LENGTH_M * 0.26,
            bow_taper: 0.44,
            stern_taper: 0.25,
            courtyard_half_width: 0.0,
            courtyard_y_aft: 0.0,
            courtyard_y_fwd: 0.0,
        }, // 3
        DeckProfileCentered {
            half_beam_scale: 0.88,
            y_aft: -SHIP_LENGTH_M * 0.49,
            y_fwd: SHIP_LENGTH_M * 0.38,
            bow_taper: 0.32,
            stern_taper: 0.18,
            courtyard_half_width: 0.0,
            courtyard_y_aft: 0.0,
            courtyard_y_fwd: 0.0,
        }, // 4
        DeckProfileCentered {
            half_beam_scale: 0.96,
            y_aft: -SHIP_LENGTH_M * 0.50,
            y_fwd: SHIP_LENGTH_M * 0.44,
            bow_taper: 0.26,
            stern_taper: 0.12,
            courtyard_half_width: 0.0,
            courtyard_y_aft: 0.0,
            courtyard_y_fwd: 0.0,
        }, // 5
        DeckProfileCentered {
            half_beam_scale: 0.98,
            y_aft: -SHIP_LENGTH_M * 0.50,
            y_fwd: SHIP_LENGTH_M * 0.46,
            bow_taper: 0.24,
            stern_taper: 0.11,
            courtyard_half_width: 0.0,
            courtyard_y_aft: 0.0,
            courtyard_y_fwd: 0.0,
        }, // 6
        DeckProfileCentered {
            half_beam_scale: 0.98,
            y_aft: -SHIP_LENGTH_M * 0.50,
            y_fwd: SHIP_LENGTH_M * 0.47,
            bow_taper: 0.23,
            stern_taper: 0.10,
            courtyard_half_width: 0.0,
            courtyard_y_aft: 0.0,
            courtyard_y_fwd: 0.0,
        }, // 7
        DeckProfileCentered {
            half_beam_scale: 0.97,
            y_aft: -SHIP_LENGTH_M * 0.50,
            y_fwd: SHIP_LENGTH_M * 0.47,
            bow_taper: 0.23,
            stern_taper: 0.10,
            courtyard_half_width: 0.0,
            courtyard_y_aft: 0.0,
            courtyard_y_fwd: 0.0,
        }, // 8
        DeckProfileCentered {
            half_beam_scale: 0.96,
            y_aft: -SHIP_LENGTH_M * 0.50,
            y_fwd: SHIP_LENGTH_M * 0.47,
            bow_taper: 0.24,
            stern_taper: 0.10,
            courtyard_half_width: 0.0,
            courtyard_y_aft: 0.0,
            courtyard_y_fwd: 0.0,
        }, // 9
        DeckProfileCentered {
            half_beam_scale: 0.94,
            y_aft: -SHIP_LENGTH_M * 0.49,
            y_fwd: SHIP_LENGTH_M * 0.46,
            bow_taper: 0.25,
            stern_taper: 0.16,
            courtyard_half_width: 9.0,
            courtyard_y_aft: -SHIP_LENGTH_M * 0.26,
            courtyard_y_fwd: SHIP_LENGTH_M * 0.21,
        }, // 10
        DeckProfileCentered {
            half_beam_scale: 0.93,
            y_aft: -SHIP_LENGTH_M * 0.49,
            y_fwd: SHIP_LENGTH_M * 0.45,
            bow_taper: 0.26,
            stern_taper: 0.17,
            courtyard_half_width: 9.5,
            courtyard_y_aft: -SHIP_LENGTH_M * 0.26,
            courtyard_y_fwd: SHIP_LENGTH_M * 0.21,
        }, // 11
        DeckProfileCentered {
            half_beam_scale: 0.92,
            y_aft: -SHIP_LENGTH_M * 0.48,
            y_fwd: SHIP_LENGTH_M * 0.44,
            bow_taper: 0.27,
            stern_taper: 0.18,
            courtyard_half_width: 10.0,
            courtyard_y_aft: -SHIP_LENGTH_M * 0.25,
            courtyard_y_fwd: SHIP_LENGTH_M * 0.20,
        }, // 12
        DeckProfileCentered {
            half_beam_scale: 0.89,
            y_aft: -SHIP_LENGTH_M * 0.47,
            y_fwd: SHIP_LENGTH_M * 0.42,
            bow_taper: 0.29,
            stern_taper: 0.20,
            courtyard_half_width: 9.2,
            courtyard_y_aft: -SHIP_LENGTH_M * 0.23,
            courtyard_y_fwd: SHIP_LENGTH_M * 0.18,
        }, // 13
        DeckProfileCentered {
            half_beam_scale: 0.86,
            y_aft: -SHIP_LENGTH_M * 0.46,
            y_fwd: SHIP_LENGTH_M * 0.40,
            bow_taper: 0.31,
            stern_taper: 0.22,
            courtyard_half_width: 8.2,
            courtyard_y_aft: -SHIP_LENGTH_M * 0.20,
            courtyard_y_fwd: SHIP_LENGTH_M * 0.16,
        }, // 14
        DeckProfileCentered {
            half_beam_scale: 0.82,
            y_aft: -SHIP_LENGTH_M * 0.45,
            y_fwd: SHIP_LENGTH_M * 0.38,
            bow_taper: 0.32,
            stern_taper: 0.23,
            courtyard_half_width: 7.4,
            courtyard_y_aft: -SHIP_LENGTH_M * 0.18,
            courtyard_y_fwd: SHIP_LENGTH_M * 0.14,
        }, // 15
        DeckProfileCentered {
            half_beam_scale: 0.78,
            y_aft: -SHIP_LENGTH_M * 0.43,
            y_fwd: SHIP_LENGTH_M * 0.36,
            bow_taper: 0.34,
            stern_taper: 0.24,
            courtyard_half_width: 6.2,
            courtyard_y_aft: -SHIP_LENGTH_M * 0.15,
            courtyard_y_fwd: SHIP_LENGTH_M * 0.12,
        }, // 16
        DeckProfileCentered {
            half_beam_scale: 0.73,
            y_aft: -SHIP_LENGTH_M * 0.40,
            y_fwd: SHIP_LENGTH_M * 0.34,
            bow_taper: 0.35,
            stern_taper: 0.25,
            courtyard_half_width: 4.8,
            courtyard_y_aft: -SHIP_LENGTH_M * 0.13,
            courtyard_y_fwd: SHIP_LENGTH_M * 0.10,
        }, // 17
        DeckProfileCentered {
            half_beam_scale: 0.68,
            y_aft: -SHIP_LENGTH_M * 0.37,
            y_fwd: SHIP_LENGTH_M * 0.31,
            bow_taper: 0.37,
            stern_taper: 0.27,
            courtyard_half_width: 0.0,
            courtyard_y_aft: 0.0,
            courtyard_y_fwd: 0.0,
        }, // 18
        DeckProfileCentered {
            half_beam_scale: 0.63,
            y_aft: -SHIP_LENGTH_M * 0.34,
            y_fwd: SHIP_LENGTH_M * 0.28,
            bow_taper: 0.40,
            stern_taper: 0.30,
            courtyard_half_width: 0.0,
            courtyard_y_aft: 0.0,
            courtyard_y_fwd: 0.0,
        }, // 19
        DeckProfileCentered {
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
    P[deck_index.min(NUM_DECKS - 1)].into()
}

/// Half-width from centreline allowed by the deck profile at along-ship `x`, before courtyard / carve rules.
fn sim_half_beam_limit(deck_index: usize, x: f32) -> Option<f32> {
    let profile = deck_profile(deck_index);
    if x < profile.x_aft || x > profile.x_fwd {
        return None;
    }

    let fwd_span = (SHIP_LENGTH_M - profile.x_fwd).max(1.0);
    let aft_span = profile.x_aft.max(1.0);
    let fwd_t = ((x - profile.x_fwd) / fwd_span).clamp(0.0, 1.0);
    let aft_t = ((profile.x_aft - x) / aft_span).clamp(0.0, 1.0);
    let taper = 1.0 - profile.bow_taper * fwd_t * fwd_t - profile.stern_taper * aft_t * aft_t;
    Some(SHIP_BEAM_M * 0.5 * profile.half_beam_scale * taper.max(0.2))
}

fn profile_allows_cell(deck_index: usize, p: Vec2) -> bool {
    let Some(beam_limit) = sim_half_beam_limit(deck_index, p.x) else {
        return false;
    };
    let y_center = SHIP_BEAM_M * 0.5;
    if (p.y - y_center).abs() > beam_limit {
        return false;
    }

    let profile = deck_profile(deck_index);
    if profile.courtyard_half_width > 0.0
        && p.x > profile.courtyard_x_aft
        && p.x < profile.courtyard_x_fwd
        && (p.y - y_center).abs() < profile.courtyard_half_width
    {
        return false;
    }

    // Upper leisure decks: emulate split stern terraces and side tapering.
    if deck_index >= 17 && p.x < SHIP_LENGTH_M * 0.20 && (p.y - y_center).abs() < SHIP_BEAM_M * 0.14
    {
        return false;
    }
    if deck_index >= 18 && p.x > SHIP_LENGTH_M * 0.15 && (p.y - y_center).abs() > SHIP_BEAM_M * 0.22
    {
        return false;
    }

    true
}

fn deck_lower_profile_outline(deck_index: usize) -> Vec<Vec2> {
    const STEPS: usize = 72;
    let profile = deck_profile(deck_index);
    let x_aft = profile.x_aft;
    let x_fwd = profile.x_fwd;
    let y_center = SHIP_BEAM_M * 0.5;
    let hb_fwd = sim_half_beam_limit(deck_index, x_fwd).unwrap_or(0.0);
    let hb_aft = sim_half_beam_limit(deck_index, x_aft).unwrap_or(0.0);

    let mut poly = Vec::with_capacity(STEPS * 2 + 4);
    poly.push(Vec2::new(x_fwd, y_center - hb_fwd));
    for i in 1..STEPS - 1 {
        let t = i as f32 / (STEPS - 1) as f32;
        let x = x_fwd + (x_aft - x_fwd) * t;
        if let Some(hb) = sim_half_beam_limit(deck_index, x) {
            poly.push(Vec2::new(x, y_center - hb));
        }
    }
    poly.push(Vec2::new(x_aft, y_center - hb_aft));
    poly.push(Vec2::new(x_aft, y_center + hb_aft));
    for i in (1..STEPS - 1).rev() {
        let t = i as f32 / (STEPS - 1) as f32;
        let x = x_fwd + (x_aft - x_fwd) * t;
        if let Some(hb) = sim_half_beam_limit(deck_index, x) {
            poly.push(Vec2::new(x, y_center + hb));
        }
    }
    poly.push(Vec2::new(x_fwd, y_center + hb_fwd));
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

/// True when any part of the axis-aligned cell footprint lies inside `poly`.
fn cell_footprint_intersects_polygon(center: Vec2, poly: &[Vec2]) -> bool {
    let hx = cell_box::length_cell_m() * 0.5;
    let hy = cell_box::beam_cell_m() * 0.5;
    let corners = [
        Vec2::new(center.x - hx, center.y - hy),
        Vec2::new(center.x + hx, center.y - hy),
        Vec2::new(center.x + hx, center.y + hy),
        Vec2::new(center.x - hx, center.y + hy),
    ];
    point_in_polygon(center, poly) || corners.iter().any(|&c| point_in_polygon(c, poly))
}

/// Occupied plan keys on the [`CellBox`] lattice inside the deck hull and profile clip.
fn occupied_plans_for_deck(deck_index: usize) -> Vec<PlanKey> {
    let poly = if deck_index >= FIRST_UPPER_DECK_STYLE_INDEX {
        deck_hull_polygon_upper()
    } else {
        deck_hull_polygon()
    };
    let deck_z = deck_index as u8;
    let mut out = Vec::new();
    for y in 0..BEAM as u16 {
        for x in 0..LENGTH as u16 {
            let idx = CellIndex::new(x, y, deck_z).expect("in range");
            let p = idx.to_world_xy();
            if cell_footprint_intersects_polygon(p, &poly) && profile_allows_cell(deck_index, p) {
                out.push((x, y));
            }
        }
    }
    out
}

const CABIN_WIDTH_CELLS: i32 = 3;
const CABIN_LENGTH_CELLS: i32 = 6;
const CABIN_COLUMN_WIDTH_M: f32 = 3.0;
/// Port and starboard cabin rows only (metres across the beam).
const NUM_CABIN_COLUMNS: usize = 2;
const NUM_INTERIOR_SPINES: usize = 1;
const CABIN_COLUMNS_TOTAL_WIDTH_M: f32 = CABIN_COLUMN_WIDTH_M * NUM_CABIN_COLUMNS as f32;
const NUM_CROSS_SECTION_BANDS: usize = NUM_CABIN_COLUMNS + NUM_INTERIOR_SPINES;
type CrossSectionBands = [(f32, f32); NUM_CROSS_SECTION_BANDS];

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

fn effective_beam_m(deck_index: usize, x: f32) -> Option<f32> {
    sim_half_beam_limit(deck_index, x).map(|hb| hb * 2.0)
}

fn spine_width_m(effective_beam_m: f32) -> f32 {
    ((effective_beam_m - CABIN_COLUMNS_TOTAL_WIDTH_M) / NUM_INTERIOR_SPINES as f32).max(1.0)
}

/// Three bands stbd→port: starboard cabins, centre spine, port cabins (metres **Y** from origin).
fn cross_section_bands(effective_beam_m: f32) -> CrossSectionBands {
    let spine_w = spine_width_m(effective_beam_m);
    let widths = [CABIN_COLUMN_WIDTH_M, spine_w, CABIN_COLUMN_WIDTH_M];
    let mut bands = [(0.0, 0.0); NUM_CROSS_SECTION_BANDS];
    let mut y = 0.0;
    for (band, &w) in bands.iter_mut().zip(widths.iter()) {
        *band = (y, y + w);
        y += w;
    }
    bands
}

fn band_index_at_y(y: f32, bands: &CrossSectionBands) -> Option<usize> {
    let last = bands.len() - 1;
    if y < bands[0].0 {
        return Some(0);
    }
    if y >= bands[last].1 {
        return Some(last);
    }
    for (i, &(lo, hi)) in bands.iter().enumerate() {
        if i == last {
            if y >= lo && y <= hi {
                return Some(i);
            }
        } else if y >= lo && y < hi {
            return Some(i);
        }
    }
    None
}

fn is_spine_band(band_index: usize) -> bool {
    band_index == 1
}

fn module_x_span(module_cells: &[PlanKey]) -> (u16, u16) {
    let min_x = module_cells.iter().map(|p| p.0).min().unwrap();
    let max_x = module_cells.iter().map(|p| p.0).max().unwrap();
    (min_x, max_x)
}

/// Inboard direction (±1 in grid `y`) from a side cabin band toward the centre spine.
fn cabin_band_inboard_dy(band_index: usize) -> i32 {
    match band_index {
        0 => 1,
        2 => -1,
        _ => 0,
    }
}

/// Longitudinal centre corridor from the port / spine / starboard cross-section (all decks).
fn build_cabin_spine_layout(centers: &[Vec2], deck_index: usize) -> HashSet<PlanKey> {
    let deck = deck_index as u8;
    let mut corridor = HashSet::new();
    for &p in centers {
        let Some(beam) = effective_beam_m(deck_index, p.x) else {
            continue;
        };
        let bands = cross_section_bands(beam);
        let Some(bi) = band_index_at_y(p.y, &bands) else {
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
    let x_m = DeckBuild::plan_world(stub, deck).x;
    let Some(beam) = effective_beam_m(deck_index, x_m) else {
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
        let y_m = DeckBuild::plan_world(next, deck).y;
        let Some(bi) = band_index_at_y(y_m, &bands) else {
            break;
        };
        if !occupied.contains(&next) {
            break;
        }
        if is_spine_band(bi) {
            corridor.insert(next);
            break;
        }
        y = next_y;
    }
}

fn inboard_column_lx(band_index: usize) -> i32 {
    match band_index {
        0 => CABIN_WIDTH_CELLS - 1,
        2 => 0,
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
        let x_m = DeckBuild::plan_world(((min_x + max_x) / 2, oy), deck).x;
        let Some(beam) = effective_beam_m(deck_index, x_m) else {
            continue;
        };
        let bands = cross_section_bands(beam);
        let across_y = oy.saturating_add(1);
        let y_m = DeckBuild::plan_world((ox, across_y), deck).y;
        let Some(band) = band_index_at_y(y_m, &bands) else {
            continue;
        };
        if is_spine_band(band) {
            continue;
        }
        let inboard_dy = cabin_band_inboard_dy(band);
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

        let door_across = oy.saturating_add(1);
        for &(carve_x, door_wall) in &[(min_x, 3usize), (max_x, 1usize)] {
            let door_cell = (carve_x, door_across);
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
            let link_stub = (stub.0, oy.saturating_add(inboard_ly as u16));
            if occupied.contains(&link_stub) {
                link_stub_to_spine(corridor, occupied, link_stub, inboard_dy, deck_index);
            }
            break;
        }
    }
}

/// Ensure inboard cabin edges touch the centre spine (without carving the full 3 m row).
fn ensure_cabin_corridor_touch(
    corridor: &mut HashSet<PlanKey>,
    occupied: &HashSet<PlanKey>,
    deck_index: usize,
) {
    let deck = deck_index as u8;
    for &coord in occupied.iter() {
        if corridor.contains(&coord) {
            continue;
        }
        let world = DeckBuild::plan_world(coord, deck);
        let Some(beam) = effective_beam_m(deck_index, world.x) else {
            continue;
        };
        let bands = cross_section_bands(beam);
        let Some(band) = band_index_at_y(world.y, &bands) else {
            continue;
        };
        if is_spine_band(band) {
            continue;
        }
        let (across_local, _) = cabin_local(coord);
        if across_local != inboard_column_lx(band) {
            continue;
        }
        let inboard_dy = cabin_band_inboard_dy(band);
        let inboard_y = coord.1 as i32 + inboard_dy;
        if !(0..crate::cell_box::BEAM as i32).contains(&inboard_y) {
            continue;
        }
        let inboard = (coord.0, inboard_y as u16);
        if !occupied.contains(&inboard) {
            continue;
        }
        let inboard_world = DeckBuild::plan_world(inboard, deck);
        if band_index_at_y(inboard_world.y, &bands).is_some_and(is_spine_band) {
            corridor.insert(inboard);
        }
    }
}

fn corridor_floor_material() -> FloorMaterial {
    FloorMaterial::Carpet
}

/// Fill 1-cell gaps between two occupied cells (corridor stub) so the deck has no lattice holes.
fn fill_single_cell_holes(build: &mut DeckBuild, deck_index: usize) {
    let corridor_floor = corridor_floor_material();
    let deck = deck_index as u8;
    for _ in 0..4 {
        let occupied: HashSet<_> = build.cells.keys().copied().collect();
        let mut filled = false;
        for &plan in &occupied {
            for wall_idx in 0..4 {
                let Some(mid) = neighbor_plan(plan, wall_idx) else {
                    continue;
                };
                let Some(far) = neighbor_plan(mid, wall_idx) else {
                    continue;
                };
                if !occupied.contains(&far) || occupied.contains(&mid) {
                    continue;
                }
                let world = DeckBuild::plan_world(mid, deck);
                if !profile_allows_cell(deck_index, world) {
                    continue;
                }
                build.cells.insert(mid, Cell::new(corridor_floor));
                filled = true;
            }
        }
        if !filled {
            break;
        }
    }
}

fn side_material_mut(cell: &mut Cell, side_idx: usize) -> &mut SideMaterial {
    match side_idx {
        0 => &mut cell.side1,
        1 => &mut cell.side2,
        2 => &mut cell.side3,
        _ => &mut cell.side4,
    }
}

fn edge_side(
    occupied: &HashSet<PlanKey>,
    corridor_cells: &HashSet<PlanKey>,
    from: PlanKey,
    side_idx: usize,
    from_is_corridor: bool,
) -> SideMaterial {
    match neighbor_plan(from, side_idx) {
        None => SideMaterial::MarinePanel,
        Some(nb) if !occupied.contains(&nb) => SideMaterial::MarinePanel,
        Some(nb) => {
            let to_is_corridor = corridor_cells.contains(&nb);
            if from_is_corridor == to_is_corridor {
                SideMaterial::Open
            } else {
                SideMaterial::MarinePanel
            }
        }
    }
}

fn assign_cell_sides(build: &mut DeckBuild, corridor_cells: &HashSet<PlanKey>) {
    let occupied: HashSet<_> = build.cells.keys().copied().collect();
    let coords: Vec<_> = build.cells.keys().copied().collect();
    for plan in coords {
        let from_corridor = corridor_cells.contains(&plan);
        let (lx, ly) = cabin_local(plan);
        let cell = build.cells.get_mut(&plan).expect("cell");
        if !from_corridor && is_cabin_interior(lx, ly) {
            cell.side1 = SideMaterial::Open;
            cell.side2 = SideMaterial::Open;
            cell.side3 = SideMaterial::Open;
            cell.side4 = SideMaterial::Open;
        } else {
            for wi in 0..4 {
                *side_material_mut(cell, wi) =
                    edge_side(&occupied, corridor_cells, plan, wi, from_corridor);
            }
        }
    }
}

/// All deck occupancy grids on the [`CellBox`] lattice, stored in a shared [`CellBox`].
///
/// `step_m` is retained for callers that used the legacy 1 m sampling pitch; layout now
/// enumerates the fixed `360×60` grid directly.
pub fn deck_cell_layouts(_step_m: f32) -> DeckLayouts {
    let mut cell_box = CellBox::new();
    let mut decks = Vec::with_capacity(NUM_DECKS);
    for deck_i in 0..NUM_DECKS {
        let deck_z = deck_i as u8;
        let occupied_plans = occupied_plans_for_deck(deck_i);
        let occupied: HashSet<_> = occupied_plans.iter().copied().collect();
        let centers: Vec<Vec2> = occupied_plans
            .iter()
            .map(|&plan| {
                CellIndex::with_plan(deck_z, plan)
                    .expect("plan in range")
                    .to_world_xy()
            })
            .collect();
        let mut corridor_cells = build_cabin_spine_layout(&centers, deck_i);
        extend_corridor_door_connectors(&mut corridor_cells, &occupied, deck_i);
        ensure_cabin_corridor_touch(&mut corridor_cells, &occupied, deck_i);

        let corridor_floor = corridor_floor_material();

        let mut cells = HashMap::new();
        for &p in &centers {
            let Some(coord) = plan_key_from_world(p, deck_z) else {
                continue;
            };
            if corridor_cells.contains(&coord) {
                cells.insert(coord, Cell::new(corridor_floor));
                continue;
            }
            let Some(beam) = effective_beam_m(deck_i, p.x) else {
                continue;
            };
            let bands = cross_section_bands(beam);
            let Some(band) = band_index_at_y(p.y, &bands) else {
                continue;
            };
            if is_spine_band(band) {
                cells.insert(coord, Cell::new(corridor_floor));
            } else {
                cells.insert(coord, Cell::new(FloorMaterial::Wood));
            }
        }

        let mut build = DeckBuild { cells };
        fill_single_cell_holes(&mut build, deck_i);
        assign_cell_sides(&mut build, &corridor_cells);

        for (plan, cell) in build.cells {
            let index = CellIndex::with_plan(deck_z, plan).expect("plan in box range");
            cell_box.insert(index, cell);
        }
        decks.push(DeckMeta {});
    }
    DeckLayouts {
        cells: cell_box,
        decks,
        entities: HashMap::new(),
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

/*
#[cfg(test)]
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

    fn midship_x_for_deck(deck_index: usize) -> f32 {
        let profile = deck_profile(deck_index);
        (profile.x_aft + profile.x_fwd) * 0.5
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

    fn midship_cross_section_row(deck: DeckCells<'_>, along_x: u16) -> String {
        let mut row = String::new();
        for y in 0..crate::cell_box::BEAM as u16 {
            let ch = match deck.get((along_x, y)) {
                None => '.',
                Some(c) => match room_category(deck, c.room) {
                    RoomCategory::Cabin => 'C',
                    RoomCategory::Corridor => '#',
                    _ => '?',
                },
            };
            row.push(ch);
        }
        row
    }

    #[test]
    fn midship_has_port_and_starboard_cabin_bands() {
        let layouts = deck_cell_layouts(CELL_SIZE_M);
        let deck = deck_five(&layouts);
        let mid_x = midship_x_for_deck(4);
        let mid_idx = CellIndex::from_world_xy_deck(Vec2::new(mid_x, SHIP_BEAM_M * 0.5), 4)
            .expect("midship");
        let row = midship_cross_section_row(deck, mid_idx.x);
        let starboard_cabin = row.starts_with("CCC");
        let port_cabin = row
            .rfind("CCC")
            .is_some_and(|i| i > row.find("###").unwrap_or(0));
        assert!(
            port_cabin && starboard_cabin,
            "expected port and starboard 3-wide cabin bands at midship x={mid_x}, got: {row}"
        );
    }

    #[test]
    fn midship_has_one_interior_spine() {
        let layouts = deck_cell_layouts(CELL_SIZE_M);
        let deck = deck_five(&layouts);
        let mid_x = midship_x_for_deck(4);
        let mid_idx = CellIndex::from_world_xy_deck(Vec2::new(mid_x, SHIP_BEAM_M * 0.5), 4)
            .expect("midship");
        let runs = corridor_y_runs_at_x(deck, mid_idx.x);
        let spine_runs: Vec<_> = runs.iter().filter(|run| run.len() >= 2).collect();
        assert_eq!(
            spine_runs.len(),
            1,
            "expected one centre spine at midship, got {spine_runs:?}"
        );
        assert!(
            spine_runs[0].len() >= 10,
            "centre spine should span most of the beam, got {:?}",
            spine_runs[0]
        );
    }

    #[test]
    fn spine_width_fills_beam() {
        let deck_index = 4;
        let mid_x = midship_x_for_deck(deck_index);
        let beam = effective_beam_m(deck_index, mid_x).expect("beam");
        let spine_w = spine_width_m(beam);
        let total = CABIN_COLUMNS_TOTAL_WIDTH_M + NUM_INTERIOR_SPINES as f32 * spine_w;
        assert!(
            (total - beam).abs() < 1.0,
            "expected 2×3m + spine ≈ beam ({total} vs {beam})"
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
        for along in 0..CABIN_LENGTH_CELLS {
            for across in 0..CABIN_WIDTH_CELLS {
                let coord = (
                    ox.saturating_add(along as u16),
                    oy.saturating_add(across as u16),
                );
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
        const FULL_MODULE_CELLS: usize = (CABIN_WIDTH_CELLS * CABIN_LENGTH_CELLS) as usize;
        for room in cabin_rooms {
            let cell_count = deck.iter_cells().filter(|(_, c)| c.room == room).count();
            if cell_count < FULL_MODULE_CELLS {
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
        let layouts = deck_cell_layouts(CELL_SIZE_M);
        let deck = deck_five(&layouts);
        const FULL_MODULE_CELLS: usize = (CABIN_WIDTH_CELLS * CABIN_LENGTH_CELLS) as usize;
        let mut by_room: HashMap<RoomId, Vec<PlanKey>> = HashMap::new();
        for (plan, cell) in deck.iter_cells() {
            if room_category(deck, cell.room) == RoomCategory::Cabin {
                by_room.entry(cell.room).or_default().push(plan);
            }
        }
        let mut found_module = false;
        for (room, coords) in &by_room {
            if coords.len() < FULL_MODULE_CELLS {
                continue;
            }
            if coords.len() > FULL_MODULE_CELLS {
                continue;
            }
            let (door, _) = room_window_and_door(deck, *room);
            if door.is_none() {
                continue;
            }
            let ox = coords.iter().map(|p| p.0).min().unwrap();
            let oy = coords.iter().map(|p| p.1).min().unwrap();
            if !module_borders_corridor(deck, ox, oy) {
                continue;
            }
            found_module = true;

            let mut interior_open = 0u32;
            let mut perimeter_with_wall = 0u32;
            for &(x, y) in coords {
                let cell = deck.get((x, y)).expect("cell");
                let (across_local, along_local) = cabin_local((x, y));
                if is_cabin_interior(across_local, along_local) {
                    assert!(
                        all_walls_open(cell),
                        "interior ({across_local},{along_local}) should have all Open walls"
                    );
                    interior_open += 1;
                } else if !all_walls_open(cell) {
                    perimeter_with_wall += 1;
                }
            }
            assert_eq!(interior_open, 4, "expected four interior cells");
            assert_eq!(
                perimeter_with_wall, 14,
                "expected fourteen perimeter cells with walls"
            );
            break;
        }
        let max_cabin_room = by_room
            .values()
            .map(|coords| coords.len())
            .max()
            .unwrap_or(0);
        assert!(
            found_module,
            "expected at least one full 3×6 cabin module bordering corridor on deck 5 (largest cabin room: {max_cabin_room} cells)"
        );

        let mut found_exterior = false;
        let mut found_interior = false;
        let mut cabin_rooms = HashSet::new();
        for (_, cell) in deck.iter_cells() {
            if room_category(deck, cell.room) == RoomCategory::Cabin {
                cabin_rooms.insert(cell.room);
            }
        }
        let occupied: HashSet<_> = deck.plan_keys().collect();
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
                let nb = neighbor_plan(window_coord, window_wall);
                assert!(
                    nb.is_none_or(|nb| !occupied.contains(&nb)),
                    "window must be on hull perimeter"
                );
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
*/
