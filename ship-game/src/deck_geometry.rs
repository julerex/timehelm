//! Extruded room floors and the deck-plan texture sampled by the clip shader.

use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

/// Caps + skirt walls for a simple XY polygon extruded along **local Z** (thickness centred on z = 0).
/// `length_m` / `beam_m` scale UVs for polygon vertices (`x` along ship, `y` across beam).
pub fn extruded_polygon_deck_mesh(
    poly: &[Vec2],
    thickness_m: f32,
    length_m: f32,
    beam_m: f32,
) -> Mesh {
    let n = poly.len();
    let half_z = thickness_m * 0.5;
    let uv_xy = |x: f32, y: f32| [x / length_m, y / beam_m];

    if n < 3 {
        return Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        );
    }

    let flat: Vec<f64> = poly.iter().flat_map(|p| [p.x as f64, p.y as f64]).collect();
    let Ok(idx_top) = earcutr::earcut(&flat, &[], 2) else {
        return Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        );
    };

    let mut positions = Vec::<[f32; 3]>::new();
    let mut normals = Vec::<[f32; 3]>::new();
    let mut uvs = Vec::<[f32; 2]>::new();
    let white: [f32; 4] = [1.0, 1.0, 1.0, 1.0];
    let mut colors = Vec::<[f32; 4]>::new();

    for p in poly {
        positions.push([p.x, p.y, half_z]);
        normals.push([0.0, 0.0, 1.0]);
        uvs.push(uv_xy(p.x, p.y));
        colors.push(white);
    }
    let bot_base = positions.len() as u32;
    for p in poly {
        positions.push([p.x, p.y, -half_z]);
        normals.push([0.0, 0.0, -1.0]);
        uvs.push(uv_xy(p.x, p.y));
        colors.push(white);
    }

    let mut indices: Vec<u32> = Vec::new();
    for t in idx_top.chunks(3) {
        indices.extend_from_slice(&[t[0] as u32, t[1] as u32, t[2] as u32]);
    }
    for t in idx_top.chunks(3) {
        indices.extend_from_slice(&[
            bot_base + t[2] as u32,
            bot_base + t[1] as u32,
            bot_base + t[0] as u32,
        ]);
    }

    for i in 0..n {
        let j = (i + 1) % n;
        let pi = poly[i];
        let pj = poly[j];
        let ex = pj.x - pi.x;
        let ey = pj.y - pi.y;
        let el = (ex * ex + ey * ey).sqrt().max(1e-6);
        let nx = ey / el;
        let ny = -ex / el;
        let base = positions.len() as u32;
        let um = uv_xy((pi.x + pj.x) * 0.5, (pi.y + pj.y) * 0.5);
        positions.push([pi.x, pi.y, half_z]);
        positions.push([pj.x, pj.y, half_z]);
        positions.push([pj.x, pj.y, -half_z]);
        positions.push([pi.x, pi.y, -half_z]);
        for _ in 0..4 {
            normals.push([nx, ny, 0.0]);
            uvs.push(um);
            colors.push(white);
        }
        indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
    }

    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
    .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, colors)
    .with_inserted_indices(Indices::U32(indices))
}

