//! Icon of the Seas waterline, in plan metres.
//!
//! World **X** runs aft→fore, **Y** starboard→port. Origin is the aft-starboard
//! corner, outside the hull. Length and beam follow the published ship:
//! 365 m overall, 213 ft (65 m) maximum beam.
//!
//! The waterline is a fine bow, a long parallel mid-body, and a transom with
//! rounded quarters. It is not a rectangular slab and not a semicircular stern.
//! Cabin decks 9–12 and 14 leave Central Park open. The suite sun decks and the
//! slide tops step in from that waterline.

use glam::Vec2;

/// Overall length, bow tip to transom, metres.
pub const SHIP_LENGTH_M: f32 = 365.0;
/// Published maximum beam, 213 ft, metres.
pub const SHIP_BEAM_M: f32 = 65.0;

/// First deck index whose floor is open over Central Park (Icon deck 9).
pub const UPPER_DECK_START: usize = 9;

const BOW_RUN_M: f32 = 70.0;
const STERN_RADIUS_M: f32 = 18.0;
const BOW_POWER: f32 = 1.35;

/// Central Park opening on the cabin decks above Icon deck 8.
pub const PARK_X0: f32 = 118.0;
pub const PARK_X1: f32 = 242.0;
pub const PARK_Y0: f32 = 20.0;
pub const PARK_Y1: f32 = 45.0;

/// One or more plan outlines, plus openings that are not deck.
#[derive(Clone, Debug)]
pub struct DeckHull {
    pub outlines: Vec<Vec<Vec2>>,
    pub holes: Vec<Vec<Vec2>>,
}

/// Closed lower-deck (and deck 15) outer boundary.
pub fn deck_hull_polygon() -> Vec<Vec2> {
    waterline(
        0.0,
        SHIP_LENGTH_M,
        0.0,
        SHIP_BEAM_M,
        BOW_RUN_M,
        STERN_RADIUS_M,
    )
}

/// Outer boundary of a cabin deck. The park opening is a hole, not a bite out of this ring.
pub fn deck_hull_polygon_upper() -> Vec<Vec2> {
    deck_hull_polygon()
}

/// Hull outlines for a deck index.
pub fn deck_hull(deck: usize) -> DeckHull {
    match deck {
        9..=13 => DeckHull {
            outlines: vec![deck_hull_polygon()],
            holes: vec![rect(PARK_X0, PARK_Y0, PARK_X1, PARK_Y1)],
        },
        15 | 16 => DeckHull {
            outlines: vec![waterline(16.0, 350.0, 3.0, 62.0, 46.0, 14.0)],
            holes: Vec::new(),
        },
        17 | 18 => DeckHull {
            outlines: vec![
                waterline(196.0, 348.0, 8.0, 57.0, 40.0, 12.0),
                waterline(56.0, 110.0, 16.0, 52.0, 10.0, 8.0),
            ],
            holes: Vec::new(),
        },
        19 => DeckHull {
            outlines: vec![waterline(18.0, 132.0, 12.0, 54.0, 12.0, 12.0)],
            holes: Vec::new(),
        },
        _ => DeckHull {
            outlines: vec![deck_hull_polygon()],
            holes: Vec::new(),
        },
    }
}

/// Primary outline. Prefer [`deck_hull`] when holes or a second island matter.
pub fn hull_for_deck(deck: usize) -> Vec<Vec2> {
    deck_hull(deck)
        .outlines
        .into_iter()
        .next()
        .unwrap_or_default()
}

/// Ray-cast point-in-polygon.
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

/// Inside an outline and outside every opening.
pub fn point_on_deck(deck: usize, point: Vec2) -> bool {
    let hull = deck_hull(deck);
    hull.outlines
        .iter()
        .any(|outline| point_in_polygon(point, outline))
        && hull.holes.iter().all(|hole| !point_in_polygon(point, hole))
}

/// True when the whole rectangle, corners included, lies on the deck.
pub fn rect_on_deck(deck: usize, x0: f32, y0: f32, x1: f32, y1: f32) -> bool {
    if x1 - x0 < 2.0 || y1 - y0 < 2.0 {
        return false;
    }
    let inset = 0.3;
    let points = [
        (x0, y0),
        (x1, y0),
        (x1, y1),
        (x0, y1),
        (x0 + inset, y0 + inset),
        (x1 - inset, y0 + inset),
        (x1 - inset, y1 - inset),
        (x0 + inset, y1 - inset),
        ((x0 + x1) * 0.5, (y0 + y1) * 0.5),
        ((x0 + x1) * 0.5, y0 + inset),
        ((x0 + x1) * 0.5, y1 - inset),
        (x0 + inset, (y0 + y1) * 0.5),
        (x1 - inset, (y0 + y1) * 0.5),
    ];
    points
        .into_iter()
        .all(|(x, y)| point_on_deck(deck, Vec2::new(x, y)))
}

fn rect(x0: f32, y0: f32, x1: f32, y1: f32) -> Vec<Vec2> {
    vec![
        Vec2::new(x0, y0),
        Vec2::new(x1, y0),
        Vec2::new(x1, y1),
        Vec2::new(x0, y1),
    ]
}

