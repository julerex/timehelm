//! Swinging doors, cabin fittings, corridor carpets, and the bulkheads between cabins.
//!
//! Door leaves are a unit-width mesh. The viewer scales them to the opening and
//! swings them about the hinge. Positions still come from the room rectangles.

use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;
use sim::{
    cabin_fittings, polygon_centroid, shared_edge_span, EdgeSpan, Fitting, FittingKind, RoomKind,
    RoomView,
};

pub const DOOR_COLOR: [f32; 4] = [0.86, 0.83, 0.76, 1.0];
pub const BED_COLOR: [f32; 4] = [0.76, 0.58, 0.38, 1.0];
pub const TOILET_COLOR: [f32; 4] = [0.93, 0.94, 0.92, 1.0];
pub const SHOWER_COLOR: [f32; 4] = [0.62, 0.78, 0.84, 1.0];
pub const RUNNER_COLOR: [f32; 4] = [0.48, 0.14, 0.16, 1.0];

const PANEL_COLOR: [f32; 4] = [0.75, 0.71, 0.63, 1.0];
const HANDLE_COLOR: [f32; 4] = [0.42, 0.38, 0.32, 1.0];
const FRAME_COLOR: [f32; 4] = [0.30, 0.26, 0.22, 1.0];
const SILL_COLOR: [f32; 4] = [0.22, 0.20, 0.18, 1.0];
const WOOD_COLOR: [f32; 4] = [0.36, 0.24, 0.16, 1.0];
const PILLOW_COLOR: [f32; 4] = [0.93, 0.91, 0.86, 1.0];
const SEAT_COLOR: [f32; 4] = [0.78, 0.82, 0.84, 1.0];
const TRAY_COLOR: [f32; 4] = [0.55, 0.58, 0.60, 1.0];
const METAL_COLOR: [f32; 4] = [0.72, 0.75, 0.78, 1.0];
const BULKHEAD_COLOR: [f32; 4] = [0.40, 0.42, 0.46, 1.0];

pub const DOOR_OPENING_M: f32 = 1.6;
pub const PARTITION_HEIGHT_M: f32 = 2.4;
const DOOR_LEAF_HEIGHT_M: f32 = 2.02;
const SWING_RAD: f32 = 75.0_f32.to_radians();

/// Width of the hole left in a shared wall, matching [`door_placements`].
pub fn opening_width(span_length: f32) -> f32 {
    DOOR_OPENING_M
        .min(span_length * 0.7)
        .clamp(0.9, span_length.max(0.9))
}

#[derive(Clone, Copy, Debug)]
pub struct DoorPlacement {
    pub hinge: [f32; 2],
    pub yaw: f32,
    pub swing: f32,
    pub width: f32,
    pub span: EdgeSpan,
}

pub fn door_placements(rooms: &[&RoomView]) -> Vec<DoorPlacement> {
    let mut doors = Vec::new();
    for (index, room) in rooms.iter().enumerate() {
        for other in rooms.iter().skip(index + 1) {
            let Some(shared) = shared_edge_span(&room.polygon, &other.polygon) else {
                continue;
            };
            let length = shared.length();
            if length < 1.2 {
                continue;
            }
            let width = opening_width(length);
            let dx = shared.end[0] - shared.start[0];
            let dy = shared.end[1] - shared.start[1];
            let dir = [dx / length, dy / length];
            let mid = shared.midpoint();
            let hinge = [mid[0] - dir[0] * width * 0.5, mid[1] - dir[1] * width * 0.5];
            let latch = [mid[0] + dir[0] * width * 0.5, mid[1] + dir[1] * width * 0.5];
            let normal = [-dir[1], dir[0]];
            doors.push(DoorPlacement {
                hinge,
                yaw: dir[1].atan2(dir[0]),
                swing: swing_into_room(room, other, mid, normal),
                width,
                span: EdgeSpan {
                    start: hinge,
                    end: latch,
                },
            });
        }
    }
    doors
}

