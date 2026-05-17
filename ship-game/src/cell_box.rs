//! Fixed 360×60×20 cell volume for the ship.
//!
//! Grid axes (indices are inclusive from 0):
//! - **X** (0..360): stern → bow; port-stern corner has the lowest X.
//! - **Y** (0..60): port → starboard; port-stern corner has the lowest Y.
//! - **Z** (0..20): deck number increases upward.
//!
//! [`CellIndex::PORT_STERN`] is `(0, 0, 0)` on the lowest deck.

use crate::cell::{cardinal_step_allowed, Cell};
use crate::ship_hull::{SHIP_BEAM_M, SHIP_LENGTH_M};

/// Legacy layout used 1 m cells in centred world coordinates before [`CellBox`].
const LEGACY_CELL_SIZE_M: f32 = 1.0;
use std::collections::HashMap;

/// Plan-view cell key `(x along ship, y across beam)` without deck.
pub type PlanKey = (u16, u16);

/// Cardinal neighbor offset for each wall (`wall1`..`wall4`) in grid space:
/// +X stern→bow, +Y port→starboard.
pub const WALL_DELTAS: [(i32, i32); 4] = [(1, 0), (0, 1), (-1, 0), (0, -1)];

/// Cells along the ship length (stern → bow).
pub const LENGTH: usize = 360;
/// Cells across the beam (port → starboard).
pub const BEAM: usize = 60;
/// Simulated deck count (bottom → top).
pub const DECKS: usize = 20;

const VOLUME: usize = LENGTH * BEAM * DECKS;

/// Grid cell address in [`CellBox`].
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub struct CellIndex {
    /// Stern (0) → bow (`LENGTH - 1`).
    pub x: u16,
    /// Port (0) → starboard (`BEAM - 1`).
    pub y: u16,
    /// Lowest deck (0) → top (`DECKS - 1`).
    pub z: u8,
}

impl CellIndex {
    /// Port-stern corner on the lowest deck.
    pub const PORT_STERN: Self = Self { x: 0, y: 0, z: 0 };

    /// Returns `None` when any component is out of range.
    #[must_use]
    pub fn new(x: u16, y: u16, z: u8) -> Option<Self> {
        let idx = Self { x, y, z };
        idx.is_valid().then_some(idx)
    }

    #[must_use]
    pub fn is_valid(self) -> bool {
        (self.x as usize) < LENGTH && (self.y as usize) < BEAM && (self.z as usize) < DECKS
    }

    #[must_use]
    pub fn linear_index(self) -> usize {
        debug_assert!(self.is_valid());
        self.z as usize * LENGTH * BEAM + self.y as usize * LENGTH + self.x as usize
    }

    #[must_use]
    pub fn from_linear_index(i: usize) -> Option<Self> {
        if i >= VOLUME {
            return None;
        }
        let z = (i / (LENGTH * BEAM)) as u8;
        let rem = i % (LENGTH * BEAM);
        let y = (rem / LENGTH) as u16;
        let x = (rem % LENGTH) as u16;
        Some(Self { x, y, z })
    }

    /// World metres: **X** = port→starboard, **Y** = stern→bow (Bevy plan / hull space).
    #[must_use]
    pub fn from_world_xy_deck(world: bevy::prelude::Vec2, deck: u8) -> Option<Self> {
        let x = world_to_grid_x(world.y);
        let y = world_to_grid_y(world.x);
        Self::new(x, y, deck)
    }

    /// Centre of this cell in world plan metres (hull space).
    #[must_use]
    pub fn to_world_xy(self) -> bevy::prelude::Vec2 {
        bevy::prelude::Vec2::new(grid_y_to_world_x(self.y), grid_x_to_world_y(self.x))
    }

    #[must_use]
    pub fn plan(self) -> PlanKey {
        (self.x, self.y)
    }

    #[must_use]
    pub fn with_plan(deck: u8, plan: PlanKey) -> Option<Self> {
        Self::new(plan.0, plan.1, deck)
    }

