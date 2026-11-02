//! Sea-cell grid types: floor/side materials, fixtures, entities, and per-cell geometry.

use bevy::prelude::*;
#[cfg(test)]
use std::collections::HashMap;

/// Floor surface material for a cell.
#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, Default)]
pub enum FloorMaterial {
    #[default]
    Carpet = 0,
    Wood = 1,
}

impl FloorMaterial {
    pub const COUNT: usize = 2;

    pub const ALL: [Self; Self::COUNT] = [Self::Carpet, Self::Wood];

    pub fn idx(self) -> usize {
        self as usize
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Carpet => "Carpet",
            Self::Wood => "Wood",
        }
    }

    pub fn color(self) -> Color {
        match self {
            Self::Carpet => Color::srgb(0.55, 0.55, 0.58),
            Self::Wood => Color::srgb(0.45, 0.32, 0.22),
        }
    }

    /// Floor tint for 2D plan view.
    pub fn plan_floor_color(self, _deck_index: usize) -> Color {
        self.color()
    }
}

/// Edge material between this cell and its neighbour (+X, +Y, −X, −Y).
#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, Default)]
pub enum SideMaterial {
    #[default]
    Open = 0,
    MarinePanel = 1,
    Door = 2,
    Window = 3,
}

impl SideMaterial {
    pub const COUNT: usize = 4;

    pub const ALL: [Self; Self::COUNT] = [Self::Open, Self::MarinePanel, Self::Door, Self::Window];

    pub fn idx(self) -> usize {
        self as usize
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Open => "Open",
            Self::MarinePanel => "Marine panel",
            Self::Door => "Door",
            Self::Window => "Window",
        }
    }

    /// Edges agents may cross when both sides are passable.
    pub fn is_passable(self) -> bool {
        matches!(self, Self::Open)
    }

    /// 2D plan / 3D fine-LOD stroke colour for non-open sides.
    pub fn plan_stroke_color(self) -> Color {
        match self {
            Self::Open => Color::srgb(0.12, 0.14, 0.18),
            Self::MarinePanel => Color::srgb(0.0, 0.0, 0.0),
            Self::Door => Color::srgb(0.55, 0.95, 0.55),
            Self::Window => Color::srgb(0.55, 0.85, 1.0),
        }
    }

    /// Editor picker swatch (marine panel matches historical brown).
    pub fn picker_color(self) -> Color {
        match self {
            Self::Open => Color::srgb(0.12, 0.14, 0.18),
            Self::MarinePanel => Color::srgb(0.62, 0.52, 0.44),
            Self::Door => Color::srgb(0.55, 0.95, 0.55),
            Self::Window => Color::srgb(0.55, 0.85, 1.0),
        }
    }

    pub fn draws_plan_stroke(self) -> bool {
        !matches!(self, Self::Open)
    }
}

/// True when agents may cross the edge between two sides.
pub fn shared_edge_passable(from: SideMaterial, to: SideMaterial) -> bool {
    from.is_passable() && to.is_passable()
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Bed;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Shower;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Toilet;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fixture {
    Bed(Bed),
    Shower(Shower),
    Toilet(Toilet),
}

impl Fixture {
    pub fn label(self) -> &'static str {
        match self {
            Self::Bed(_) => "Bed",
            Self::Shower(_) => "Shower",
            Self::Toilet(_) => "Toilet",
        }
    }

    pub const TAG_BED: u8 = 0;
    pub const TAG_SHOWER: u8 = 1;
    pub const TAG_TOILET: u8 = 2;

    pub fn to_tag(self) -> u8 {
        match self {
            Self::Bed(_) => Self::TAG_BED,
            Self::Shower(_) => Self::TAG_SHOWER,
            Self::Toilet(_) => Self::TAG_TOILET,
        }
    }

    pub fn from_tag(tag: u8) -> Option<Self> {
        match tag {
            Self::TAG_BED => Some(Self::Bed(Bed)),
            Self::TAG_SHOWER => Some(Self::Shower(Shower)),
            Self::TAG_TOILET => Some(Self::Toilet(Toilet)),
            _ => None,
        }
    }
}

/// CellBox grid indices `(x along ship, y across beam, deck z)`.
pub type Location = (u16, u16, u8);

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct EntityId(pub u64);

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum EntityKind {
    SimHuman,
}

impl EntityKind {
    pub const TAG_SIM_HUMAN: u8 = 0;

    pub fn to_tag(self) -> u8 {
        match self {
            Self::SimHuman => Self::TAG_SIM_HUMAN,
        }
    }