/// Closed door in local space: hinge at the origin, latch toward +X, thickness on Y.
pub fn door_leaf_mesh() -> Mesh {
    let mut parts = Parts::default();
    add_aabb(
        &mut parts,
        [0.04, -0.028, 0.02],
        [0.96, 0.028, DOOR_LEAF_HEIGHT_M],
        DOOR_COLOR,
    );
    for side in [-1.0_f32, 1.0] {
        let y0 = side * 0.028;
        let y1 = side * 0.046;
        add_aabb(&mut parts, [0.12, y0, 0.22], [0.88, y1, 0.92], PANEL_COLOR);
        add_aabb(&mut parts, [0.12, y0, 1.08], [0.88, y1, 1.82], PANEL_COLOR);
        let hy0 = if side > 0.0 { 0.028 } else { -0.09 };
        let hy1 = if side > 0.0 { 0.09 } else { -0.028 };
        add_aabb(
            &mut parts,
            [0.80, hy0, 0.92],
            [0.88, hy1, 1.08],
            HANDLE_COLOR,
        );
    }
    parts.into_mesh().expect("door leaf")
}

pub fn door_frame_mesh(rooms: &[&RoomView]) -> Option<Mesh> {
    let mut parts = Parts::default();
    for door in door_placements(rooms) {
        let length = door.span.length().max(0.001);
        let dir = [
            (door.span.end[0] - door.span.start[0]) / length,
            (door.span.end[1] - door.span.start[1]) / length,
        ];
        let right = [-dir[1], dir[0]];
        let hinge_post = shift(door.hinge, dir, 0.035);
        let latch = door.span.end;
        let latch_post = shift(latch, dir, -0.035);
        add_prism_oriented(
            &mut parts,
            hinge_post,
            right,
            dir,
            0.14,
            0.07,
            0.0,
            PARTITION_HEIGHT_M,
            FRAME_COLOR,
        );
        add_prism_oriented(
            &mut parts,
            latch_post,
            right,
            dir,
            0.14,
            0.07,
            0.0,
            PARTITION_HEIGHT_M,
            FRAME_COLOR,
        );
        let header = shift(door.hinge, dir, door.width * 0.5);
        add_prism_oriented(
            &mut parts,
            header,
            right,
            dir,
            0.14,
            door.width,
            DOOR_LEAF_HEIGHT_M,
            2.16,
            FRAME_COLOR,
        );
        add_prism_oriented(
            &mut parts,
            header,
            right,
            dir,
            0.12,
            door.width,
            2.16,
            PARTITION_HEIGHT_M,
            FRAME_COLOR,
        );
        add_prism_oriented(
            &mut parts, header, right, dir, 0.16, door.width, 0.0, 0.025, SILL_COLOR,
        );
    }
    parts.into_mesh()
}

pub fn deck_fitting_mesh(rooms: &[&RoomView]) -> Option<Mesh> {
    let mut parts = Parts::default();
    for room in rooms {
        let doorway = doorway_for(room, rooms);
        for fitting in cabin_fittings(room.kind, &room.polygon, doorway) {
            add_fitting(&mut parts, &fitting);
        }
    }
    parts.into_mesh()
}

pub fn corridor_runner_mesh(rooms: &[&RoomView]) -> Option<Mesh> {
    let mut parts = Parts::default();
    for room in rooms {
        if room.kind != RoomKind::Corridor || venue_floor(room) {
            continue;
        }
        let Some(rect) = runner_rect(bounds(&room.polygon)) else {
            continue;
        };
        add_aabb(
            &mut parts,
            [rect[0], rect[1], 0.012],
            [rect[2], rect[3], 0.03],
            RUNNER_COLOR,
        );
    }
    parts.into_mesh()
}

