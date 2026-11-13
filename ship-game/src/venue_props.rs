//! Furniture for the working rooms and the public venues.
//!
//! These meshes follow the room rectangles. They do not move crew, stocks, or recipes.
//! A pool, a slide, and a tree are scenery in the room that already has that name.
//! Lifeboats are a separate mesh on [`LIFEBOAT_DECK`].

use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;
use sim::{RoomKind, RoomView};

const WATER: [f32; 4] = [0.12, 0.48, 0.78, 1.0];
const COPING: [f32; 4] = [0.82, 0.84, 0.86, 1.0];
const CANOPY: [f32; 4] = [0.16, 0.48, 0.22, 1.0];
const TRUNK: [f32; 4] = [0.38, 0.26, 0.14, 1.0];
const BOAT: [f32; 4] = [0.95, 0.42, 0.08, 1.0];
const BOAT_CABIN: [f32; 4] = [0.93, 0.94, 0.95, 1.0];
const DAVIT: [f32; 4] = [0.72, 0.74, 0.76, 1.0];
const STEEL: [f32; 4] = [0.62, 0.66, 0.70, 1.0];
const ENGINE: [f32; 4] = [0.22, 0.25, 0.28, 1.0];
const CAUTION: [f32; 4] = [0.86, 0.72, 0.18, 1.0];
const TABLE: [f32; 4] = [0.55, 0.34, 0.22, 1.0];
const CLOTH: [f32; 4] = [0.78, 0.82, 0.86, 1.0];
const RANGE: [f32; 4] = [0.18, 0.18, 0.20, 1.0];
const OVEN: [f32; 4] = [0.45, 0.28, 0.22, 1.0];
const WASHER: [f32; 4] = [0.86, 0.88, 0.92, 1.0];
const DRUM: [f32; 4] = [0.15, 0.28, 0.42, 1.0];
const WOOD: [f32; 4] = [0.58, 0.40, 0.22, 1.0];
const SAW: [f32; 4] = [0.75, 0.76, 0.78, 1.0];
const LATHE: [f32; 4] = [0.35, 0.48, 0.42, 1.0];
const INCINERATOR: [f32; 4] = [0.32, 0.22, 0.20, 1.0];
const TANK: [f32; 4] = [0.28, 0.40, 0.36, 1.0];
const SLIDE_A: [f32; 4] = [0.15, 0.72, 0.82, 1.0];
const SLIDE_B: [f32; 4] = [0.90, 0.78, 0.12, 1.0];
const SLIDE_C: [f32; 4] = [0.82, 0.22, 0.28, 1.0];

/// Icon deck 7 (sim index 6). The boats hang outboard of the Surfside topsides.
pub const LIFEBOAT_DECK: usize = 6;

pub fn venue_mesh(rooms: &[&RoomView]) -> Option<Mesh> {
    let mut parts = Parts::default();
    for room in rooms {
        furnish(&mut parts, room);
    }
    parts.into_mesh()
}

/// Boats and davits outboard of [`LIFEBOAT_DECK`]. Kept off the room mesh so a
/// closed lower deck can still show them.
pub fn lifeboat_mesh() -> Option<Mesh> {
    let mut parts = Parts::default();
    lifeboats(&mut parts);
    parts.into_mesh()
}

fn furnish(parts: &mut Parts, room: &RoomView) {
    let rect = bounds(&room.polygon);
    match room.kind {
        RoomKind::Carpentry => carpentry(parts, rect),
        RoomKind::Metalwork => metalshop(parts, rect),
        RoomKind::Sewing => sewing(parts, rect),
        RoomKind::Laundry => laundry(parts, rect),
        RoomKind::Engineering => generators(parts, rect),
        RoomKind::Electrical => switchgear(parts, rect),
        RoomKind::FuelStore => gas_space(parts, rect),
        RoomKind::WasteTreatment => waste_plant(parts, rect),
        RoomKind::Recycling => recycling(parts, rect),
        RoomKind::Galley | RoomKind::Bakery => kitchen(parts, rect, room.kind == RoomKind::Bakery),
        RoomKind::Restaurant | RoomKind::Mess => dining(parts, room, rect),
        RoomKind::Hydroponics => trees(parts, rect),
        RoomKind::Corridor => corridor_venue(parts, room, rect),
        RoomKind::Stair
        | RoomKind::Berth
        | RoomKind::Head
        | RoomKind::Fishing
        | RoomKind::ColdStore
        | RoomKind::DryStore
        | RoomKind::Desalination
        | RoomKind::WaterStore
        | RoomKind::TimberStore
        | RoomKind::MetalStore
        | RoomKind::Clinic
        | RoomKind::Infirmary
        | RoomKind::Bridge => {}
    }
}

