//! Cruise-ship deck outline in plan view (world **X** = port/starboard, **Y** = stern→bow).
//! Footprint matches Deck 10 reference scale ([`SHIP_BEAM_M`] × [`SHIP_LENGTH_M`]).

use bevy::prelude::*;

/// Overall waterline length (bow tip to stern tip), metres — slightly under 365 m (Deck 10 plan).
pub const SHIP_LENGTH_M: f32 = 362.0;
/// Maximum beam (width), metres (Deck 10: 60 m).
pub const SHIP_BEAM_M: f32 = 60.0;

fn smoothstep01(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Closed deck outer boundary (CCW in XY, viewed from +Z). Symmetric; bow at +Y.
pub fn deck_hull_polygon() -> Vec<Vec2> {
    let half_len = SHIP_LENGTH_M * 0.5;
    let half_beam = SHIP_BEAM_M * 0.5;
    let r_stern = half_beam;
    // Parallel section meets stern arc where circle (center on centreline) hits x = ±half_beam.
    let y_stern_join = -half_len + r_stern;
    let y_bow_parallel = half_len - 0.22 * SHIP_LENGTH_M;

    let mut v = Vec::with_capacity(64);

    // Bow tip (+Y forward)
    v.push(Vec2::new(0.0, half_len));

    // Starboard bow fairing → full beam (12 segments; last point is parallel corner)
    for i in 0..12 {
        let t = (i + 1) as f32 / 12.0;
        let y = half_len - t * (half_len - y_bow_parallel);
        let w = half_beam * smoothstep01(t);
        v.push(Vec2::new(w, y));
    }

    // Starboard parallel body (corner already at y_bow_parallel)
    v.push(Vec2::new(half_beam, y_stern_join));

    // Stern: semicircle aft, centre (0, y_stern_join), radius r_stern, θ from 0 → −π
    // Start at i = 1 so the first arc point is not duplicated with the starboard parallel corner.
    const ARC_SEGS: usize = 28;
    for i in 1..=ARC_SEGS {
        let t = i as f32 / ARC_SEGS as f32;
        let theta = -t * std::f32::consts::PI;
        v.push(Vec2::new(
            r_stern * theta.cos(),
            y_stern_join + r_stern * theta.sin(),
        ));
    }

    // Arc ends at (-half_beam, y_stern_join); port bow fairing back to tip (no duplicate vertex)
    for i in (0..12).rev() {
        let t = (i + 1) as f32 / 12.0;
        let y = half_len - t * (half_len - y_bow_parallel);
        let w = half_beam * smoothstep01(t);
        v.push(Vec2::new(-w, y));
    }

    v
}

/// Ray-cast point-in-polygon (robust for deck footprint).
pub fn point_in_polygon(point: Vec2, poly: &[Vec2]) -> bool {
    if poly.len() < 3 {
        return false;
    }
    let (x, y) = (point.x, point.y);
    let mut inside = false;
    let n = poly.len();
    for i in 0..n {
        let j = (i + 1) % n;
        let (xi, yi) = (poly[i].x, poly[i].y);
        let (xj, yj) = (poly[j].x, poly[j].y);
        let denom = yj - yi;
        if denom.abs() < 1e-8 {
            continue;
        }
        let intersect = ((yi > y) != (yj > y)) && (x < (xj - xi) * (y - yi) / denom + xi);
        if intersect {
            inside = !inside;
        }
    }
    inside
}

/// Cell centres for axis-aligned squares of side `step_m` inside the hull.
pub fn deck_tile_centers(step_m: f32) -> Vec<Vec2> {
    let poly = deck_hull_polygon();
    let half_len = SHIP_LENGTH_M * 0.5;
    let pad = step_m;
    let half_b = SHIP_BEAM_M * 0.5 + pad;
    let mut out = Vec::new();
    let mut cy = -half_len + step_m * 0.5;
    while cy <= half_len - step_m * 0.25 {
        let mut cx = -half_b + step_m * 0.5;
        while cx <= half_b - step_m * 0.25 {
            let p = Vec2::new(cx, cy);
            if point_in_polygon(p, &poly) {
                out.push(p);
            }
            cx += step_m;
        }
        cy += step_m;
    }
    out
}

pub fn is_perimeter_tile(center: Vec2, step_m: f32, poly: &[Vec2]) -> bool {
    const NEI: [(f32, f32); 4] = [(1.0, 0.0), (-1.0, 0.0), (0.0, 1.0), (0.0, -1.0)];
    for (kx, ky) in NEI {
        let q = Vec2::new(center.x + kx * step_m, center.y + ky * step_m);
        if !point_in_polygon(q, poly) {
            return true;
        }
    }
    false
}