/// Solid fill for the gap between cabin modules. Those gaps are bulkheads, not doors.
pub fn bulkhead_mesh(rooms: &[&RoomView]) -> Option<Mesh> {
    let mut parts = Parts::default();
    let cabins: Vec<_> = rooms
        .iter()
        .copied()
        .filter(|room| matches!(room.kind, RoomKind::Berth | RoomKind::Head))
        .collect();
    for (index, room) in cabins.iter().enumerate() {
        let left = bounds(&room.polygon);
        for other in cabins.iter().skip(index + 1) {
            let Some(gap) = cabin_gap(left, bounds(&other.polygon)) else {
                continue;
            };
            add_aabb(
                &mut parts,
                [gap[0], gap[1], 0.0],
                [gap[2], gap[3], PARTITION_HEIGHT_M],
                BULKHEAD_COLOR,
            );
        }
    }
    parts.into_mesh()
}

fn swing_into_room(a: &RoomView, b: &RoomView, mid: [f32; 2], normal: [f32; 2]) -> f32 {
    let a_side = room_side(a, mid, normal);
    let b_side = room_side(b, mid, normal);
    let away = if a.kind == RoomKind::Corridor && b.kind != RoomKind::Corridor {
        -a_side.signum()
    } else if b.kind == RoomKind::Corridor && a.kind != RoomKind::Corridor {
        -b_side.signum()
    } else if a_side.abs() >= b_side.abs() {
        -a_side.signum()
    } else {
        -b_side.signum()
    };
    if away == 0.0 {
        SWING_RAD
    } else {
        away * SWING_RAD
    }
}

fn room_side(room: &RoomView, mid: [f32; 2], normal: [f32; 2]) -> f32 {
    let (cx, cy) = polygon_centroid(&room.polygon);
    (cx - mid[0]) * normal[0] + (cy - mid[1]) * normal[1]
}

fn venue_floor(room: &RoomView) -> bool {
    matches!(
        room.name.as_str(),
        "The Hideaway"
            | "Water's Edge"
            | "Cloud 17"
            | "Category 6"
            | "Slide tower"
            | "The Grove"
            | "Park path"
    )
}

fn doorway_for(room: &RoomView, rooms: &[&RoomView]) -> Option<[f32; 2]> {
    let mut fallback = None;
    for other in rooms {
        if other.id == room.id {
            continue;
        }
        let Some(span) = shared_edge_span(&room.polygon, &other.polygon) else {
            continue;
        };
        let mid = span.midpoint();
        if other.kind == RoomKind::Corridor {
            return Some(mid);
        }
        fallback.get_or_insert(mid);
    }
    fallback
}

fn add_fitting(parts: &mut Parts, fitting: &Fitting) {
    match fitting.kind {
        FittingKind::Bed => add_bed(parts, fitting),
        FittingKind::Toilet => add_toilet(parts, fitting),
        FittingKind::Shower => add_shower(parts, fitting),
    }
}

fn add_bed(parts: &mut Parts, fitting: &Fitting) {
    let (right, facing) = basis(fitting.facing);
    let center = fitting.center;
    add_prism_oriented(
        parts, center, right, facing, 1.40, 2.00, 0.02, 0.20, WOOD_COLOR,
    );
    add_prism_oriented(
        parts, center, right, facing, 1.26, 1.84, 0.20, 0.46, BED_COLOR,
    );
    add_prism_oriented(
        parts,
        shift(center, facing, 0.62),
        right,
        facing,
        0.64,
        0.36,
        0.46,
        0.60,
        PILLOW_COLOR,
    );
    add_prism_oriented(
        parts,
        shift(center, facing, 0.96),
        right,
        facing,
        1.46,
        0.08,
        0.20,
        1.02,
        WOOD_COLOR,
    );
}

