//! Cruise-ship deck outline in plan view (world **X** = aft→fore, **Y** = starboard→port).
//! Origin at the aft-starboard corner; footprint matches [`SHIP_LENGTH_M`] × [`SHIP_BEAM_M`].

use bevy::prelude::*;

/// Overall waterline length (bow tip to stern tip), metres — Oasis-class scale from reference plan.
pub const SHIP_LENGTH_M: f32 = 318.0;
/// Maximum beam (width), metres (reference Deck 10 plan).
pub const SHIP_BEAM_M: f32 = 60.0;

/// Half-width of the open central void (atrium / “Central Park”), metres (~22 m total ≈ 37% of beam).
pub const UPPER_VOID_HALF_WIDTH_M: f32 = 11.0;
/// Forward (bow) end of the void along **+X** (metres from aft origin).
pub const UPPER_VOID_X_FWD_M: f32 = SHIP_LENGTH_M * 0.5 + 50.0;
/// Aft end of the void along **+X** (metres from aft origin).
pub const UPPER_VOID_X_AFT_M: f32 = SHIP_LENGTH_M * 0.5 - 78.0;
/// First deck index (0-based) that uses the upper-deck footprint with courtyard + U-stern.
pub const FIRST_UPPER_DECK_STYLE_INDEX: usize = 9;

fn smoothstep01(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Map legacy centred `(port/starboard, stern/bow)` metres to corner-origin plan metres.
fn legacy_centered_to_plan(p: Vec2) -> Vec2 {
    let half_len = SHIP_LENGTH_M * 0.5;
    let half_beam = SHIP_BEAM_M * 0.5;
    Vec2::new(p.y + half_len, half_beam - p.x)
}

fn deck_hull_polygon_centered() -> Vec<Vec2> {
    let half_len = SHIP_LENGTH_M * 0.5;
    let half_beam = SHIP_BEAM_M * 0.5;
    let r_stern = half_beam;
    let y_stern_join = -half_len + r_stern;
    let y_bow_parallel = half_len - 0.22 * SHIP_LENGTH_M;

    let mut v = Vec::with_capacity(64);

    v.push(Vec2::new(0.0, half_len));

    for i in 0..12 {
        let t = (i + 1) as f32 / 12.0;
        let y = half_len - t * (half_len - y_bow_parallel);
        let w = half_beam * smoothstep01(t);
        v.push(Vec2::new(w, y));
    }

    v.push(Vec2::new(half_beam, y_stern_join));

    const ARC_SEGS: usize = 28;
    for i in 1..=ARC_SEGS {
        let t = i as f32 / ARC_SEGS as f32;
        let theta = -t * std::f32::consts::PI;
        v.push(Vec2::new(
            r_stern * theta.cos(),
            y_stern_join + r_stern * theta.sin(),
        ));
    }

    for i in (0..12).rev() {
        let t = (i + 1) as f32 / 12.0;
        let y = half_len - t * (half_len - y_bow_parallel);
        let w = half_beam * smoothstep01(t);
        v.push(Vec2::new(-w, y));
    }

    v
}

/// Closed deck outer boundary (CCW in XY, viewed from +Z). Bow at +X; origin aft-starboard.
pub fn deck_hull_polygon() -> Vec<Vec2> {
    deck_hull_polygon_centered()
        .into_iter()
        .map(legacy_centered_to_plan)
        .collect()
}

fn deck_hull_polygon_upper_centered() -> Vec<Vec2> {
    let half_len = SHIP_LENGTH_M * 0.5;
    let half_beam = SHIP_BEAM_M * 0.5;
    let vw = UPPER_VOID_HALF_WIDTH_M;
    let yvf = UPPER_VOID_X_FWD_M - half_len;
    let yva = UPPER_VOID_X_AFT_M - half_len;
    let y_bow_parallel = half_len - 0.22 * SHIP_LENGTH_M;

    let y_stern_outer = -half_len + 10.0;
    let r_wing = (half_beam - vw).max(4.0);
    let y_wing_inner = y_stern_outer + r_wing;

    let mut v = Vec::with_capacity(72);

    v.push(Vec2::new(0.0, half_len));

    for i in 0..12 {
        let t = (i + 1) as f32 / 12.0;
        let y = half_len - t * (half_len - y_bow_parallel);
        let w = half_beam * smoothstep01(t);
        v.push(Vec2::new(w, y));
    }

    v.push(Vec2::new(half_beam, y_stern_outer));

    const TIP_SEGS: usize = 8;
    for i in 1..=TIP_SEGS {
        let t = i as f32 / TIP_SEGS as f32;
        let theta = t * std::f32::consts::FRAC_PI_2;
        v.push(Vec2::new(
            vw + r_wing * theta.cos(),
            y_stern_outer + r_wing * theta.sin(),
        ));
    }

    v.push(Vec2::new(vw, yva));
    v.push(Vec2::new(vw, yvf));
    v.push(Vec2::new(-vw, yvf));
    v.push(Vec2::new(-vw, yva));
    v.push(Vec2::new(-vw, y_wing_inner));

    for i in 1..=TIP_SEGS {
        let t = i as f32 / TIP_SEGS as f32;
        let theta = std::f32::consts::FRAC_PI_2 + t * std::f32::consts::FRAC_PI_2;
        v.push(Vec2::new(
            -vw + r_wing * theta.cos(),
            y_stern_outer + r_wing * theta.sin(),
        ));
    }

    for i in (0..12).rev() {
        let t = (i + 1) as f32 / 12.0;
        let y = half_len - t * (half_len - y_bow_parallel);
        let w = half_beam * smoothstep01(t);
        v.push(Vec2::new(-w, y));
    }

    v
}

/// Upper-deck plan: bow fairing, long parallel sides, **U-shaped stern**, and central courtyard void.
pub fn deck_hull_polygon_upper() -> Vec<Vec2> {
    deck_hull_polygon_upper_centered()
        .into_iter()
        .map(legacy_centered_to_plan)
        .collect()
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn origin_is_outside_hull() {
        let poly = deck_hull_polygon();
        assert!(
            !point_in_polygon(Vec2::ZERO, &poly),
            "aft-starboard corner is outside the curved stern"
        );
    }

    #[test]
    fn upper_deck_excludes_atrium_void() {
        let poly = deck_hull_polygon_upper();
        let mid = Vec2::new(SHIP_LENGTH_M * 0.5, SHIP_BEAM_M * 0.5);
        assert!(
            !point_in_polygon(mid, &poly),
            "midship centreline should be open void"
        );
        let aft_void = Vec2::new(SHIP_LENGTH_M * 0.5 - 40.0, SHIP_BEAM_M * 0.5);
        assert!(!point_in_polygon(aft_void, &poly));
    }

    #[test]
    fn upper_deck_includes_side_cabins_and_bow() {
        let poly = deck_hull_polygon_upper();
        assert!(point_in_polygon(
            legacy_centered_to_plan(Vec2::new(24.0, 5.0)),
            &poly
        ));
        assert!(point_in_polygon(
            legacy_centered_to_plan(Vec2::new(0.0, 120.0)),
            &poly
        ));
    }

    #[test]
    fn upper_deck_stern_wings_and_open_gap() {
        let poly = deck_hull_polygon_upper();
        assert!(point_in_polygon(
            legacy_centered_to_plan(Vec2::new(29.0, -140.0)),
            &poly
        ));
        let gap = legacy_centered_to_plan(Vec2::new(0.0, -155.0));
        assert!(!point_in_polygon(gap, &poly));
    }
}