fn corridor_venue(parts: &mut Parts, room: &RoomView, rect: [f32; 4]) {
    match room.name.as_str() {
        "The Hideaway" | "Water's Edge" | "Cloud 17" => pool(parts, inset(rect, 4.0, 3.5)),
        "Category 6" => {
            pool(
                parts,
                [rect[0] + 3.0, rect[1] + 2.0, rect[2] - 3.0, rect[1] + 8.0],
            );
            slides(parts, rect);
        }
        "Slide tower" => slides(parts, rect),
        "The Grove" => {
            let mid_y = (rect[1] + rect[3]) * 0.5;
            trees(parts, [rect[0], rect[1], rect[2], mid_y]);
        }
        _ => {}
    }
}

fn dining(parts: &mut Parts, room: &RoomView, rect: [f32; 4]) {
    if room.name == "Windjammer Marketplace" {
        // Royal Bay sits on the starboard side of this Chill Island footprint.
        pool(
            parts,
            [rect[0] + 8.0, rect[1] + 2.5, rect[2] - 8.0, rect[1] + 14.0],
        );
        tables(parts, [rect[0], rect[1] + 15.5, rect[2], rect[3]]);
        return;
    }
    tables(parts, rect);
}

fn carpentry(parts: &mut Parts, rect: [f32; 4]) {
    bench_row(parts, rect, WOOD, 1.4, 0.75);
    let saw_at = [rect[0] + 3.2, (rect[1] + rect[3]) * 0.5];
    block(parts, saw_at, [1.6, 0.9], 0.0, 0.82, SAW);
    block(
        parts,
        [saw_at[0], saw_at[1]],
        [0.08, 0.7],
        0.82,
        1.15,
        STEEL,
    );
    rack(parts, [rect[2] - 1.6, rect[3] - 1.4], WOOD);
}

fn metalshop(parts: &mut Parts, rect: [f32; 4]) {
    let y = (rect[1] + rect[3]) * 0.5;
    block(parts, [rect[0] + 4.0, y], [3.6, 0.8], 0.72, 1.05, LATHE);
    block(parts, [rect[0] + 4.0, y], [3.8, 0.95], 0.0, 0.72, STEEL);
    block(parts, [rect[2] - 3.5, y], [1.1, 1.1], 0.0, 1.9, STEEL);
    block(
        parts,
        [rect[2] - 3.5, y + 0.7],
        [0.9, 0.35],
        1.15,
        1.45,
        LATHE,
    );
    bench_row(
        parts,
        [rect[0], rect[1], rect[2], rect[1] + 2.2],
        WOOD,
        1.6,
        0.7,
    );
}

fn sewing(parts: &mut Parts, rect: [f32; 4]) {
    bench_row(parts, rect, TABLE, 1.5, 0.7);
    let mut x = rect[0] + 2.4;
    let y = (rect[1] + rect[3]) * 0.5;
    while x < rect[2] - 1.5 {
        block(parts, [x, y - 0.15], [0.45, 0.32], 0.74, 0.92, STEEL);
        x += 2.2;
    }
}

fn laundry(parts: &mut Parts, rect: [f32; 4]) {
    let mut x = rect[0] + 1.6;
    let y = rect[1] + 1.5;
    while x < rect[2] - 1.2 {
        block(parts, [x, y], [0.85, 0.8], 0.0, 1.15, WASHER);
        block(parts, [x, y], [0.42, 0.08], 0.55, 0.95, DRUM);
        x += 1.15;
    }
    bench_row(
        parts,
        [rect[0], rect[3] - 2.4, rect[2], rect[3]],
        CLOTH,
        1.8,
        0.8,
    );
}

fn generators(parts: &mut Parts, rect: [f32; 4]) {
    let mut x = rect[0] + 8.0;
    let y = (rect[1] + rect[3]) * 0.5;
    while x < rect[2] - 6.0 {
        block(parts, [x, y], [7.2, 2.5], 0.0, 0.28, STEEL);
        block(parts, [x - 0.6, y], [4.2, 1.7], 0.28, 1.7, ENGINE);
        block(parts, [x + 2.4, y], [1.5, 1.5], 0.28, 1.45, STEEL);
        block(parts, [x + 2.4, y], [0.35, 0.35], 1.45, 2.55, CAUTION);
        x += 14.0;
    }
}