fn add_toilet(parts: &mut Parts, fitting: &Fitting) {
    let (right, facing) = basis(fitting.facing);
    let center = fitting.center;
    add_prism_oriented(
        parts,
        shift(center, facing, -0.04),
        right,
        facing,
        0.22,
        0.30,
        0.0,
        0.18,
        TOILET_COLOR,
    );
    add_prism_oriented(
        parts,
        shift(center, facing, -0.06),
        right,
        facing,
        0.38,
        0.46,
        0.18,
        0.40,
        TOILET_COLOR,
    );
    add_prism_oriented(
        parts,
        shift(center, facing, -0.08),
        right,
        facing,
        0.34,
        0.40,
        0.40,
        0.47,
        SEAT_COLOR,
    );
    add_prism_oriented(
        parts,
        shift(center, facing, 0.24),
        right,
        facing,
        0.34,
        0.16,
        0.18,
        0.68,
        TOILET_COLOR,
    );
}

fn add_shower(parts: &mut Parts, fitting: &Fitting) {
    let (right, facing) = basis(fitting.facing);
    let center = fitting.center;
    add_prism_oriented(
        parts, center, right, facing, 1.00, 1.00, 0.0, 0.08, TRAY_COLOR,
    );
    add_prism_oriented(
        parts,
        shift(center, facing, 0.47),
        right,
        facing,
        1.00,
        0.05,
        0.08,
        1.88,
        SHOWER_COLOR,
    );
    add_prism_oriented(
        parts,
        shift(center, right, 0.47),
        right,
        facing,
        0.05,
        1.00,
        0.08,
        1.88,
        SHOWER_COLOR,
    );
    add_prism_oriented(
        parts,
        shift(center, right, -0.47),
        right,
        facing,
        0.05,
        1.00,
        0.08,
        1.88,
        SHOWER_COLOR,
    );
    add_prism_oriented(
        parts,
        shift(shift(center, facing, 0.40), right, 0.0),
        right,
        facing,
        0.12,
        0.10,
        1.62,
        1.78,
        METAL_COLOR,
    );
}

fn basis(facing: [f32; 2]) -> ([f32; 2], [f32; 2]) {
    let len = (facing[0] * facing[0] + facing[1] * facing[1])
        .sqrt()
        .max(0.001);
    let facing = [facing[0] / len, facing[1] / len];
    ([-facing[1], facing[0]], facing)
}

fn shift(center: [f32; 2], dir: [f32; 2], amount: f32) -> [f32; 2] {
    [center[0] + dir[0] * amount, center[1] + dir[1] * amount]
}

fn bounds(poly: &[[f32; 2]]) -> [f32; 4] {
    let mut rect = [f32::MAX, f32::MAX, f32::MIN, f32::MIN];
    for point in poly {
        rect[0] = rect[0].min(point[0]);
        rect[1] = rect[1].min(point[1]);
        rect[2] = rect[2].max(point[0]);
        rect[3] = rect[3].max(point[1]);
    }
    rect
}

fn runner_rect(rect: [f32; 4]) -> Option<[f32; 4]> {
    let width = rect[2] - rect[0];
    let depth = rect[3] - rect[1];
    if width < 1.6 || depth < 1.6 {
        return None;
    }
    let carpet = if width >= depth {
        let inset = (depth * 0.28).clamp(0.35, depth * 0.42);
        let pad = 0.7_f32.min(width * 0.06);
        [
            rect[0] + pad,
            rect[1] + inset,
            rect[2] - pad,
            rect[3] - inset,
        ]
    } else {
        let inset = (width * 0.28).clamp(0.35, width * 0.42);
        let pad = 0.7_f32.min(depth * 0.06);
        [
            rect[0] + inset,
            rect[1] + pad,
            rect[2] - inset,
            rect[3] - pad,
        ]
    };
    (carpet[2] - carpet[0] > 0.4 && carpet[3] - carpet[1] > 0.4).then_some(carpet)
}

