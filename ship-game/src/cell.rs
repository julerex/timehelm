//! Sea-cell grid types: materials, rooms, bags, and per-cell geometry.

use bevy::prelude::*;
use std::collections::HashMap;

/// Wall and floor surface materials (exactly 18 variants).
#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum Material {
    Open = 0,
    Hull = 1,
    Window = 2,
    CabinPartition = 3,
    Corridor = 4,
    PublicShell = 5,
    DeckBase = 6,
    Theatre = 7,
    Dining = 8,
    Buffet = 9,
    Pool = 10,
    Casino = 11,
    CabinStripeA = 12,
    CabinStripeB = 13,
    CorridorWhite = 14,
    BowAccent = 15,
    MarinePanel = 16,
    Door = 17,
}

impl Material {
    pub const COUNT: usize = 18;

    pub const ALL: [Self; Self::COUNT] = [
        Self::Open,
        Self::Hull,
        Self::Window,
        Self::CabinPartition,
        Self::Corridor,
        Self::PublicShell,
        Self::DeckBase,
        Self::Theatre,
        Self::Dining,
        Self::Buffet,
        Self::Pool,
        Self::Casino,
        Self::CabinStripeA,
        Self::CabinStripeB,
        Self::CorridorWhite,
        Self::BowAccent,
        Self::MarinePanel,
        Self::Door,
    ];

    pub fn idx(self) -> usize {
        self as usize
    }

    pub fn from_idx(idx: usize) -> Self {
        Self::ALL[idx % Self::COUNT]
    }

    pub fn next(self) -> Self {
        Self::from_idx(self.idx() + 1)
    }

    /// Edges agents may cross when both sides are passable (`Open`, or `Door` via [`shared_edge_passable`]).
    pub fn is_passable(self) -> bool {
        matches!(self, Self::Open | Self::Door)
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Open => "Open",
            Self::Hull => "Hull",
            Self::Window => "Window",
            Self::CabinPartition => "Cabin partition",
            Self::Corridor => "Corridor",
            Self::PublicShell => "Public shell",
            Self::DeckBase => "Deck base",
            Self::Theatre => "Theatre",
            Self::Dining => "Dining",
            Self::Buffet => "Buffet",
            Self::Pool => "Pool",
            Self::Casino => "Casino",
            Self::CabinStripeA => "Cabin stripe A",
            Self::CabinStripeB => "Cabin stripe B",
            Self::CorridorWhite => "Corridor white",
            Self::BowAccent => "Bow accent",
            Self::MarinePanel => "Marine panel",
            Self::Door => "Door",
        }
    }

    pub fn color(self) -> Color {
        match self {
            Self::Open => Color::srgb(0.0, 0.0, 0.0),
            Self::Hull => Color::srgb(0.38, 0.3, 0.24),
            Self::Window => Color::srgb(0.42, 0.62, 0.9),
            Self::CabinPartition => Color::srgb(0.55, 0.45, 0.38),
            Self::Corridor => Color::srgb(0.65, 0.68, 0.72),
            Self::PublicShell => Color::srgb(0.78, 0.86, 0.92),
            Self::DeckBase => Color::srgb(0.42, 0.48, 0.55),
            Self::Theatre => Color::srgb(0.52, 0.36, 0.68),
            Self::Dining => Color::srgb(0.82, 0.48, 0.34),
            Self::Buffet => Color::srgb(0.42, 0.70, 0.50),
            Self::Pool => Color::srgb(0.32, 0.74, 0.88),
            Self::Casino => Color::srgb(0.72, 0.22, 0.42),
            Self::CabinStripeA => Color::srgb(0.06, 0.38, 0.96),
            Self::CabinStripeB => Color::srgb(0.55, 0.90, 0.58),
            Self::CorridorWhite => Color::srgb(0.97, 0.97, 0.995),
            Self::BowAccent => Color::srgb(0.88, 0.55, 0.72),
            Self::MarinePanel => Color::srgb(0.62, 0.52, 0.44),
            Self::Door => Color::srgb(0.48, 0.32, 0.22),
        }
    }

    /// Floor tint for 2D plan view (folds hull edge and inner cabin into outer cabin mass).
    pub fn plan_floor_color(self, deck_index: usize) -> Color {
        const OUTER_CABIN: Color = Color::srgb(0.95, 0.82, 0.35);
        match self {
            Self::Hull | Self::CabinPartition | Self::MarinePanel => OUTER_CABIN,
            Self::DeckBase => {
                let hue = 0.52 + (deck_index as f32 * 0.012);
                Color::hsla((hue * 360.0) % 360.0, 0.28, 0.42, 1.0)
            }
            _ => self.color(),
        }
    }
}