fn switchgear(parts: &mut Parts, rect: [f32; 4]) {
    let mut x = rect[0] + 1.8;
    let y0 = rect[1] + 1.3;
    let y1 = rect[3] - 1.3;
    while x < rect[2] - 1.2 {
        for y in [y0, y1] {
            block(parts, [x, y], [0.7, 0.85], 0.0, 2.05, STEEL);
            block(parts, [x, y], [0.72, 0.08], 1.35, 1.5, CAUTION);
        }
        x += 1.3;
    }
}

fn gas_space(parts: &mut Parts, rect: [f32; 4]) {
    let y = (rect[1] + rect[3]) * 0.5;
    block(
        parts,
        [(rect[0] + rect[2]) * 0.5, y - 2.2],
        [18.0, 2.4],
        0.35,
        2.15,
        TANK,
    );
    block(
        parts,
        [(rect[0] + rect[2]) * 0.5, y + 2.2],
        [18.0, 2.4],
        0.35,
        2.15,
        TANK,
    );
    block(
        parts,
        [(rect[0] + rect[2]) * 0.5, y - 2.2],
        [18.2, 2.6],
        0.0,
        0.35,
        STEEL,
    );
    block(
        parts,
        [(rect[0] + rect[2]) * 0.5, y + 2.2],
        [18.2, 2.6],
        0.0,
        0.35,
        STEEL,
    );
}

fn waste_plant(parts: &mut Parts, rect: [f32; 4]) {
    let y = (rect[1] + rect[3]) * 0.5;
    block(parts, [rect[0] + 6.0, y], [4.2, 3.2], 0.0, 2.2, INCINERATOR);
    block(parts, [rect[0] + 4.4, y], [0.9, 1.6], 0.7, 1.7, CAUTION);
    block(parts, [rect[0] + 6.0, y], [0.7, 0.7], 2.2, 2.65, STEEL);
    let mut x = rect[0] + 16.0;
    while x < rect[2] - 4.0 {
        block(parts, [x, y], [5.5, 5.0], 0.0, 1.35, TANK);
        block(parts, [x, y], [4.2, 3.6], 1.35, 1.55, WATER);
        x += 8.0;
    }
}

fn recycling(parts: &mut Parts, rect: [f32; 4]) {
    let colors = [CAUTION, SLIDE_C, CANOPY, STEEL];
    let mut x = rect[0] + 2.5;
    let y = (rect[1] + rect[3]) * 0.5;
    let mut index = 0;
    while x < rect[2] - 1.5 {
        block(
            parts,
            [x, y],
            [1.4, 1.3],
            0.0,
            1.25,
            colors[index % colors.len()],
        );
        x += 2.2;
        index += 1;
    }
}

fn kitchen(parts: &mut Parts, rect: [f32; 4], bakery: bool) {
    let counter_y = rect[1] + 1.6;
    block(
        parts,
        [(rect[0] + rect[2]) * 0.5, counter_y],
        [rect[2] - rect[0] - 2.0, 0.9],
        0.0,
        0.92,
        STEEL,
    );
    let mut x = rect[0] + 2.5;
    let appliance = if bakery { OVEN } else { RANGE };
    while x < rect[2] - 2.0 {
        block(parts, [x, counter_y], [0.85, 0.7], 0.92, 1.15, appliance);
        x += 1.7;
    }
    tables(parts, [rect[0], rect[1] + 3.5, rect[2], rect[3]]);
}

fn tables(parts: &mut Parts, rect: [f32; 4]) {
    let width = rect[2] - rect[0];
    let depth = rect[3] - rect[1];
    if width < 4.0 || depth < 3.0 {
        return;
    }
    let mut y = rect[1] + 2.2;
    let mut placed = 0;
    while y < rect[3] - 1.4 && placed < 36 {
        let mut x = rect[0] + 2.2;
        while x < rect[2] - 1.4 && placed < 36 {
            block(parts, [x, y], [1.15, 0.75], 0.0, 0.76, TABLE);
            block(parts, [x, y], [1.05, 0.65], 0.76, 0.8, CLOTH);
            x += 3.4;
            placed += 1;
        }
        y += 3.2;
    }
}