fn cabin_gap(a: [f32; 4], b: [f32; 4]) -> Option<[f32; 4]> {
    let y0 = a[1].max(b[1]);
    let y1 = a[3].min(b[3]);
    let x0 = a[0].max(b[0]);
    let x1 = a[2].min(b[2]);
    if a[2] <= b[0] {
        let gap = b[0] - a[2];
        if (0.8..=3.2).contains(&gap) && y1 - y0 > 2.5 {
            return Some([a[2], y0, b[0], y1]);
        }
    }
    if b[2] <= a[0] {
        let gap = a[0] - b[2];
        if (0.8..=3.2).contains(&gap) && y1 - y0 > 2.5 {
            return Some([b[2], y0, a[0], y1]);
        }
    }
    if a[3] <= b[1] {
        let gap = b[1] - a[3];
        if (0.8..=3.2).contains(&gap) && x1 - x0 > 2.5 {
            return Some([x0, a[3], x1, b[1]]);
        }
    }
    if b[3] <= a[1] {
        let gap = a[1] - b[3];
        if (0.8..=3.2).contains(&gap) && x1 - x0 > 2.5 {
            return Some([x0, b[3], x1, a[1]]);
        }
    }
    None
}

#[allow(clippy::too_many_arguments)]
fn add_prism_oriented(
    parts: &mut Parts,
    center: [f32; 2],
    right: [f32; 2],
    facing: [f32; 2],
    across: f32,
    along: f32,
    z0: f32,
    z1: f32,
    color: [f32; 4],
) {
    let corner = |sx: f32, sy: f32| -> [f32; 2] {
        [
            center[0] + right[0] * sx * across * 0.5 + facing[0] * sy * along * 0.5,
            center[1] + right[1] * sx * across * 0.5 + facing[1] * sy * along * 0.5,
        ]
    };
    add_prism(
        parts,
        [
            corner(-1.0, -1.0),
            corner(-1.0, 1.0),
            corner(1.0, 1.0),
            corner(1.0, -1.0),
        ],
        z0,
        z1,
        color,
    );
}

fn add_aabb(parts: &mut Parts, min: [f32; 3], max: [f32; 3], color: [f32; 4]) {
    add_prism(
        parts,
        [
            [min[0], min[1]],
            [max[0], min[1]],
            [max[0], max[1]],
            [min[0], max[1]],
        ],
        min[2],
        max[2],
        color,
    );
}

fn add_prism(parts: &mut Parts, corners: [[f32; 2]; 4], z0: f32, z1: f32, color: [f32; 4]) {
    let at = |index: usize, z: f32| [corners[index][0], corners[index][1], z];
    add_face(parts, [at(0, z1), at(1, z1), at(2, z1), at(3, z1)], color);
    add_face(parts, [at(0, z0), at(3, z0), at(2, z0), at(1, z0)], color);
    for index in 0..4 {
        let next = (index + 1) % 4;
        add_face(
            parts,
            [at(index, z0), at(next, z0), at(next, z1), at(index, z1)],
            color,
        );
    }
}

fn add_face(parts: &mut Parts, quad: [[f32; 3]; 4], color: [f32; 4]) {
    let normal = face_normal(quad[0], quad[1], quad[2]);
    let base = parts.positions.len() as u32;
    for point in quad {
        parts.positions.push(point);
        parts.normals.push(normal);
        parts.uvs.push([0.0, 0.0]);
        parts.colors.push(color);
    }
    parts
        .indices
        .extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
}

fn face_normal(a: [f32; 3], b: [f32; 3], c: [f32; 3]) -> [f32; 3] {
    let ux = b[0] - a[0];
    let uy = b[1] - a[1];
    let uz = b[2] - a[2];
    let vx = c[0] - a[0];
    let vy = c[1] - a[1];
    let vz = c[2] - a[2];
    let nx = uy * vz - uz * vy;
    let ny = uz * vx - ux * vz;
    let nz = ux * vy - uy * vx;
    let len = (nx * nx + ny * ny + nz * nz).sqrt().max(1e-8);
    [nx / len, ny / len, nz / len]
}

#[derive(Default)]
struct Parts {
    positions: Vec<[f32; 3]>,
    normals: Vec<[f32; 3]>,
    uvs: Vec<[f32; 2]>,
    colors: Vec<[f32; 4]>,
    indices: Vec<u32>,
}