    /// World metres from legacy centred indices (`ix` = world X, `iy` = world Y at 1 m pitch).
    #[must_use]
    pub fn from_legacy_xy_deck(ix: i32, iy: i32, deck: u8) -> Option<Self> {
        let world = bevy::prelude::Vec2::new(
            ix as f32 * LEGACY_CELL_SIZE_M,
            iy as f32 * LEGACY_CELL_SIZE_M,
        );
        Self::from_world_xy_deck(world, deck)
    }

    #[must_use]
    pub fn offset(self, dx: i32, dy: i32, dz: i32) -> Option<Self> {
        let x = self.x as i32 + dx;
        let y = self.y as i32 + dy;
        let z = self.z as i32 + dz;
        if x < 0 || y < 0 || z < 0 {
            return None;
        }
        Self::new(x as u16, y as u16, z as u8)
    }

    #[must_use]
    pub fn neighbor(self, wall_idx: usize) -> Option<Self> {
        let (dx, dy) = WALL_DELTAS[wall_idx];
        self.offset(dx, dy, 0)
    }
}

/// Dense 360×60×20 storage; empty slots are `None`.
#[derive(Clone, Debug)]
pub struct CellBox {
    cells: Box<[Option<Cell>]>,
}

impl Default for CellBox {
    fn default() -> Self {
        Self::new()
    }
}

impl CellBox {
    #[must_use]
    pub fn new() -> Self {
        let cells = (0..VOLUME)
            .map(|_| None)
            .collect::<Vec<_>>()
            .into_boxed_slice();
        Self { cells }
    }

    #[must_use]
    pub fn volume() -> usize {
        VOLUME
    }

    #[must_use]
    pub fn get(&self, index: CellIndex) -> Option<&Cell> {
        if !index.is_valid() {
            return None;
        }
        self.cells[index.linear_index()].as_ref()
    }

    #[must_use]
    pub fn get_mut(&mut self, index: CellIndex) -> Option<&mut Cell> {
        if !index.is_valid() {
            return None;
        }
        self.cells[index.linear_index()].as_mut()
    }

    pub fn insert(&mut self, index: CellIndex, cell: Cell) -> Option<Cell> {
        if !index.is_valid() {
            return None;
        }
        self.cells[index.linear_index()].replace(cell)
    }

    pub fn remove(&mut self, index: CellIndex) -> Option<Cell> {
        if !index.is_valid() {
            return None;
        }
        self.cells[index.linear_index()].take()
    }

    #[must_use]
    pub fn contains(&self, index: CellIndex) -> bool {
        self.get(index).is_some()
    }

    pub fn iter_occupied(&self) -> impl Iterator<Item = (CellIndex, &Cell)> + '_ {
        self.cells.iter().enumerate().filter_map(|(i, cell)| {
            let cell = cell.as_ref()?;
            let index = CellIndex::from_linear_index(i)?;
            Some((index, cell))
        })
    }

    pub fn iter_occupied_mut(&mut self) -> impl Iterator<Item = (CellIndex, &mut Cell)> + '_ {
        self.cells.iter_mut().enumerate().filter_map(|(i, cell)| {
            let cell = cell.as_mut()?;
            let index = CellIndex::from_linear_index(i)?;
            Some((index, cell))
        })
    }

    pub fn iter_deck(&self, deck: u8) -> impl Iterator<Item = (CellIndex, &Cell)> + '_ {
        self.iter_occupied().filter(move |(idx, _)| idx.z == deck)
    }

    #[must_use]
    pub fn deck_occupied(&self, deck: u8) -> usize {
        self.iter_deck(deck).count()
    }
}

/// True when a cardinal step on deck `z` exists and every crossed edge is passable.
pub fn step_allowed(box_: &CellBox, index: CellIndex, dx: i32, dy: i32) -> bool {
    let Some(from) = box_.get(index) else {
        return false;
    };
    if dx == 0 && dy == 0 {
        return false;
    }
    if dx == 0 || dy == 0 {
        let Some(to_idx) = index.offset(dx, dy, 0) else {
            return false;
        };
        let Some(to) = box_.get(to_idx) else {
            return false;
        };
        return cardinal_step_allowed(from, to, dx, dy);
    }
    let c1 = (dx.signum(), 0);
    let c2 = (0, dy.signum());
    let Some(mid1_idx) = index.offset(c1.0, c1.1, 0) else {
        return false;
    };
    let Some(mid2_idx) = index.offset(c2.0, c2.1, 0) else {
        return false;
    };
    let Some(mid1) = box_.get(mid1_idx) else {
        return false;
    };
    let Some(mid2) = box_.get(mid2_idx) else {
        return false;
    };
    cardinal_step_allowed(from, mid1, c1.0, c1.1) && cardinal_step_allowed(from, mid2, c2.0, c2.1)
}