/// Ship-shaped ring. `bow_run` is the fine entry; `stern_r` is the transom quarter radius.
fn waterline(
    x_aft: f32,
    x_fore: f32,
    y_stbd: f32,
    y_port: f32,
    bow_run: f32,
    stern_r: f32,
) -> Vec<Vec2> {
    let length = (x_fore - x_aft).max(8.0);
    let beam = (y_port - y_stbd).max(4.0);
    let bow = bow_run.clamp(0.0, length * 0.45);
    let radius = stern_r.min(beam * 0.45).min(length * 0.45).max(0.0);
    let center_y = (y_stbd + y_port) * 0.5;
    let half = beam * 0.5;
    let bow_start = x_fore - bow;
    let stern_parallel = x_aft + radius;

    let mut v = Vec::with_capacity(48);
    v.push(Vec2::new(x_fore, center_y));

    const BOW_SEGS: usize = 14;
    if bow > 1.0 {
        for i in 1..=BOW_SEGS {
            let t = i as f32 / BOW_SEGS as f32;
            let x = x_fore - bow * t;
            let half_w = half * t.powf(BOW_POWER);
            v.push(Vec2::new(x, center_y - half_w));
        }
    } else {
        v.push(Vec2::new(x_fore, y_stbd));
    }

    v.push(Vec2::new(stern_parallel, y_stbd));
    push_quarter(
        &mut v,
        Vec2::new(stern_parallel, y_stbd + radius),
        radius,
        -std::f32::consts::FRAC_PI_2,
        -std::f32::consts::PI,
    );
    v.push(Vec2::new(x_aft, y_port - radius));
    push_quarter(
        &mut v,
        Vec2::new(stern_parallel, y_port - radius),
        radius,
        std::f32::consts::PI,
        std::f32::consts::FRAC_PI_2,
    );
    v.push(Vec2::new(bow_start, y_port));

    if bow > 1.0 {
        for i in (1..BOW_SEGS).rev() {
            let t = i as f32 / BOW_SEGS as f32;
            let x = x_fore - bow * t;
            let half_w = half * t.powf(BOW_POWER);
            v.push(Vec2::new(x, center_y + half_w));
        }
    }

    dedup(&mut v);
    v
}

fn push_quarter(points: &mut Vec<Vec2>, center: Vec2, radius: f32, from: f32, to: f32) {
    if radius < 0.5 {
        return;
    }
    const SEGS: usize = 6;
    for i in 1..=SEGS {
        let t = i as f32 / SEGS as f32;
        let theta = from + (to - from) * t;
        points.push(Vec2::new(
            center.x + radius * theta.cos(),
            center.y + radius * theta.sin(),
        ));
    }
}

fn dedup(points: &mut Vec<Vec2>) {
    points.dedup_by(|a, b| (*a - *b).length_squared() < 1e-4);
    let closed = points.len() >= 2
        && points
            .first()
            .zip(points.last())
            .is_some_and(|(first, last)| (*first - *last).length_squared() < 1e-4);
    if closed {
        points.pop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn origin_is_outside_the_transom_quarter() {
        let poly = deck_hull_polygon();
        assert!(!point_in_polygon(Vec2::ZERO, &poly));
        assert!(point_in_polygon(Vec2::new(4.0, SHIP_BEAM_M * 0.5), &poly));
        assert!(!point_in_polygon(Vec2::new(4.0, 2.0), &poly));
        assert!(point_in_polygon(Vec2::new(30.0, 1.0), &poly));
    }

    #[test]
    fn bow_is_finer_than_the_midship() {
        let poly = deck_hull_polygon();
        assert!(point_in_polygon(Vec2::new(180.0, 2.0), &poly));
        assert!(!point_in_polygon(Vec2::new(352.0, 2.0), &poly));
        assert!(point_in_polygon(Vec2::new(352.0, SHIP_BEAM_M * 0.5), &poly));
        assert!(!point_in_polygon(
            Vec2::new(SHIP_LENGTH_M + 2.0, SHIP_BEAM_M * 0.5),
            &poly
        ));
    }

    #[test]
    fn central_park_is_open_above_its_floor() {
        let park = Vec2::new((PARK_X0 + PARK_X1) * 0.5, (PARK_Y0 + PARK_Y1) * 0.5);
        assert!(point_on_deck(8, park), "deck 8 is the park floor");
        assert!(!point_on_deck(9, park));
        assert!(
            !point_on_deck(13, park),
            "Icon deck 14 still opens onto the park"
        );
        assert!(point_on_deck(14, park), "deck 15 is the pool plate");
        assert!(point_on_deck(9, Vec2::new(180.0, 6.0)));
    }

    #[test]
    fn sun_deck_and_slide_tops_step_in() {
        assert!(point_on_deck(17, Vec2::new(230.0, 32.0)));
        assert!(!point_on_deck(17, Vec2::new(150.0, 32.0)));
        assert!(point_on_deck(17, Vec2::new(78.0, 32.0)), "slide tower");
        assert!(point_on_deck(19, Vec2::new(50.0, 32.0)));
        assert!(!point_on_deck(19, Vec2::new(230.0, 32.0)));
    }
}