    pub fn from_tag(tag: u8) -> Option<Self> {
        match tag {
            Self::TAG_SIM_HUMAN => Some(Self::SimHuman),
            _ => None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Entity {
    pub kind: EntityKind,
    pub location: Location,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EntityError {
    DuplicateId,
    UnknownId,
    InvalidLocation,
}

/// One 1 m sea cell: four sides, floor, and built-in fixtures.
#[derive(Clone, Debug)]
pub struct Cell {
    pub side1: SideMaterial,
    pub side2: SideMaterial,
    pub side3: SideMaterial,
    pub side4: SideMaterial,
    pub floor: FloorMaterial,
    pub fixtures: Vec<Fixture>,
}

impl Cell {
    pub fn new(floor: FloorMaterial) -> Self {
        Self {
            side1: SideMaterial::Open,
            side2: SideMaterial::Open,
            side3: SideMaterial::Open,
            side4: SideMaterial::Open,
            floor,
            fixtures: Vec::new(),
        }
    }
}

/// Cardinal and diagonal step offsets for pathfinding.
pub const CELL_NEIGHBOUR_OFFSETS: [(i32, i32); 8] = [
    (1, 0),
    (-1, 0),
    (0, 1),
    (0, -1),
    (1, 1),
    (1, -1),
    (-1, 1),
    (-1, -1),
];

/// True when a cardinal step crosses only passable edges.
pub fn cardinal_step_allowed(from: &Cell, to: &Cell, dx: i32, dy: i32) -> bool {
    match (dx, dy) {
        (1, 0) => shared_edge_passable(from.side1, to.side3),
        (-1, 0) => shared_edge_passable(from.side3, to.side1),
        (0, 1) => shared_edge_passable(from.side2, to.side4),
        (0, -1) => shared_edge_passable(from.side4, to.side2),
        _ => false,
    }
}

/// True when a step to `(ix + dx, iy + dy)` exists and every crossed edge is passable.
#[cfg(test)]
pub fn step_allowed(cells: &HashMap<(i32, i32), Cell>, ix: i32, iy: i32, dx: i32, dy: i32) -> bool {
    let Some(from) = cells.get(&(ix, iy)) else {
        return false;
    };
    if dx == 0 && dy == 0 {
        return false;
    }
    if dx == 0 || dy == 0 {
        let Some(to) = cells.get(&(ix + dx, iy + dy)) else {
            return false;
        };
        return cardinal_step_allowed(from, to, dx, dy);
    }
    let c1 = (dx.signum(), 0);
    let c2 = (0, dy.signum());
    let Some(mid1) = cells.get(&(ix + c1.0, iy + c1.1)) else {
        return false;
    };
    let Some(mid2) = cells.get(&(ix + c2.0, iy + c2.1)) else {
        return false;
    };
    cardinal_step_allowed(from, mid1, c1.0, c1.1) && cardinal_step_allowed(from, mid2, c2.0, c2.1)
}

/// True when a diagonal step is allowed (both adjacent cardinals must exist).
#[cfg(test)]
pub fn diagonal_step_allowed(
    grid: &HashMap<(i32, i32), Vec2>,
    ix: i32,
    iy: i32,
    dx: i32,
    dy: i32,
) -> bool {
    if dx == 0 || dy == 0 {
        return grid.contains_key(&(ix + dx, iy + dy));
    }
    grid.contains_key(&(ix + dx, iy)) && grid.contains_key(&(ix, iy + dy))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn floor_material_has_two_variants() {
        assert_eq!(FloorMaterial::COUNT, 2);
        assert_eq!(FloorMaterial::Wood.idx(), 1);
    }

    #[test]
    fn side_material_has_four_variants() {
        assert_eq!(SideMaterial::COUNT, 4);
        assert_eq!(SideMaterial::MarinePanel.idx(), 1);
        assert_eq!(SideMaterial::Door.idx(), 2);
        assert_eq!(SideMaterial::Window.idx(), 3);
    }

    #[test]
    fn diagonal_requires_both_cardinals() {
        let mut grid = HashMap::new();
        grid.insert((0, 0), Vec2::ZERO);
        grid.insert((1, 0), Vec2::X);
        assert!(!diagonal_step_allowed(&grid, 0, 0, 1, 1));
        grid.insert((0, 1), Vec2::Y);
        assert!(diagonal_step_allowed(&grid, 0, 0, 1, 1));
    }

    fn open_cell() -> Cell {
        Cell::new(FloorMaterial::Carpet)
    }

    fn walled_east(cell: &mut Cell) {
        cell.side1 = SideMaterial::MarinePanel;
    }

    #[test]
    fn cardinal_blocked_when_side_not_open() {
        let mut cells = HashMap::new();
        let mut a = open_cell();
        walled_east(&mut a);
        cells.insert((0, 0), a);
        cells.insert((1, 0), open_cell());
        assert!(!step_allowed(&cells, 0, 0, 1, 0));
    }

    #[test]
    fn cardinal_allowed_when_both_edges_open() {
        let mut cells = HashMap::new();
        cells.insert((0, 0), open_cell());
        cells.insert((1, 0), open_cell());
        assert!(step_allowed(&cells, 0, 0, 1, 0));
    }

    #[test]
    fn diagonal_blocked_if_one_cardinal_side_closed() {
        let mut cells = HashMap::new();
        let mut a = open_cell();
        walled_east(&mut a);
        cells.insert((0, 0), a);
        cells.insert((1, 0), open_cell());
        cells.insert((0, 1), open_cell());
        cells.insert((1, 1), open_cell());
        assert!(!step_allowed(&cells, 0, 0, 1, 1));
    }

    #[test]
    fn cell_stores_fixtures() {
        let mut cell = open_cell();
        cell.fixtures.push(Fixture::Bed(Bed));
        cell.fixtures.push(Fixture::Toilet(Toilet));
        assert_eq!(cell.fixtures.len(), 2);
        assert_eq!(cell.fixtures[0].label(), "Bed");
    }

    #[test]
    fn fixture_tag_round_trip() {
        for fixture in [
            Fixture::Bed(Bed),
            Fixture::Shower(Shower),
            Fixture::Toilet(Toilet),
        ] {
            let tag = fixture.to_tag();
            assert_eq!(Fixture::from_tag(tag), Some(fixture));
        }
        assert_eq!(Fixture::from_tag(99), None);
    }
}
