//! Door thresholds and berth/head fittings drawn on the deck plan.
//!
//! Positions come from the room rectangles. Nothing here is a cell lattice.

use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;
use sim::{cabin_fittings, shared_edge_span, FittingKind, RoomKind, RoomView};

pub const DOOR_COLOR: [f32; 4] = [0.55, 0.95, 0.55, 1.0];
pub const BED_COLOR: [f32; 4] = [0.76, 0.58, 0.38, 1.0];
pub const TOILET_COLOR: [f32; 4] = [0.93, 0.93, 0.88, 1.0];
pub const SHOWER_COLOR: [f32; 4] = [0.45, 0.78, 0.95, 1.0];

const DOOR_GAP_M: f32 = 1.6;
const DOOR_THICK_M: f32 = 0.22;
const MARK_Z0: f32 = 0.03;
const MARK_Z1: f32 = 0.08;

pub fn deck_door_mesh(rooms: &[&RoomView]) -> Option<Mesh> {
    let mut parts = Parts::default();
    for (index, room) in rooms.iter().enumerate() {
        for other in rooms.iter().skip(index + 1) {
            let Some(span) = shared_edge_span(&room.polygon, &other.polygon) else {
                continue;
            };
            let opening = door_opening(span.start, span.end);
            push_threshold(&mut parts, opening.0, opening.1, DOOR_COLOR);
        }
    }
    parts.into_mesh()
}

pub fn deck_fitting_mesh(rooms: &[&RoomView]) -> Option<Mesh> {
    let mut parts = Parts::default();
    for room in rooms {
        let doorway = doorway_for(room, rooms);
        for fitting in cabin_fittings(room.kind, &room.polygon, doorway) {
            push_box(
                &mut parts,
                fitting.center,
                fitting.half,
                fitting_color(fitting.kind),
            );
        }
    }
    parts.into_mesh()
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

fn door_opening(start: [f32; 2], end: [f32; 2]) -> ([f32; 2], [f32; 2]) {
    let dx = end[0] - start[0];
    let dy = end[1] - start[1];
    let length = (dx * dx + dy * dy).sqrt().max(0.001);
    let gap = DOOR_GAP_M.min(length * 0.7).max(0.9);
    let half = (gap * 0.5).min(length * 0.5);
    let ux = dx / length;
    let uy = dy / length;
    let mid = [(start[0] + end[0]) * 0.5, (start[1] + end[1]) * 0.5];
    (
        [mid[0] - ux * half, mid[1] - uy * half],
        [mid[0] + ux * half, mid[1] + uy * half],
    )
}

fn push_threshold(parts: &mut Parts, start: [f32; 2], end: [f32; 2], color: [f32; 4]) {
    let dx = end[0] - start[0];
    let dy = end[1] - start[1];
    let length = (dx * dx + dy * dy).sqrt().max(0.001);
    let px = -dy / length * DOOR_THICK_M * 0.5;
    let py = dx / length * DOOR_THICK_M * 0.5;
    let corners = [
        [start[0] + px, start[1] + py],
        [end[0] + px, end[1] + py],
        [end[0] - px, end[1] - py],
        [start[0] - px, start[1] - py],
    ];
    push_prism(parts, &corners, color);
}

fn push_box(parts: &mut Parts, center: [f32; 2], half: [f32; 2], color: [f32; 4]) {
    let corners = [
        [center[0] - half[0], center[1] - half[1]],
        [center[0] + half[0], center[1] - half[1]],
        [center[0] + half[0], center[1] + half[1]],
        [center[0] - half[0], center[1] + half[1]],
    ];
    push_prism(parts, &corners, color);
}

fn push_prism(parts: &mut Parts, corners: &[[f32; 2]; 4], color: [f32; 4]) {
    let base = parts.positions.len() as u32;
    for point in corners {
        parts.positions.push([point[0], point[1], MARK_Z1]);
        parts.normals.push([0.0, 0.0, 1.0]);
        parts.uvs.push([0.0, 0.0]);
        parts.colors.push(color);
    }
    for point in corners {
        parts.positions.push([point[0], point[1], MARK_Z0]);
        parts.normals.push([0.0, 0.0, -1.0]);
        parts.uvs.push([0.0, 0.0]);
        parts.colors.push(color);
    }
    parts.indices.extend_from_slice(&[
        base,
        base + 1,
        base + 2,
        base,
        base + 2,
        base + 3,
        base + 4,
        base + 6,
        base + 5,
        base + 4,
        base + 7,
        base + 6,
    ]);
}

fn fitting_color(kind: FittingKind) -> [f32; 4] {
    match kind {
        FittingKind::Bed => BED_COLOR,
        FittingKind::Toilet => TOILET_COLOR,
        FittingKind::Shower => SHOWER_COLOR,
    }
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

    fn room(id: u32, kind: RoomKind, polygon: Vec<[f32; 2]>) -> RoomView {
        RoomView {
            id,
            deck: 13,
            kind,
            name: kind.label().to_string(),
            polygon,
        }
    }

    #[test]
    fn plan_marks_draw_the_door_bed_toilet_and_shower() {
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
        let doors = deck_door_mesh(&rooms).expect("door marks");
        let fittings = deck_fitting_mesh(&rooms).expect("fittings");
        assert!(mesh_has_color(&doors, DOOR_COLOR));
        assert!(mesh_has_color(&fittings, BED_COLOR));
        assert!(mesh_has_color(&fittings, TOILET_COLOR));
        assert!(mesh_has_color(&fittings, SHOWER_COLOR));
    }
}
