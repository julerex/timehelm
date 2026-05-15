//! Extruded hull meshes (LOD coarse), procedural deck-plan texture, and bucket geometry helpers.

use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, PrimitiveTopology, VertexAttributeValues};
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

/// Caps + skirt walls for a simple XY polygon extruded along **local Z** (thickness centred on z = 0).
pub fn extruded_polygon_deck_mesh(
    poly: &[Vec2],
    thickness_m: f32,
    beam_m: f32,
    length_m: f32,
) -> Mesh {
    let n = poly.len();
    let half_z = thickness_m * 0.5;
    let uv_xy = |x: f32, y: f32| [x / beam_m + 0.5, y / length_m + 0.5];

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

/// Merge translated cuboids (`prototype` includes [`Mesh::ATTRIBUTE_COLOR`]) or `None` when empty.
pub fn accumulate_translated_tile_instances(
    prototype: &Mesh,
    translations: &[Vec3],
) -> Option<Mesh> {
    if translations.is_empty() {
        return None;
    }
    let VertexAttributeValues::Float32x3(template_pos) =
        prototype.attribute(Mesh::ATTRIBUTE_POSITION)?
    else {
        return None;
    };
    let VertexAttributeValues::Float32x3(template_norm) =
        prototype.attribute(Mesh::ATTRIBUTE_NORMAL)?
    else {
        return None;
    };
    let VertexAttributeValues::Float32x2(template_uv) =
        prototype.attribute(Mesh::ATTRIBUTE_UV_0)?
    else {
        return None;
    };
    let VertexAttributeValues::Float32x4(template_col) =
        prototype.attribute(Mesh::ATTRIBUTE_COLOR)?
    else {
        return None;
    };
    let template_indices = match prototype.indices()? {
        Indices::U32(v) => v.as_slice(),
        Indices::U16(_) => return None,
    };

    let nv = template_pos.len();
    let nt = translations.len();
    debug_assert_eq!(template_norm.len(), nv);
    debug_assert_eq!(template_uv.len(), nv);
    debug_assert_eq!(template_col.len(), nv);

    let mut pos = Vec::with_capacity(nv * nt);
    let mut norm = Vec::with_capacity(nv * nt);
    let mut uv = Vec::with_capacity(nv * nt);
    let mut col = Vec::with_capacity(nv * nt);
    let mut idx = Vec::with_capacity(template_indices.len() * nt);

    for (ti, t) in translations.iter().enumerate() {
        let base = (ti * nv) as u32;
        for p in template_pos.iter() {
            pos.push([p[0] + t.x, p[1] + t.y, p[2] + t.z]);
        }
        norm.extend_from_slice(template_norm);
        uv.extend_from_slice(template_uv);
        col.extend_from_slice(template_col);
        for i in template_indices.iter().copied() {
            idx.push(i + base);
        }
    }

    Some(
        Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        )
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, pos)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, norm)
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uv)
        .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, col)
        .with_inserted_indices(Indices::U32(idx)),
    )
}

/// Merge axis-aligned XY squares centred at each point (camera down **−Z**, quads lie in XY at `z`).
pub fn merged_plan_squares_mesh(centers: &[Vec2], half_size: f32, z: f32) -> Mesh {
    let mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );
    if centers.is_empty() {
        return mesh;
    }
    let mut positions = Vec::with_capacity(centers.len() * 4);
    let mut normals = Vec::with_capacity(centers.len() * 4);
    let mut uvs = Vec::with_capacity(centers.len() * 4);
    let mut indices = Vec::with_capacity(centers.len() * 6);
    let h = half_size;
    for (i, c) in centers.iter().enumerate() {
        let base = (i * 4) as u32;
        positions.push([c.x - h, c.y - h, z]);
        positions.push([c.x + h, c.y - h, z]);
        positions.push([c.x + h, c.y + h, z]);
        positions.push([c.x - h, c.y + h, z]);
        for _ in 0..4 {
            normals.push([0.0, 0.0, 1.0]);
            uvs.push([0.0, 0.0]);
        }
        indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
    }
    mesh.with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
        .with_inserted_indices(Indices::U32(indices))
}