/// Low-frequency deck “plan” tint sampled in [`super::ShipClipMaterial`] (multiplies vertex colour).
pub fn procedural_deck_plan_texture_image() -> Image {
    let w = 160u32;
    let h = 160u32;
    let mut rgba = vec![0u8; (w * h * 4) as usize];
    for y in 0..h {
        for x in 0..w {
            let stripe_x = (x / 10) % 2 == 0;
            let stripe_y = (y / 12) % 2 == 0;
            let hull_ring = x.min(w - 1 - x) < 12 || y.min(h - 1 - y) < 12;
            let base = if stripe_x ^ stripe_y {
                [235u8, 228, 218, 255]
            } else {
                [210, 205, 196, 255]
            };
            let tint = if hull_ring {
                [200u8, 175, 155, 255]
            } else {
                base
            };
            let i = ((y * w + x) * 4) as usize;
            rgba[i..i + 4].copy_from_slice(&tint);
        }
    }
    Image::new_fill(
        Extent3d {
            width: w,
            height: h,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &rgba,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    )
}

const HULL_PLATE_COLOR: [f32; 4] = [0.56, 0.63, 0.70, 1.0];
/// Half-thickness of [`ship_hull_plate`]. Local Z runs from `-half` to `+half`.
pub const HULL_PLATE_HALF_THICKNESS_M: f32 = 0.09;

/// Deck plate for one hull outline. Holes are left open. Local Z is centred on 0.
pub fn ship_hull_plate(outline: &[[f32; 2]], holes: &[Vec<[f32; 2]>]) -> Option<Mesh> {
    if outline.len() < 3 {
        return None;
    }
    let mut flat = Vec::new();
    let mut verts = Vec::new();
    for point in outline {
        flat.push(f64::from(point[0]));
        flat.push(f64::from(point[1]));
        verts.push(*point);
    }
    let mut hole_starts = Vec::new();
    for hole in holes {
        if hole.len() < 3 {
            continue;
        }
        hole_starts.push(verts.len());
        for point in hole {
            flat.push(f64::from(point[0]));
            flat.push(f64::from(point[1]));
            verts.push(*point);
        }
    }
    let Ok(tris) = earcutr::earcut(&flat, &hole_starts, 2) else {
        return None;
    };
    if tris.len() < 3 {
        return None;
    }
    let half = HULL_PLATE_HALF_THICKNESS_M;
    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    let mut colors = Vec::new();
    for point in &verts {
        positions.push([point[0], point[1], half]);
        normals.push([0.0, 0.0, 1.0]);
        uvs.push([0.0, 0.0]);
        colors.push(HULL_PLATE_COLOR);
    }
    let bottom = positions.len() as u32;
    for point in &verts {
        positions.push([point[0], point[1], -half]);
        normals.push([0.0, 0.0, -1.0]);
        uvs.push([0.0, 0.0]);
        colors.push(HULL_PLATE_COLOR);
    }
    let reverse = signed_area(outline) < 0.0;
    let mut indices = Vec::new();
    for tri in tris.chunks(3) {
        let (a, b, c) = if reverse {
            (tri[0] as u32, tri[2] as u32, tri[1] as u32)
        } else {
            (tri[0] as u32, tri[1] as u32, tri[2] as u32)
        };
        indices.extend_from_slice(&[a, b, c]);
        indices.extend_from_slice(&[bottom + a, bottom + c, bottom + b]);
    }
    Some(mesh_from(positions, normals, uvs, colors, indices))
}

/// Vertical shell around a ring. The band sits outside the polygon, from `z0` to `z1`.
pub fn ship_bulwark_mesh(
    rings: &[Vec<[f32; 2]>],
    z0: f32,
    z1: f32,
    thickness: f32,
    color: [f32; 4],
) -> Option<Mesh> {
    let mut mesh = BuiltMesh::default();
    for ring in rings {
        if ring.len() < 2 {
            continue;
        }
        for index in 0..ring.len() {
            let a = ring[index];
            let b = ring[(index + 1) % ring.len()];
            let dx = b[0] - a[0];
            let dy = b[1] - a[1];
            let len = (dx * dx + dy * dy).sqrt();
            if len < 0.05 {
                continue;
            }
            let outward = outward_normal(ring, dx / len, dy / len);
            let ao = [a[0] + outward[0] * thickness, a[1] + outward[1] * thickness];
            let bo = [b[0] + outward[0] * thickness, b[1] + outward[1] * thickness];
            mesh.push_quad(
                [
                    [ao[0], ao[1], z0],
                    [bo[0], bo[1], z0],
                    [bo[0], bo[1], z1],
                    [ao[0], ao[1], z1],
                ],
                [outward[0], outward[1], 0.0],
                color,
            );
            mesh.push_quad(
                [
                    [b[0], b[1], z0],
                    [a[0], a[1], z0],
                    [a[0], a[1], z1],
                    [b[0], b[1], z1],
                ],
                [-outward[0], -outward[1], 0.0],
                color,
            );
            mesh.push_quad(
                [
                    [a[0], a[1], z1],
                    [b[0], b[1], z1],
                    [bo[0], bo[1], z1],
                    [ao[0], ao[1], z1],
                ],
                [0.0, 0.0, 1.0],
                color,
            );
        }
    }
    mesh.into_mesh()
}

fn outward_normal(ring: &[[f32; 2]], dx: f32, dy: f32) -> [f32; 2] {
    if signed_area(ring) < 0.0 {
        [-dy, dx]
    } else {
        [dy, -dx]
    }
}

fn signed_area(ring: &[[f32; 2]]) -> f32 {
    let mut sum = 0.0;
    for index in 0..ring.len() {
        let next = (index + 1) % ring.len();
        sum += ring[index][0] * ring[next][1] - ring[next][0] * ring[index][1];
    }
    sum * 0.5
}

#[derive(Default)]
struct BuiltMesh {
    positions: Vec<[f32; 3]>,
    normals: Vec<[f32; 3]>,
    uvs: Vec<[f32; 2]>,
    colors: Vec<[f32; 4]>,
    indices: Vec<u32>,
}

impl BuiltMesh {
    fn push_quad(&mut self, mut quad: [[f32; 3]; 4], want: [f32; 3], color: [f32; 4]) {
        let normal = face_normal(quad[0], quad[1], quad[2]);
        let facing = normal[0] * want[0] + normal[1] * want[1] + normal[2] * want[2];
        let normal = if facing >= 0.0 {
            normal
        } else {
            quad = [quad[0], quad[3], quad[2], quad[1]];
            [-normal[0], -normal[1], -normal[2]]
        };
        let base = self.positions.len() as u32;
        for point in quad {
            self.positions.push(point);
            self.normals.push(normal);
            self.uvs.push([0.0, 0.0]);
            self.colors.push(color);
        }
        self.indices
            .extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
    }

    fn into_mesh(self) -> Option<Mesh> {
        if self.indices.is_empty() {
            None
        } else {
            Some(mesh_from(
                self.positions,
                self.normals,
                self.uvs,
                self.colors,
                self.indices,
            ))
        }
    }
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

fn mesh_from(
    positions: Vec<[f32; 3]>,
    normals: Vec<[f32; 3]>,
    uvs: Vec<[f32; 2]>,
    colors: Vec<[f32; 4]>,
    indices: Vec<u32>,
) -> Mesh {
    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
    .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, colors)
    .with_inserted_indices(Indices::U32(indices))
}

#[cfg(test)]
mod tests {
    use super::*;
    use sim::deck_hull;

    fn ring(poly: &[bevy::math::Vec2]) -> Vec<[f32; 2]> {
        poly.iter().map(|point| [point.x, point.y]).collect()
    }

    #[test]
    fn hull_plate_reaches_the_bow_and_accepts_the_park_hole() {
        let lower = deck_hull(0);
        let outline = ring(&lower.outlines[0]);
        let plate = ship_hull_plate(&outline, &[]).expect("waterline plate");
        let positions = plate
            .attribute(Mesh::ATTRIBUTE_POSITION)
            .and_then(|values| match values {
                bevy::mesh::VertexAttributeValues::Float32x3(values) => Some(values),
                _ => None,
            })
            .expect("positions");
        assert!(positions.iter().any(|point| point[0] > 360.0));
        assert!(positions.iter().any(|point| point[0] < 1.0));

        let park = deck_hull(9);
        let outline = ring(&park.outlines[0]);
        let holes: Vec<Vec<[f32; 2]>> = park.holes.iter().map(|hole| ring(hole)).collect();
        assert!(ship_hull_plate(&outline, &holes).is_some());
        let rail = ship_bulwark_mesh(&holes, 0.0, 1.1, 0.28, [0.7, 0.8, 0.78, 1.0]);
        assert!(rail.is_some());
    }
}
