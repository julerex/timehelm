//! Beds, toilets, and showers derived from a room's rectangle.
//!
//! Nothing is stored. A berth is the bed; a head is the toilet and the shower.
//! The doorway hint pushes them off the door so the opening stays clear.

use crate::geom::polygon_centroid;
use crate::hull::point_in_polygon;
use crate::kinds::RoomKind;
use glam::Vec2;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FittingKind {
    Bed,
    Toilet,
    Shower,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Fitting {
    pub kind: FittingKind,
    pub center: [f32; 2],
    pub half: [f32; 2],
    /// Unit vector toward the head of the bed, the toilet tank, or the shower's back wall.
    pub facing: [f32; 2],
}

/// What a room shows on the plan. Empty for every kind except berth and head.
pub fn cabin_fittings(
    kind: RoomKind,
    polygon: &[[f32; 2]],
    doorway: Option<[f32; 2]>,
) -> Vec<Fitting> {
    match kind {
        RoomKind::Berth => vec![bed(polygon, doorway)],
        RoomKind::Head => head(polygon, doorway),
        RoomKind::Corridor
        | RoomKind::Stair
        | RoomKind::Hydroponics
        | RoomKind::Fishing
        | RoomKind::ColdStore
        | RoomKind::DryStore
        | RoomKind::Galley
        | RoomKind::Bakery
        | RoomKind::Restaurant
        | RoomKind::Mess
        | RoomKind::Desalination
        | RoomKind::WaterStore
        | RoomKind::WasteTreatment
        | RoomKind::Recycling
        | RoomKind::Carpentry
        | RoomKind::Sewing
        | RoomKind::Metalwork
        | RoomKind::Laundry
        | RoomKind::TimberStore
        | RoomKind::MetalStore
        | RoomKind::Clinic
        | RoomKind::Infirmary
        | RoomKind::Engineering
        | RoomKind::Electrical
        | RoomKind::FuelStore
        | RoomKind::Bridge => Vec::new(),
    }
}

/// Short plan label for the fittings in a room.
pub fn contents_label(kind: RoomKind) -> &'static str {
    match kind {
        RoomKind::Berth => "bed",
        RoomKind::Head => "toilet and shower",
        RoomKind::Corridor
        | RoomKind::Stair
        | RoomKind::Hydroponics
        | RoomKind::Fishing
        | RoomKind::ColdStore
        | RoomKind::DryStore
        | RoomKind::Galley
        | RoomKind::Bakery
        | RoomKind::Restaurant
        | RoomKind::Mess
        | RoomKind::Desalination
        | RoomKind::WaterStore
        | RoomKind::WasteTreatment
        | RoomKind::Recycling
        | RoomKind::Carpentry
        | RoomKind::Sewing
        | RoomKind::Metalwork
        | RoomKind::Laundry
        | RoomKind::TimberStore
        | RoomKind::MetalStore
        | RoomKind::Clinic
        | RoomKind::Infirmary
        | RoomKind::Engineering
        | RoomKind::Electrical
        | RoomKind::FuelStore
        | RoomKind::Bridge => "",
    }
}

pub fn fitting_inside_room(polygon: &[[f32; 2]], fitting: Fitting) -> bool {
    let hull: Vec<Vec2> = polygon
        .iter()
        .map(|point| Vec2::new(point[0], point[1]))
        .collect();
    let [cx, cy] = fitting.center;
    let [hx, hy] = fitting.half;
    [
        (cx, cy),
        (cx - hx, cy - hy),
        (cx + hx, cy - hy),
        (cx + hx, cy + hy),
        (cx - hx, cy + hy),
    ]
    .into_iter()
    .all(|(x, y)| point_in_polygon(Vec2::new(x, y), &hull))
}

fn bed(polygon: &[[f32; 2]], doorway: Option<[f32; 2]>) -> Fitting {
    let center = deep_center(polygon, doorway, 0.18);
    let facing = into_room(polygon, doorway);
    Fitting {
        kind: FittingKind::Bed,
        center,
        half: half_for(facing, 1.08, 0.76),
        facing: [facing.0, facing.1],
    }
}

fn head(polygon: &[[f32; 2]], doorway: Option<[f32; 2]>) -> Vec<Fitting> {
    let center = deep_center(polygon, doorway, 0.12);
    let (fx, fy) = into_room(polygon, doorway);
    let (px, py) = perpendicular(polygon, doorway);
    let gap = 1.15;
    vec![
        Fitting {
            kind: FittingKind::Toilet,
            center: [center[0] + px * gap, center[1] + py * gap],
            half: [0.46, 0.46],
            facing: [fx, fy],
        },
        Fitting {
            kind: FittingKind::Shower,
            center: [center[0] - px * gap, center[1] - py * gap],
            half: [0.58, 0.58],
            facing: [fx, fy],
        },
    ]
}

fn half_for(facing: (f32, f32), along: f32, across: f32) -> [f32; 2] {
    if facing.0.abs() >= facing.1.abs() {
        [along, across]
    } else {
        [across, along]
    }
}

fn deep_center(polygon: &[[f32; 2]], doorway: Option<[f32; 2]>, reach_fraction: f32) -> [f32; 2] {
    let (cx, cy) = polygon_centroid(polygon);
    let (vx, vy) = into_room(polygon, doorway);
    let (x0, y0, x1, y1) = bounds(polygon);
    let reach = ((x1 - x0).min(y1 - y0) * reach_fraction).clamp(0.6, 2.2);
    [cx + vx * reach, cy + vy * reach]
}

fn into_room(polygon: &[[f32; 2]], doorway: Option<[f32; 2]>) -> (f32, f32) {
    let (cx, cy) = polygon_centroid(polygon);
    let door = doorway.unwrap_or([cx, cy]);
    let mut vx = cx - door[0];
    let mut vy = cy - door[1];
    let len = (vx * vx + vy * vy).sqrt();
    if len < 0.2 {
        (0.0, 1.0)
    } else {
        vx /= len;
        vy /= len;
        (vx, vy)
    }
}

fn perpendicular(polygon: &[[f32; 2]], doorway: Option<[f32; 2]>) -> (f32, f32) {
    let (vx, vy) = into_room(polygon, doorway);
    (-vy, vx)
}

fn bounds(polygon: &[[f32; 2]]) -> (f32, f32, f32, f32) {
    let mut x0 = f32::MAX;
    let mut y0 = f32::MAX;
    let mut x1 = f32::MIN;
    let mut y1 = f32::MIN;
    for point in polygon {
        x0 = x0.min(point[0]);
        y0 = y0.min(point[1]);
        x1 = x1.max(point[0]);
        y1 = y1.max(point[1]);
    }
    (x0, y0, x1, y1)
}