/// Walkable cell centres for one deck (world plan metres).
pub fn deck_walk_grid(box_: &CellBox, deck: u8) -> HashMap<PlanKey, bevy::prelude::Vec2> {
    box_.iter_deck(deck)
        .map(|(idx, _)| (idx.plan(), idx.to_world_xy()))
        .collect()
}

/// Metres per cell along the hull length axis (grid X).
#[must_use]
pub fn length_cell_m() -> f32 {
    SHIP_LENGTH_M / LENGTH as f32
}

/// Metres per cell across the beam (grid Y).
#[must_use]
pub fn beam_cell_m() -> f32 {
    SHIP_BEAM_M / BEAM as f32
}

#[must_use]
fn grid_x_to_world_y(x: u16) -> f32 {
    -SHIP_LENGTH_M * 0.5 + (x as f32 + 0.5) * length_cell_m()
}

#[must_use]
fn grid_y_to_world_x(y: u16) -> f32 {
    -SHIP_BEAM_M * 0.5 + (y as f32 + 0.5) * beam_cell_m()
}

#[must_use]
fn world_to_grid_x(world_y: f32) -> u16 {
    let t = (world_y + SHIP_LENGTH_M * 0.5) / length_cell_m();
    (t.floor() as i32).clamp(0, LENGTH as i32 - 1) as u16
}

#[must_use]
fn world_to_grid_y(world_x: f32) -> u16 {
    let t = (world_x + SHIP_BEAM_M * 0.5) / beam_cell_m();
    (t.floor() as i32).clamp(0, BEAM as i32 - 1) as u16
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cell::{Material, RoomId};
    use bevy::prelude::Vec2;

    #[test]
    fn port_stern_is_origin() {
        assert_eq!(CellIndex::PORT_STERN, CellIndex { x: 0, y: 0, z: 0 });
    }

    #[test]
    fn linear_index_round_trip() {
        let idx = CellIndex::new(359, 59, 19).unwrap();
        assert_eq!(CellIndex::from_linear_index(idx.linear_index()), Some(idx));
    }

    #[test]
    fn world_maps_port_stern_to_grid_origin() {
        let world = Vec2::new(-SHIP_BEAM_M * 0.5, -SHIP_LENGTH_M * 0.5);
        let idx = CellIndex::from_world_xy_deck(world, 0).unwrap();
        assert_eq!(idx, CellIndex::PORT_STERN);
    }

    #[test]
    fn adjacent_length_cells_share_an_edge() {
        let a = CellIndex::new(10, 30, 0).unwrap();
        let b = CellIndex::new(11, 30, 0).unwrap();
        let half = length_cell_m() * 0.5;
        let gap = (b.to_world_xy().y - half) - (a.to_world_xy().y + half);
        assert!(gap.abs() < 1e-5, "gap along stern→bow: {gap}");
    }

    #[test]
    fn legacy_one_metre_lattice_skips_length_indices() {
        let mut prev: Option<u16> = None;
        let mut skips = 0u32;
        for iy in -170..170 {
            let Some(idx) = CellIndex::from_legacy_xy_deck(0, iy, 0) else {
                continue;
            };
            if let Some(p) = prev {
                if idx.x > p + 1 {
                    skips += 1;
                }
            }
            prev = Some(idx.x);
        }
        assert!(skips > 0, "expected legacy lattice to skip grid X indices");
    }

    #[test]
    fn insert_and_get() {
        let mut box_ = CellBox::new();
        let idx = CellIndex::new(10, 20, 3).unwrap();
        let cell = Cell::new(Material::DeckBase, RoomId(0));
        box_.insert(idx, cell);
        assert!(box_.contains(idx));
        assert_eq!(box_.get(idx).unwrap().floor, Material::DeckBase);
    }
}