fn bench_row(parts: &mut Parts, rect: [f32; 4], color: [f32; 4], along: f32, across: f32) {
    let y = (rect[1] + rect[3]) * 0.5;
    let mut x = rect[0] + along;
    while x < rect[2] - along * 0.5 {
        block(parts, [x, y], [along, across], 0.0, 0.78, color);
        x += along + 0.7;
    }
}

fn rack(parts: &mut Parts, at: [f32; 2], color: [f32; 4]) {
    for step in 0..4 {
        let z = 0.25 + step as f32 * 0.32;
        block(parts, at, [2.4, 0.35], z, z + 0.12, color);
    }
    block(parts, [at[0] - 1.05, at[1]], [0.12, 0.4], 0.0, 1.6, WOOD);
    block(parts, [at[0] + 1.05, at[1]], [0.12, 0.4], 0.0, 1.6, WOOD);
}

fn trees(parts: &mut Parts, rect: [f32; 4]) {
    if rect[2] - rect[0] < 4.0 || rect[3] - rect[1] < 3.0 {
        return;
    }
    let mut y = rect[1] + 2.4;
    let mut planted = 0;
    while y < rect[3] - 1.6 && planted < 28 {
        let mut x = rect[0] + 2.4;
        while x < rect[2] - 1.6 && planted < 28 {
            block(parts, [x, y], [0.38, 0.38], 0.0, 1.55, TRUNK);
            block(parts, [x, y], [1.9, 1.9], 1.35, 2.35, CANOPY);
            x += 5.6;
            planted += 1;
        }
        y += 5.2;
    }
}

fn pool(parts: &mut Parts, rect: [f32; 4]) {
    if rect[2] - rect[0] < 3.0 || rect[3] - rect[1] < 3.0 {
        return;
    }
    block(
        parts,
        center(rect),
        [rect[2] - rect[0], rect[3] - rect[1]],
        0.02,
        0.16,
        COPING,
    );
    let water = inset(rect, 0.45, 0.45);
    block(
        parts,
        center(water),
        [water[2] - water[0], water[3] - water[1]],
        0.16,
        0.28,
        WATER,
    );
}

fn slides(parts: &mut Parts, rect: [f32; 4]) {
    let length = rect[2] - rect[0];
    let depth = rect[3] - rect[1];
    if length < 8.0 || depth < 6.0 {
        return;
    }
    let colors = [SLIDE_A, SLIDE_B, SLIDE_C];
    for (index, color) in colors.iter().enumerate() {
        let x = rect[0] + 2.5 + index as f32 * (length - 5.0) / 3.0;
        for step in 0..5 {
            let t = step as f32 / 4.0;
            let y = rect[1] + 1.5 + t * (depth - 3.0);
            let top = 2.15 - t * 1.45;
            block(parts, [x, y], [1.15, 1.35], top - 0.28, top, *color);
        }
    }
}

fn lifeboats(parts: &mut Parts) {
    for side in [0.0_f32, 1.0] {
        let y = if side == 0.0 { -2.6 } else { 67.6 };
        let davit_y = if side == 0.0 { -0.7 } else { 65.7 };
        for index in 0..8 {
            let x = 100.0 + index as f32 * 20.0;
            block(parts, [x, y], [9.2, 3.1], 0.35, 1.45, BOAT);
            block(parts, [x - 0.4, y], [3.4, 2.2], 1.45, 2.15, BOAT_CABIN);
            block(parts, [x - 3.6, davit_y], [0.28, 0.28], 0.0, 2.3, DAVIT);
            block(parts, [x + 3.6, davit_y], [0.28, 0.28], 0.0, 2.3, DAVIT);
            block(
                parts,
                [x, (y + davit_y) * 0.5],
                [0.2, (y - davit_y).abs()],
                2.05,
                2.25,
                DAVIT,
            );
        }
    }
}

fn inset(rect: [f32; 4], pad_x: f32, pad_y: f32) -> [f32; 4] {
    let pad_x = pad_x.min((rect[2] - rect[0]) * 0.3);
    let pad_y = pad_y.min((rect[3] - rect[1]) * 0.3);
    [
        rect[0] + pad_x,
        rect[1] + pad_y,
        rect[2] - pad_x,
        rect[3] - pad_y,
    ]
}

