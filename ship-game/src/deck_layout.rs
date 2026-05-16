//! Procedural deck grid and cell zoning shared by 3D and 2D ship views (+Y bow).

use crate::cell::{Cell, Material, RoomCatalog, RoomCategory, RoomId};
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
pub struct DeckLayouts(pub Vec<DeckCells>);

/// Precomputed corridors and striping bounds on deck seven (sea-facing cabin ring).
#[derive(Clone, Debug)]
pub struct DeckSevenCache {
    /// Occupied walkway cells painted white (`2 tiles wide`).
    pub corridor: HashSet<(i32, i32)>,
    pub star_ix_inner: i32,
    pub star_ix_outer: i32,
    pub port_ix_inner: i32,
    pub port_ix_outer: i32,
}

#[derive(Clone)]
pub struct DeckCells {
    pub cells: HashMap<(i32, i32), Cell>,
    pub rooms: RoomCatalog,
    /// Human-facing deck 7 (`DECK_SEVEN_INDEX`): exterior cabin striping + dual corridor.
    pub deck_seven_cache: Option<DeckSevenCache>,
}

impl DeckCells {
    pub fn centers(&self, step_m: f32) -> Vec<Vec2> {
        self.cells
            .keys()
            .map(|&(ix, iy)| Vec2::new(ix as f32 * step_m, iy as f32 * step_m))
            .collect()
    }

    pub fn perimeter(&self) -> HashSet<(i32, i32)> {
        let occupied: HashSet<_> = self.cells.keys().copied().collect();
        occupied
            .iter()
            .copied()
            .filter(|cell| is_perimeter_cell(*cell, &occupied))
            .collect()
    }

    pub fn cell_coords(p: Vec2) -> (i32, i32) {
        (
            (p.x / CELL_SIZE_M).round() as i32,
            (p.y / CELL_SIZE_M).round() as i32,
        )
    }