impl Parts {
    fn into_mesh(self) -> Option<Mesh> {
        if self.indices.is_empty() {
            return None;
        }
        Some(
            Mesh::new(
                PrimitiveTopology::TriangleList,
                bevy::asset::RenderAssetUsages::default(),
            )
            .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, self.positions)
            .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, self.normals)
            .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, self.uvs)
            .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, self.colors)
            .with_inserted_indices(Indices::U32(self.indices)),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::mesh::VertexAttributeValues;

    fn mesh_has_color(mesh: &Mesh, color: [f32; 4]) -> bool {
        let Some(VertexAttributeValues::Float32x4(values)) = mesh.attribute(Mesh::ATTRIBUTE_COLOR)
        else {
            return false;
        };
        values.iter().any(|value| {
            value
                .iter()
                .zip(color)
                .all(|(got, want)| (got - want).abs() < 1e-5)
        })
    }

    fn mesh_max_z(mesh: &Mesh) -> f32 {
        let Some(VertexAttributeValues::Float32x3(values)) =
            mesh.attribute(Mesh::ATTRIBUTE_POSITION)
        else {
            return 0.0;
        };
        values.iter().map(|value| value[2]).fold(0.0, f32::max)
    }

    fn room(id: u32, kind: RoomKind, polygon: Vec<[f32; 2]>) -> RoomView {
        RoomView {
            id,
            deck: 9,
            kind,
            name: kind.label().to_string(),
            polygon,
        }
    }

    #[test]
    fn plan_marks_draw_a_door_leaf_bed_toilet_and_shower() {
        let corridor = room(
            0,
            RoomKind::Corridor,
            vec![[0.0, 8.0], [16.0, 8.0], [16.0, 14.0], [0.0, 14.0]],
        );
        let berth = room(
            1,
            RoomKind::Berth,
            vec![[0.0, 0.0], [8.0, 0.0], [8.0, 8.0], [0.0, 8.0]],
        );
        let head = room(
            2,
            RoomKind::Head,
            vec![[8.0, 0.0], [16.0, 0.0], [16.0, 8.0], [8.0, 8.0]],
        );
        let rooms = [&corridor, &berth, &head];
        let doors = door_placements(&rooms);
        assert!(doors.len() >= 2);
        assert!(doors.iter().all(|door| (0.8..1.7).contains(&door.width)));
        assert!(doors.iter().all(|door| door.swing.abs() > 0.5));
        let leaf = door_leaf_mesh();
        assert!(mesh_has_color(&leaf, DOOR_COLOR));
        assert!(mesh_max_z(&leaf) > 1.8);
        let frame = door_frame_mesh(&rooms).expect("frame");
        assert!(mesh_max_z(&frame) > 2.2);
        let fittings = deck_fitting_mesh(&rooms).expect("fittings");
        assert!(mesh_has_color(&fittings, BED_COLOR));
        assert!(mesh_has_color(&fittings, TOILET_COLOR));
        assert!(mesh_has_color(&fittings, SHOWER_COLOR));
        assert!(mesh_max_z(&fittings) > 1.5);
        let runner = corridor_runner_mesh(&rooms).expect("runner");
        assert!(mesh_has_color(&runner, RUNNER_COLOR));
    }

    #[test]
    fn cabin_gaps_become_bulkheads_instead_of_doors() {
        let left = room(
            0,
            RoomKind::Berth,
            vec![[0.0, 0.0], [4.4, 0.0], [4.4, 7.2], [0.0, 7.2]],
        );
        let right = room(
            1,
            RoomKind::Berth,
            vec![[5.6, 0.0], [10.0, 0.0], [10.0, 7.2], [5.6, 7.2]],
        );
        let rooms = [&left, &right];
        assert!(door_placements(&rooms).is_empty());
        let bulk = bulkhead_mesh(&rooms).expect("bulkhead");
        assert!(mesh_max_z(&bulk) > 2.0);
    }
}