fn center(rect: [f32; 4]) -> [f32; 2] {
    [(rect[0] + rect[2]) * 0.5, (rect[1] + rect[3]) * 0.5]
}

fn block(parts: &mut Parts, center: [f32; 2], size: [f32; 2], z0: f32, z1: f32, color: [f32; 4]) {
    if size[0] < 0.05 || size[1] < 0.05 || z1 <= z0 {
        return;
    }
    add_prism(
        parts,
        [
            [center[0] - size[0] * 0.5, center[1] - size[1] * 0.5],
            [center[0] - size[0] * 0.5, center[1] + size[1] * 0.5],
            [center[0] + size[0] * 0.5, center[1] + size[1] * 0.5],
            [center[0] + size[0] * 0.5, center[1] - size[1] * 0.5],
        ],
        z0,
        z1,
        color,
    );
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

fn add_prism(parts: &mut Parts, corners: [[f32; 2]; 4], z0: f32, z1: f32, color: [f32; 4]) {
    let at = |index: usize, z: f32| [corners[index][0], corners[index][1], z];
    add_face(parts, [at(0, z1), at(3, z1), at(2, z1), at(1, z1)], color);
    add_face(parts, [at(0, z0), at(1, z0), at(2, z0), at(3, z0)], color);
    for index in 0..4 {
        let next = (index + 1) % 4;
        add_face(
            parts,
            [at(index, z0), at(index, z1), at(next, z1), at(next, z0)],
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

    fn room(kind: RoomKind, name: &str, polygon: Vec<[f32; 2]>) -> RoomView {
        RoomView {
            id: 1,
            deck: 1,
            kind,
            name: name.to_string(),
            polygon,
        }
    }

    fn rect(x0: f32, y0: f32, x1: f32, y1: f32) -> Vec<[f32; 2]> {
        vec![[x0, y0], [x1, y0], [x1, y1], [x0, y1]]
    }

    fn has_color(mesh: &Mesh, color: [f32; 4]) -> bool {
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

    fn positions(mesh: &Mesh) -> Vec<[f32; 3]> {
        match mesh.attribute(Mesh::ATTRIBUTE_POSITION) {
            Some(VertexAttributeValues::Float32x3(values)) => values.clone(),
            _ => Vec::new(),
        }
    }

    #[test]
    fn workshops_plant_kitchens_and_park_leave_a_readable_mesh() {
        let shop = room(RoomKind::Carpentry, "Carpentry", rect(0.0, 0.0, 20.0, 12.0));
        let galley = room(RoomKind::Galley, "Galley", rect(0.0, 0.0, 24.0, 14.0));
        let park = room(
            RoomKind::Hydroponics,
            "Central Park",
            rect(120.0, 21.0, 150.0, 31.0),
        );
        let grove = room(
            RoomKind::Corridor,
            "The Grove",
            rect(220.0, 20.0, 250.0, 46.0),
        );
        let hideaway = room(
            RoomKind::Corridor,
            "The Hideaway",
            rect(36.0, 14.0, 70.0, 50.0),
        );
        let waste = room(
            RoomKind::WasteTreatment,
            "Wastewater and incinerator",
            rect(150.0, 12.0, 200.0, 26.0),
        );
        let rooms = [&shop, &galley, &park, &grove, &hideaway, &waste];
        let mesh = venue_mesh(&rooms).expect("venues");
        assert!(has_color(&mesh, SAW));
        assert!(has_color(&mesh, RANGE));
        assert!(has_color(&mesh, CANOPY));
        assert!(has_color(&mesh, WATER));
        assert!(has_color(&mesh, INCINERATOR));
        let points = positions(&mesh);
        assert!(points
            .iter()
            .any(|point| point[2] > 2.0 && point[0] > 120.0));
        let boats = lifeboat_mesh().expect("boats");
        assert!(has_color(&boats, BOAT));
        let boat_points = positions(&boats);
        assert!(boat_points
            .iter()
            .any(|point| point[1] < 0.0 && point[2] > 1.0));
        assert!(boat_points.iter().any(|point| point[1] > 65.0));
        let grove_canopy: Vec<_> = points
            .iter()
            .filter(|point| {
                (221.0..249.0).contains(&point[0])
                    && (18.0..48.0).contains(&point[1])
                    && point[2] > 2.0
            })
            .collect();
        assert!(!grove_canopy.is_empty());
        assert!(grove_canopy.iter().all(|point| point[1] < 33.0));
    }
}