    pub fn cell_mut(&mut self, coord: (i32, i32)) -> Option<&mut Cell> {
        self.cells.get_mut(&coord)
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

fn exterior_wall_material(p: Vec2, is_perimeter: bool) -> Material {
    if is_perimeter && window_strip_zone(p) {
        Material::Window
    } else {
        Material::Hull
    }
}

fn zone_floor_material(zone: ZoneBucket) -> Material {
    match zone {
        ZoneBucket::HullEdge => Material::Hull,
        ZoneBucket::WindowStrip => Material::Window,
        ZoneBucket::InnerCabin | ZoneBucket::OuterCabin => Material::CabinPartition,
        ZoneBucket::PublicDeck => Material::PublicShell,
        ZoneBucket::DeckBase => Material::DeckBase,
    }
}

const CABIN_WIDTH_CELLS: i32 = 3;
const CABIN_LENGTH_CELLS: i32 = 6;
const CORRIDOR_WIDTH_CELLS: i32 = 2;

/// Cardinal neighbor offset for each wall (`wall1`..`wall4`).
const WALL_DELTAS: [(i32, i32); 4] = [(1, 0), (0, 1), (-1, 0), (0, -1)];

fn cabin_room_key(ix: i32, iy: i32) -> (i32, i32) {
    (
        ix.div_euclid(CABIN_WIDTH_CELLS),
        iy.div_euclid(CABIN_LENGTH_CELLS),
    )
}

fn cabin_local(ix: i32, iy: i32) -> (i32, i32) {
    (
        ix.rem_euclid(CABIN_WIDTH_CELLS),
        iy.rem_euclid(CABIN_LENGTH_CELLS),
    )
}

fn is_cabin_interior(lx: i32, ly: i32) -> bool {
    lx == 1 && (1..=4).contains(&ly)
}

fn room_category_at(
    catalog: &RoomCatalog,
    room_map: &HashMap<(i32, i32), RoomId>,
    coord: (i32, i32),
) -> Option<RoomCategory> {
    room_map
        .get(&coord)
        .and_then(|&id| catalog.category(id))
}

fn cabin_edge_wall(
    occupied: &HashSet<(i32, i32)>,
    from_room: RoomId,
    to: (i32, i32),
    room_map: &HashMap<(i32, i32), RoomId>,
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

fn insert_inboard_corridor_strip(
    corridor: &mut HashSet<(i32, i32)>,
    occupied: &HashSet<(i32, i32)>,
    ix_inner: i32,
    iy_min: i32,
    iy_max: i32,
    starboard: bool,
) {
    for iy in iy_min..=iy_max {
        for delta in 1..=CORRIDOR_WIDTH_CELLS {
            let ix = if starboard {
                ix_inner - delta
            } else {
                ix_inner + delta
            };
            if starboard && ix <= 0 {
                continue;
            }
            if !starboard && ix >= 0 {
                continue;
            }
            let c = (ix, iy);
            if occupied.contains(&c) {
                corridor.insert(c);
            }
        }
    }
}

/// Two-cell-wide inboard walkways beside port/starboard cabin stacks (all decks).
fn build_corridor_cells(centers: &[Vec2], occupied: &HashSet<(i32, i32)>) -> HashSet<(i32, i32)> {
    let perimeter: HashSet<_> = occupied
        .iter()
        .copied()
        .filter(|&cell| is_perimeter_cell(cell, occupied))
        .collect();

    let mut star_outer = Vec::<(i32, i32)>::new();
    let mut port_outer = Vec::<(i32, i32)>::new();
    let mut star_inner = Vec::<(i32, i32)>::new();
    let mut port_inner = Vec::<(i32, i32)>::new();

    for &p in centers {
        let cell = DeckCells::cell_coords(p);
        let zone = ZoneBucket::classify(p, perimeter.contains(&cell));
        match zone {
            ZoneBucket::OuterCabin => match cell.0.signum() {
                1 => star_outer.push(cell),
                -1 => port_outer.push(cell),
                _ => {}
            },
            ZoneBucket::InnerCabin => match cell.0.signum() {
                1 => star_inner.push(cell),
                -1 => port_inner.push(cell),
                _ => {}
            },
            _ => {}
        }
    }

    let mut corridor = HashSet::new();

    if !star_outer.is_empty() {
        let ix_inner = star_outer.iter().map(|t| t.0).min().unwrap();
        let iy_min = star_outer.iter().map(|t| t.1).min().unwrap();
        let iy_max = star_outer.iter().map(|t| t.1).max().unwrap();
        insert_inboard_corridor_strip(&mut corridor, occupied, ix_inner, iy_min, iy_max, true);
    }

    if !port_outer.is_empty() {
        let ix_inner = port_outer.iter().map(|t| t.0).max().unwrap();
        let iy_min = port_outer.iter().map(|t| t.1).min().unwrap();
        let iy_max = port_outer.iter().map(|t| t.1).max().unwrap();
        insert_inboard_corridor_strip(&mut corridor, occupied, ix_inner, iy_min, iy_max, false);
    }

    if !star_inner.is_empty() {
        let ix_inner = star_inner.iter().map(|t| t.0).min().unwrap();
        let iy_min = star_inner.iter().map(|t| t.1).min().unwrap();
        let iy_max = star_inner.iter().map(|t| t.1).max().unwrap();
        insert_inboard_corridor_strip(&mut corridor, occupied, ix_inner, iy_min, iy_max, true);
    }

    if !port_inner.is_empty() {
        let ix_inner = port_inner.iter().map(|t| t.0).max().unwrap();
        let iy_min = port_inner.iter().map(|t| t.1).min().unwrap();
        let iy_max = port_inner.iter().map(|t| t.1).max().unwrap();
        insert_inboard_corridor_strip(&mut corridor, occupied, ix_inner, iy_min, iy_max, false);
    }

    ensure_module_corridor_access(occupied, &mut corridor);

    corridor
}

/// Add 2-wide inboard strips for cabin modules that do not yet touch a corridor cell.
fn ensure_module_corridor_access(
    occupied: &HashSet<(i32, i32)>,
    corridor: &mut HashSet<(i32, i32)>,
) {
    let mut modules: HashMap<(i32, i32), Vec<(i32, i32)>> = HashMap::new();
    for &coord in occupied.iter() {
        if corridor.contains(&coord) {
            continue;
        }
        modules
            .entry(cabin_room_key(coord.0, coord.1))
            .or_default()
            .push(coord);
    }

    for cells in modules.values() {
        let touches_corridor = cells.iter().any(|&(ix, iy)| {
            WALL_DELTAS
                .iter()
                .any(|&(dx, dy)| corridor.contains(&(ix + dx, iy + dy)))
        });
        if touches_corridor {
            continue;
        }

        let starboard = cells[0].0 > 0;
        let mut by_iy: HashMap<i32, Vec<i32>> = HashMap::new();
        for &(ix, iy) in cells {
            by_iy.entry(iy).or_default().push(ix);
        }
        let toward = if starboard { -1 } else { 1 };
        for (iy, ixs) in by_iy {
            let anchor_ix = if starboard {
                *ixs.iter().min().unwrap()
            } else {
                *ixs.iter().max().unwrap()
            };
            for delta in 1..=CORRIDOR_WIDTH_CELLS {
                let c = (anchor_ix + toward * delta, iy);
                if occupied.contains(&c) {
                    corridor.insert(c);
                }
            }
        }
    }
}

fn corridor_floor_material(deck_index: usize) -> Material {
    if deck_index == DECK_SEVEN_INDEX {
        Material::CorridorWhite
    } else {
        Material::Corridor
    }
}

fn room_for_cabin(
    catalog: &mut RoomCatalog,
    cabin_rooms: &mut HashMap<(i32, i32), RoomId>,
    deck: u8,
    ix: i32,
    iy: i32,
) -> RoomId {
    *cabin_rooms
        .entry(cabin_room_key(ix, iy))
        .or_insert_with(|| catalog.insert("Cabin", deck, RoomCategory::Cabin))
}

fn room_for_zone(
    catalog: &mut RoomCatalog,
    cabin_rooms: &mut HashMap<(i32, i32), RoomId>,
    shared: &mut SharedDeckRooms,
    deck: u8,
    zone: ZoneBucket,
    ix: i32,
    iy: i32,
    amenity: Option<AmenityKind>,
) -> RoomId {
    if let Some(a) = amenity {
        let (name, category) = match a {
            AmenityKind::Theatre => ("Theatre", RoomCategory::Amenity),
            AmenityKind::MainDining => ("Main Dining", RoomCategory::Amenity),
            AmenityKind::Buffet => ("Buffet", RoomCategory::Amenity),
            AmenityKind::Pools => ("Pools", RoomCategory::Amenity),
            AmenityKind::Casino => ("Casino", RoomCategory::Amenity),
        };
        return catalog.insert(name, deck, category);
    }
    match zone {
        ZoneBucket::HullEdge | ZoneBucket::WindowStrip => *shared
            .exterior
            .get_or_insert_with(|| catalog.insert("Exterior", deck, RoomCategory::Exterior)),
        ZoneBucket::PublicDeck => *shared
            .public_deck
            .get_or_insert_with(|| catalog.insert("Public Deck", deck, RoomCategory::Public)),
        ZoneBucket::InnerCabin | ZoneBucket::OuterCabin => {
            let key = cabin_room_key(ix, iy);
            let name = if matches!(zone, ZoneBucket::InnerCabin) {
                "Inner Cabin"
            } else {
                "Outer Cabin"
            };
            *cabin_rooms
                .entry(key)
                .or_insert_with(|| catalog.insert(name, deck, RoomCategory::Cabin))
        }
        ZoneBucket::DeckBase => *shared
            .deck_base
            .get_or_insert_with(|| catalog.insert("Deck", deck, RoomCategory::Cabin)),
    }
}

struct SharedDeckRooms {
    exterior: Option<RoomId>,
    public_deck: Option<RoomId>,
    deck_base: Option<RoomId>,
}

fn floor_material_for_cell(
    deck_index: usize,
    p: Vec2,
    cell: (i32, i32),
    deck_seven_cache: Option<&DeckSevenCache>,
    zone: ZoneBucket,
    is_perimeter: bool,
) -> Material {
    if let Some(mat) =
        deck_seven_floor_material(deck_index, p, cell, deck_seven_cache, is_perimeter)
    {
        return mat;
    }
    if let Some(amenity) = amenity_overlay(deck_index, p) {
        return match amenity {
            AmenityKind::Theatre => Material::Theatre,
            AmenityKind::MainDining => Material::Dining,
            AmenityKind::Buffet => Material::Buffet,
            AmenityKind::Pools => Material::Pool,
            AmenityKind::Casino => Material::Casino,
        };
    }
    let _ = is_perimeter;
    zone_floor_material(zone)
}

fn assign_non_cabin_walls(deck: &mut DeckCells) {
    let occupied: HashSet<_> = deck.cells.keys().copied().collect();
    let room_map: HashMap<(i32, i32), RoomId> =
        deck.cells.iter().map(|(&c, cell)| (c, cell.room)).collect();
    let coords: Vec<(i32, i32)> = deck.cells.keys().copied().collect();
    for (ix, iy) in coords {
        let from_room = room_map[&(ix, iy)];
        if deck.rooms.category(from_room) == Some(RoomCategory::Cabin) {
            continue;
        }
        let cell = deck.cells.get_mut(&(ix, iy)).expect("cell");
        for (wi, &(dx, dy)) in WALL_DELTAS.iter().enumerate() {
            let wall = cabin_edge_wall(
                &occupied,
                from_room,
                (ix + dx, iy + dy),
                &room_map,
                &deck.rooms,
            );
            match wi {
                0 => cell.wall1 = wall,
                1 => cell.wall2 = wall,
                2 => cell.wall3 = wall,
                _ => cell.wall4 = wall,
            }
        }
    }
}

fn assign_cabin_module_walls(deck: &mut DeckCells) {
    let occupied: HashSet<_> = deck.cells.keys().copied().collect();
    let room_map: HashMap<(i32, i32), RoomId> =
        deck.cells.iter().map(|(&c, cell)| (c, cell.room)).collect();
    let coords: Vec<(i32, i32)> = deck.cells.keys().copied().collect();
    for (ix, iy) in coords {
        let from_room = room_map[&(ix, iy)];
        if deck.rooms.category(from_room) != Some(RoomCategory::Cabin) {
            continue;
        }
        let (lx, ly) = cabin_local(ix, iy);
        let cell = deck.cells.get_mut(&(ix, iy)).expect("cell");
        if is_cabin_interior(lx, ly) {
            cell.wall1 = Material::Open;
            cell.wall2 = Material::Open;
            cell.wall3 = Material::Open;
            cell.wall4 = Material::Open;
        } else {
            cell.wall1 = cabin_edge_wall(
                &occupied,
                from_room,
                (ix + 1, iy),
                &room_map,
                &deck.rooms,
            );
            cell.wall2 = cabin_edge_wall(
                &occupied,
                from_room,
                (ix, iy + 1),
                &room_map,
                &deck.rooms,
            );
            cell.wall3 = cabin_edge_wall(
                &occupied,
                from_room,
                (ix - 1, iy),
                &room_map,
                &deck.rooms,
            );
            cell.wall4 = cabin_edge_wall(
                &occupied,
                from_room,
                (ix, iy - 1),
                &room_map,
                &deck.rooms,
            );
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

fn neighbor_coord((ix, iy): (i32, i32), wall_idx: usize) -> (i32, i32) {
    let (dx, dy) = WALL_DELTAS[wall_idx];
    (ix + dx, iy + dy)
}

fn is_corridor_cell(
    coord: (i32, i32),
    corridor_cells: &HashSet<(i32, i32)>,
    catalog: &RoomCatalog,
    room_map: &HashMap<(i32, i32), RoomId>,
) -> bool {
    corridor_cells.contains(&coord)
        || room_category_at(catalog, room_map, coord) == Some(RoomCategory::Corridor)
}

fn assign_cabin_openings(deck: &mut DeckCells, corridor_cells: &HashSet<(i32, i32)>) {
    let room_map: HashMap<(i32, i32), RoomId> =
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

        let mut door_candidates: Vec<((i32, i32), usize)> = Vec::new();
        let mut window_candidates: Vec<((i32, i32), usize)> = Vec::new();
        let mut has_hull_edge = false;

        for &(ix, iy) in &module_cells {
            let (lx, ly) = cabin_local(ix, iy);
            if is_cabin_interior(lx, ly) {
                continue;
            }
            for wall_idx in 0..4 {
                let nb = neighbor_coord((ix, iy), wall_idx);
                let wall = wall_material(deck.cells.get(&(ix, iy)).expect("cell"), wall_idx);
                if wall == Material::MarinePanel
                    && is_corridor_cell(nb, corridor_cells, &deck.rooms, &room_map)
                {
                    door_candidates.push(((ix, iy), wall_idx));
                }
                if wall == Material::Hull && !occupied.contains(&nb) {
                    has_hull_edge = true;
                    window_candidates.push(((ix, iy), wall_idx));
                }
            }
        }

        let door = pick_door(&door_candidates, &module_cells);
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
}

fn pick_door(candidates: &[((i32, i32), usize)], module_cells: &[(i32, i32)]) -> Option<((i32, i32), usize)> {
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

fn door_candidate_score(((ix, iy), wall_idx): ((i32, i32), usize), module_cells: &[(i32, i32)]) -> i32 {
    let (lx, ly) = cabin_local(ix, iy);
    let mut score = 0;
    // Prefer middle of the corridor-facing edge.
    if lx == 1 || ly == 3 {
        score += 10;
    }
    // Prefer edges with more corridor neighbors along the face.
    let (dx, _dy) = WALL_DELTAS[wall_idx];
    let along = if dx != 0 { (0, 1) } else { (1, 0) };
    for &(mx, my) in module_cells {
        if mx != ix && my != iy {
            continue;
        }
        if (mx - ix, my - iy) == along || (mx - ix, my - iy) == (-along.0, -along.1) {
            score += 1;
        }
    }
    // Tie-break: closer to ship center.
    score - ix.unsigned_abs() as i32
}

fn is_opposite_cabin_end((ix1, iy1): (i32, i32), (ix2, iy2): (i32, i32)) -> bool {
    let (lx1, ly1) = cabin_local(ix1, iy1);
    let (lx2, ly2) = cabin_local(ix2, iy2);
    lx1 == 1
        && lx2 == 1
        && ((ly1 <= 1 && ly2 >= 4) || (ly1 >= 4 && ly2 <= 1))
}

fn pick_window(
    candidates: &[((i32, i32), usize)],
    door: Option<((i32, i32), usize)>,
) -> Option<((i32, i32), usize)> {
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

fn window_candidate_score(((ix, iy), _wall_idx): ((i32, i32), usize)) -> i32 {
    let (lx, ly) = cabin_local(ix, iy);
    let mut score = ix.unsigned_abs() as i32;
    if lx == 1 || ly == 3 {
        score += 10;
    }
    let _ = ly;
    score
}

/// All deck occupancy grids at `step_m` cell spacing.
pub fn deck_cell_layouts(step_m: f32) -> Vec<DeckCells> {
    let mut out = Vec::with_capacity(NUM_DECKS);
    for deck_i in 0..NUM_DECKS {
        let centers = fallback_deck_cell_centers(deck_i, step_m)
            .into_iter()
            .filter(|p| profile_allows_cell(deck_i, *p))
            .collect::<Vec<_>>();
        let occupied: HashSet<_> = centers
            .iter()
            .map(|&p| DeckCells::cell_coords(p))
            .collect();
        let corridor_cells = build_corridor_cells(&centers, &occupied);

        let mut rooms = RoomCatalog::default();
        let mut cabin_rooms = HashMap::new();
        let corridor_room = rooms.insert("Corridor", deck_i as u8, RoomCategory::Corridor);
        let corridor_floor = corridor_floor_material(deck_i);

        let mut cells = HashMap::new();
        for &p in &centers {
            let coord = DeckCells::cell_coords(p);
            if corridor_cells.contains(&coord) {
                cells.insert(coord, Cell::new(corridor_floor, corridor_room));
            } else {
                let room = room_for_cabin(&mut rooms, &mut cabin_rooms, deck_i as u8, coord.0, coord.1);
                cells.insert(coord, Cell::new(Material::CabinPartition, room));
            }
        }

        let deck_seven_cache = if corridor_cells.is_empty() {
            None
        } else {
            Some(DeckSevenCache {
                corridor: corridor_cells.clone(),
                star_ix_inner: 0,
                star_ix_outer: 0,
                port_ix_inner: 0,
                port_ix_outer: 0,
            })
        };

        let mut deck = DeckCells {
            cells,
            rooms,
            deck_seven_cache,
        };
        assign_cabin_module_walls(&mut deck);
        assign_non_cabin_walls(&mut deck);
        assign_cabin_openings(&mut deck, &corridor_cells);
        out.push(deck);
    }
    out
}

/// Alias retained for callers migrating from the old name.
pub fn deck_layouts(step_m: f32) -> Vec<DeckCells> {
    deck_cell_layouts(step_m)
}

fn is_perimeter_cell(cell: (i32, i32), occupied: &HashSet<(i32, i32)>) -> bool {
    for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
        if !occupied.contains(&(cell.0 + dx, cell.1 + dy)) {
            return true;
        }
    }
    false
}

/// Human-facing passenger **deck seven** (“Main deck” tier). **0-based** layout index [`DECK_SEVEN_INDEX`].
pub const DECK_SEVEN_INDEX: usize = 6;

const DECK_SEV_CABINX: i32 = 3;
/// Target cabin module depth along the ship (±Y metres); reused for bow banding.
const DECK_SEV_CABINY: i32 = 8;

/// Forward dense interior cabin wedge at the bow (plan view), Centreline-heavy.
#[inline]
fn deck_seven_bow_interior_geom(p: Vec2) -> bool {
    p.y > SHIP_LENGTH_M * 0.195 && p.y < SHIP_LENGTH_M * 0.46 && p.x.abs() < SHIP_BEAM_M * 0.29
}

fn deck_seven_bow_paint_allowed(raw: ZoneBucket, p: Vec2) -> bool {
    if matches!(raw, ZoneBucket::WindowStrip | ZoneBucket::PublicDeck) {
        return false;
    }
    if !deck_seven_bow_interior_geom(p) {
        return false;
    }
    match raw {
        ZoneBucket::InnerCabin | ZoneBucket::DeckBase => true,
        ZoneBucket::HullEdge => p.x.abs() < SHIP_BEAM_M * 0.32,
        ZoneBucket::OuterCabin => p.x.abs() < SHIP_BEAM_M * 0.34,
        _ => false,
    }
}

fn build_deck_seven_cache_inner(
    centers: &[Vec2],
    perimeter: &HashSet<(i32, i32)>,
    occupied: &HashSet<(i32, i32)>,
) -> Option<DeckSevenCache> {
    let mut star = Vec::<(i32, i32)>::new();
    let mut port = Vec::<(i32, i32)>::new();
    for &p in centers {
        let cell = (
            (p.x / CELL_SIZE_M).round() as i32,
            (p.y / CELL_SIZE_M).round() as i32,
        );
        if ZoneBucket::classify(p, perimeter.contains(&cell)) != ZoneBucket::OuterCabin {
            continue;
        }
        match cell.0.signum() {
            1 => star.push(cell),
            -1 => port.push(cell),
            _ => {}
        }
    }

    if star.is_empty() && port.is_empty() {
        return None;
    }

    let (star_ix_inner, star_ix_outer, star_iy_min, star_iy_max) = if star.is_empty() {
        (1, 0, 0, -1)
    } else {
        let xi_in = star.iter().map(|t| t.0).min().unwrap();
        let xi_out = star.iter().map(|t| t.0).max().unwrap();
        let yi_lo = star.iter().map(|t| t.1).min().unwrap();
        let yi_hi = star.iter().map(|t| t.1).max().unwrap();
        (xi_in, xi_out, yi_lo, yi_hi)
    };

    let (port_ix_inner, port_ix_outer, port_iy_min, port_iy_max) = if port.is_empty() {
        (-1, 0, 0, -1)
    } else {
        let xi_in = port.iter().map(|t| t.0).max().unwrap();
        let xi_out = port.iter().map(|t| t.0).min().unwrap();
        let yi_lo = port.iter().map(|t| t.1).min().unwrap();
        let yi_hi = port.iter().map(|t| t.1).max().unwrap();
        (xi_in, xi_out, yi_lo, yi_hi)
    };

    let mut corridor = HashSet::<(i32, i32)>::new();

    if !star.is_empty() && star_ix_inner <= star_ix_outer && star_iy_min <= star_iy_max {
        for iy in star_iy_min..=star_iy_max {
            for delta in [1_i32, 2] {
                let ix = star_ix_inner - delta;
                if ix <= 0 {
                    continue;
                }
                let c = (ix, iy);
                if occupied.contains(&c) {
                    corridor.insert(c);
                }
            }
        }
    }

    if !port.is_empty() && port_ix_outer <= port_ix_inner && port_iy_min <= port_iy_max {
        for iy in port_iy_min..=port_iy_max {
            for delta in [1_i32, 2] {
                let ix = port_ix_inner.saturating_add(delta);
                if ix >= 0 {
                    continue;
                }
                let c = (ix, iy);
                if occupied.contains(&c) {
                    corridor.insert(c);
                }
            }
        }
    }

    Some(DeckSevenCache {
        corridor,
        star_ix_inner,
        star_ix_outer,
        port_ix_inner,
        port_ix_outer,
    })
}

fn deck_seven_floor_material(
    deck_index: usize,
    p: Vec2,
    cell: (i32, i32),
    cache: Option<&DeckSevenCache>,
    is_perimeter: bool,
) -> Option<Material> {
    if deck_index != DECK_SEVEN_INDEX {
        return None;
    }
    let raw = ZoneBucket::classify(p, is_perimeter);

    if cache.is_some_and(|c| c.corridor.contains(&cell)) {
        return Some(Material::CorridorWhite);
    }

    if deck_seven_bow_paint_allowed(raw, p) {
        let alt_bow = cell.1.div_euclid(DECK_SEV_CABINY).rem_euclid(2) == 0;
        return Some(if alt_bow {
            Material::BowAccent
        } else {
            Material::CabinStripeB
        });
    }

    let cache = cache?;

    if matches!(raw, ZoneBucket::HullEdge | ZoneBucket::OuterCabin)
        && outer_cabin_zone(p)
        && !window_strip_zone(p)
    {
        if cell.0 > 0 && cache.star_ix_inner <= cache.star_ix_outer {
            if let Some(is_blue_strip) = starboard_strip_is_blue(cell.0, cache) {
                return Some(exterior_material_from_blue(is_blue_strip));
            }
        }
        if cell.0 < 0 && cache.port_ix_outer <= cache.port_ix_inner {
            if let Some(is_blue_strip) = portside_strip_is_blue(cell.0, cache) {
                return Some(exterior_material_from_blue(is_blue_strip));
            }
        }
    }

    None
}

#[inline]
fn exterior_material_from_blue(is_blue_strip: bool) -> Material {
    if is_blue_strip {
        Material::CabinStripeA
    } else {
        Material::CabinStripeB
    }
}

/// Starboard façade (`ix > 0`): map hull perimeter columns to the same alternating beam bands as inboard cabins.
#[inline]
fn starboard_strip_is_blue(ix: i32, cache: &DeckSevenCache) -> Option<bool> {
    if ix <= 0 || cache.star_ix_inner > cache.star_ix_outer {
        return None;
    }
    if ix >= cache.star_ix_inner {
        let cab_col = (ix - cache.star_ix_inner).div_euclid(DECK_SEV_CABINX);
        return Some(cab_col.rem_euclid(2) == 0);
    }
    None
}

#[inline]
fn portside_strip_is_blue(ix: i32, cache: &DeckSevenCache) -> Option<bool> {
    if ix >= 0 || cache.port_ix_outer > cache.port_ix_inner {
        return None;
    }
    if ix <= cache.port_ix_inner {
        let cab_col = (cache.port_ix_inner - ix).div_euclid(DECK_SEV_CABINX);
        return Some(cab_col.rem_euclid(2) == 0);
    }
    None
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ZoneBucket {
    HullEdge,
    WindowStrip,
    InnerCabin,
    OuterCabin,
    PublicDeck,
    DeckBase,
}

impl ZoneBucket {
    /// Mirrors the classification order in legacy `setup`. Callers pass a precomputed perimeter
    /// flag (see [`DeckCells::perimeter`]) so this stays branchy but allocation-free.
    fn classify(c: Vec2, is_perimeter: bool) -> Self {
        if is_perimeter {
            return if window_strip_zone(c) {
                ZoneBucket::WindowStrip
            } else {
                ZoneBucket::HullEdge
            };
        }
        if inner_cabin_zone(c) {
            return ZoneBucket::InnerCabin;
        }
        if outer_cabin_zone(c) {
            return ZoneBucket::OuterCabin;
        }
        if c.y < -SHIP_LENGTH_M * 0.28 {
            return ZoneBucket::PublicDeck;
        }
        ZoneBucket::DeckBase
    }
}

/// Closed set of amenity overlays painted on top of base bucket colours.
#[derive(Clone, Copy)]
pub enum AmenityKind {
    Theatre,
    MainDining,
    Buffet,
    Pools,
    Casino,
}

impl AmenityKind {
    pub const COUNT: usize = 5;
    pub const ALL: [Self; Self::COUNT] = [
        Self::Theatre,
        Self::MainDining,
        Self::Buffet,
        Self::Pools,
        Self::Casino,
    ];

    pub fn idx(self) -> usize {
        self as usize
    }

    pub fn color(self) -> Color {
        match self {
            Self::Theatre => Color::srgb(0.52, 0.36, 0.68),
            Self::MainDining => Color::srgb(0.82, 0.48, 0.34),
            Self::Buffet => Color::srgb(0.42, 0.70, 0.50),
            Self::Pools => Color::srgb(0.32, 0.74, 0.88),
            Self::Casino => Color::srgb(0.72, 0.22, 0.42),
        }
    }
}

/// Optional axis-aligned amenity overlay on top of base bucket colours (restaurant, pool, theatre, …).
pub fn amenity_overlay(deck_index: usize, p: Vec2) -> Option<AmenityKind> {
    // Forward theatre / aqua show — decks 5–10, centreline-forward
    if (5..=10).contains(&deck_index)
        && p.x.abs() < 17.5
        && p.y > SHIP_LENGTH_M * 0.36
        && p.y < SHIP_LENGTH_M * 0.49
    {
        return Some(AmenityKind::Theatre);
    }
    // Main dining room — mid-aft hull
    if (4..=7).contains(&deck_index)
        && p.x.abs() < 17.5
        && p.y > -SHIP_LENGTH_M * 0.36
        && p.y < -SHIP_LENGTH_M * 0.10
    {
        return Some(AmenityKind::MainDining);
    }
    // Buffet / speciality restaurant strip — upper lido bays
    if (8..=12).contains(&deck_index)
        && p.x.abs() > 9.0
        && p.x.abs() < 27.5
        && p.y > SHIP_LENGTH_M * 0.04
        && p.y < SHIP_LENGTH_M * 0.30
    {
        return Some(AmenityKind::Buffet);
    }
    // Pool / sports — open forward-upper
    if (11..=16).contains(&deck_index)
        && p.y > SHIP_LENGTH_M * 0.16
        && p.x.abs() < SHIP_BEAM_M * 0.36
    {
        return Some(AmenityKind::Pools);
    }
    // Casino / nightclub — outboard promenade band
    if (6..=9).contains(&deck_index)
        && p.y > -SHIP_LENGTH_M * 0.06
        && p.y < SHIP_LENGTH_M * 0.14
        && p.x.abs() > 19.0
    {
        return Some(AmenityKind::Casino);
    }
    None
}

#[cfg(test)]
mod layout_tests {
    use super::*;
    use crate::cell::Material;

    #[test]
    fn deck_cell_layouts_builds_walls_and_floors() {
        let decks = deck_cell_layouts(CELL_SIZE_M);
        let deck = &decks[4];
        assert!(!deck.cells.is_empty());
        let cell = deck.cells.values().next().expect("cell");
        assert_ne!(cell.floor, Material::Open);
    }

    #[test]
    fn deck_five_has_no_lattice_holes_anywhere() {
        let deck = &deck_cell_layouts(CELL_SIZE_M)[4];
        let occupied: std::collections::HashSet<_> = deck.cells.keys().copied().collect();
        for &(ix, iy) in &occupied {
            for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                let nb = (ix + dx, iy + dy);
                if occupied.contains(&(ix + 2 * dx, iy + 2 * dy)) && !occupied.contains(&nb) {
                    panic!(
                        "deck 5 lattice hole: {nb:?} between ({ix},{iy}) and {:?}",
                        (ix + 2 * dx, iy + 2 * dy)
                    );
                }
            }
        }
    }

    fn all_walls_open(cell: &Cell) -> bool {
        [cell.wall1, cell.wall2, cell.wall3, cell.wall4]
            .iter()
            .all(|&w| w == Material::Open)
    }

    fn is_full_cabin_module(deck: &DeckCells, ox: i32, oy: i32) -> Option<RoomId> {
        let anchor = deck.cells.get(&(ox, oy))?;
        let room = anchor.room;
        for lx in 0..CABIN_WIDTH_CELLS {
            for ly in 0..CABIN_LENGTH_CELLS {
                let coord = (ox + lx, oy + ly);
                let cell = deck.cells.get(&coord)?;
                if cell.room != room {
                    return None;
                }
            }
        }
        Some(room)
    }

    fn room_category(deck: &DeckCells, room: RoomId) -> RoomCategory {
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

    fn neighbor_is_corridor(deck: &DeckCells, coord: (i32, i32), wall_idx: usize) -> bool {
        let nb = neighbor_coord(coord, wall_idx);
        deck.cells.get(&nb).is_some_and(|c| {
            room_category(deck, c.room) == RoomCategory::Corridor
        })
    }

    fn module_has_hull_edge(deck: &DeckCells, ox: i32, oy: i32) -> bool {
        let occupied: HashSet<_> = deck.cells.keys().copied().collect();
        for lx in 0..CABIN_WIDTH_CELLS {
            for ly in 0..CABIN_LENGTH_CELLS {
                let (ix, iy) = (ox + lx, oy + ly);
                for wall_idx in 0..4 {
                    let nb = neighbor_coord((ix, iy), wall_idx);
                    if !occupied.contains(&nb) {
                        return true;
                    }
                }
            }
        }
        false
    }

    #[test]
    fn deck_five_has_corridor_rooms() {
        let deck = &deck_cell_layouts(CELL_SIZE_M)[4];
        let corridor_cells: Vec<_> = deck
            .cells
            .iter()
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

    #[test]
    fn corridor_is_two_tiles_wide_where_geometry_allows() {
        let centers: Vec<_> = fallback_deck_cell_centers(4, CELL_SIZE_M)
            .into_iter()
            .filter(|p| profile_allows_cell(4, *p))
            .collect();
        let occupied: HashSet<_> = centers.iter().map(|p| DeckCells::cell_coords(*p)).collect();
        let corridor = build_corridor_cells(&centers, &occupied);
        assert!(!corridor.is_empty());

        let mut star_by_iy: HashMap<i32, Vec<i32>> = HashMap::new();
        let mut port_by_iy: HashMap<i32, Vec<i32>> = HashMap::new();
        for &(ix, iy) in &corridor {
            if ix > 0 {
                star_by_iy.entry(iy).or_default().push(ix);
            } else if ix < 0 {
                port_by_iy.entry(iy).or_default().push(ix);
            }
        }
        for iy_map in [&star_by_iy, &port_by_iy] {
            for cols in iy_map.values() {
                if cols.len() >= 2 {
                    let mut sorted = cols.clone();
                    sorted.sort_unstable();
                    let width = sorted.last().unwrap() - sorted.first().unwrap() + 1;
                    assert!(
                        width >= 2,
                        "expected at least two corridor columns per side, got {sorted:?}"
                    );
                }
            }
        }
    }

    fn module_borders_corridor(deck: &DeckCells, ox: i32, oy: i32) -> bool {
        for lx in 0..CABIN_WIDTH_CELLS {
            for ly in 0..CABIN_LENGTH_CELLS {
                let coord = (ox + lx, oy + ly);
                for wall_idx in 0..4 {
                    if neighbor_is_corridor(deck, coord, wall_idx) {
                        return true;
                    }
                }
            }
        }
        false
    }

    fn cabin_room_has_door(deck: &DeckCells, room: RoomId) -> bool {
        deck.cells.values().any(|cell| {
            cell.room == room && cell_has_door(cell).is_some()
        })
    }

    #[test]
    fn every_cabin_room_has_door_to_corridor() {
        let deck = &deck_cell_layouts(CELL_SIZE_M)[4];
        let mut cabin_rooms = HashSet::new();
        for cell in deck.cells.values() {
            if room_category(deck, cell.room) == RoomCategory::Cabin {
                cabin_rooms.insert(cell.room);
            }
        }
        for room in cabin_rooms {
            assert!(
                cabin_room_has_door(deck, room),
                "cabin room {:?} should have a door",
                room
            );
        }
    }

    fn room_has_hull_edge(deck: &DeckCells, room: RoomId) -> bool {
        let occupied: HashSet<_> = deck.cells.keys().copied().collect();
        deck.cells.iter().any(|(&(ix, iy), cell)| {
            if cell.room != room {
                return false;
            }
            WALL_DELTAS.iter().any(|&(dx, dy)| !occupied.contains(&(ix + dx, iy + dy)))
        })
    }

    fn room_window_and_door(deck: &DeckCells, room: RoomId) -> (Option<((i32, i32), usize)>, Option<((i32, i32), usize)>) {
        let mut door = None;
        let mut window = None;
        for (&coord, cell) in &deck.cells {
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
        let mut found_full = false;
        for &(ix, iy) in deck.cells.keys() {
            let ox = ix.div_euclid(CABIN_WIDTH_CELLS) * CABIN_WIDTH_CELLS;
            let oy = iy.div_euclid(CABIN_LENGTH_CELLS) * CABIN_LENGTH_CELLS;
            if (ix, iy) != (ox, oy) {
                continue;
            }
            let Some(room) = is_full_cabin_module(deck, ox, oy) else {
                continue;
            };
            if room_category(deck, room) != RoomCategory::Cabin {
                continue;
            }
            if !module_borders_corridor(deck, ox, oy) {
                continue;
            }
            found_full = true;

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
            assert_eq!(perimeter_with_wall, 14, "expected fourteen perimeter cells with walls");
        }
        assert!(
            found_full,
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
                assert!(
                    !occupied.contains(&nb),
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
        assert!(found_exterior, "expected at least one exterior cabin room on deck 5");
        assert!(found_interior, "expected at least one interior cabin room on deck 5");
    }
}