/// True when agents may cross the edge between two wall materials.
pub fn shared_edge_passable(from_wall: Material, to_wall: Material) -> bool {
    if from_wall == Material::Door || to_wall == Material::Door {
        return true;
    }
    from_wall.is_passable() && to_wall.is_passable()
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum RoomCategory {
    Exterior,
    Cabin,
    Corridor,
    Public,
    Amenity,
}

impl RoomCategory {
    pub fn label(self) -> &'static str {
        match self {
            Self::Exterior => "Exterior",
            Self::Cabin => "Cabin",
            Self::Corridor => "Corridor",
            Self::Public => "Public",
            Self::Amenity => "Amenity",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub struct RoomId(pub u32);

#[derive(Clone, Debug)]
pub struct Room {
    pub id: u32,
    pub name: &'static str,
    pub deck: u8,
    pub category: RoomCategory,
}

#[derive(Clone, Default)]
pub struct RoomCatalog {
    pub rooms: HashMap<RoomId, Room>,
    next_id: u32,
}

impl RoomCatalog {
    pub fn insert(&mut self, name: &'static str, deck: u8, category: RoomCategory) -> RoomId {
        let id = RoomId(self.next_id);
        self.next_id += 1;
        self.rooms.insert(
            id,
            Room {
                id: id.0,
                name,
                deck,
                category,
            },
        );
        id
    }

    pub fn get(&self, id: RoomId) -> Option<&Room> {
        self.rooms.get(&id)
    }

    pub fn category(&self, id: RoomId) -> Option<RoomCategory> {
        self.get(id).map(|r| r.category)
    }

    /// Rebuild a catalog from persisted room rows (ids must match saved data).
    pub fn from_persisted(
        rooms: impl IntoIterator<Item = (RoomId, &'static str, u8, RoomCategory)>,
    ) -> Self {
        let mut catalog = Self::default();
        let mut next_id = 0u32;
        for (id, name, deck, category) in rooms {
            catalog.rooms.insert(
                id,
                Room {
                    id: id.0,
                    name,
                    deck,
                    category,
                },
            );
            next_id = next_id.max(id.0 + 1);
        }
        catalog.next_id = next_id;
        catalog
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub struct AgentId(pub u64);

#[derive(Clone, Default, Debug)]
pub struct Bag {
    agents: Vec<AgentId>,
}

impl Bag {
    pub fn insert(&mut self, agent: AgentId) {
        if !self.agents.contains(&agent) {
            self.agents.push(agent);
        }
    }

    pub fn remove(&mut self, agent: AgentId) {
        self.agents.retain(|&a| a != agent);
    }

    pub fn contains(&self, agent: AgentId) -> bool {
        self.agents.contains(&agent)
    }

    pub fn agents(&self) -> &[AgentId] {
        &self.agents
    }
}

/// One 1 m sea cell: four edge walls, floor, room membership, and agents on the cell.
#[derive(Clone, Debug)]
pub struct Cell {
    pub wall1: Material,
    pub wall2: Material,
    pub wall3: Material,
    pub wall4: Material,
    pub floor: Material,
    pub room: RoomId,
    pub contents: Bag,
}

impl Cell {
    pub fn new(floor: Material, room: RoomId) -> Self {
        Self {
            wall1: Material::Open,
            wall2: Material::Open,
            wall3: Material::Open,
            wall4: Material::Open,
            floor,
            room,
            contents: Bag::default(),
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

/// True when a cardinal step crosses only passable edges (`Open`, or `Door` on either side).
pub fn cardinal_step_allowed(from: &Cell, to: &Cell, dx: i32, dy: i32) -> bool {
    match (dx, dy) {
        (1, 0) => shared_edge_passable(from.wall1, to.wall3),
        (-1, 0) => shared_edge_passable(from.wall3, to.wall1),
        (0, 1) => shared_edge_passable(from.wall2, to.wall4),
        (0, -1) => shared_edge_passable(from.wall4, to.wall2),
        _ => false,
    }
}

/// True when a step to `(ix + dx, iy + dy)` exists and every crossed edge is passable.
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

/// Edge wall material between two cells (or exterior when neighbour missing).
pub fn edge_wall_material(
    occupied: &std::collections::HashSet<(i32, i32)>,
    _from: (i32, i32),
    to: (i32, i32),
    from_room: RoomId,
    to_room: Option<RoomId>,
    exterior: Material,
) -> Material {
    if !occupied.contains(&to) {
        return exterior;
    }
    match to_room {
        Some(r) if r == from_room => Material::Open,
        Some(_) => Material::CabinPartition,
        None => Material::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn material_has_eighteen_variants() {
        assert_eq!(Material::COUNT, 18);
        assert_eq!(Material::BowAccent.idx(), 15);
        assert_eq!(Material::MarinePanel.idx(), 16);
        assert_eq!(Material::Door.idx(), 17);
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
        Cell::new(Material::DeckBase, RoomId(0))
    }

    fn walled_east(cell: &mut Cell) {
        cell.wall1 = Material::MarinePanel;
    }

    #[test]
    fn cardinal_blocked_when_wall_not_open() {
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
    fn cardinal_allowed_through_door() {
        let mut cells = HashMap::new();
        let mut a = open_cell();
        a.wall1 = Material::Door;
        cells.insert((0, 0), a);
        cells.insert((1, 0), open_cell());
        assert!(step_allowed(&cells, 0, 0, 1, 0));
    }

    #[test]
    fn cardinal_allowed_door_against_marine_panel() {
        let mut cells = HashMap::new();
        let mut a = open_cell();
        a.wall1 = Material::Door;
        let mut b = open_cell();
        b.wall3 = Material::MarinePanel;
        cells.insert((0, 0), a);
        cells.insert((1, 0), b);
        assert!(step_allowed(&cells, 0, 0, 1, 0));
    }

    #[test]
    fn diagonal_blocked_if_one_cardinal_wall_closed() {
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
    fn edge_wall_open_same_room() {
        let mut occ = std::collections::HashSet::new();
        occ.insert((0, 0));
        occ.insert((1, 0));
        let r = RoomId(1);
        assert_eq!(
            edge_wall_material(&occ, (0, 0), (1, 0), r, Some(r), Material::Hull),
            Material::Open
        );
    }

    #[test]
    fn edge_wall_hull_when_missing_neighbour() {
        let occ = std::collections::HashSet::from([(0, 0)]);
        assert_eq!(
            edge_wall_material(&occ, (0, 0), (1, 0), RoomId(0), None, Material::Hull),
            Material::Hull
        );
    }

    #[test]
    fn bag_tracks_agents() {
        let mut bag = Bag::default();
        let a = AgentId(7);
        bag.insert(a);
        assert!(bag.contains(a));
        bag.remove(a);
        assert!(!bag.contains(a));
    }
}
