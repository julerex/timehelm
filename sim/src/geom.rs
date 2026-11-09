//! Polygon helpers that do not depend on the ship outline.

use glam::Vec2;

use crate::hull::point_in_polygon;

/// Shoelace area in square metres.
pub fn polygon_area(poly: &[[f32; 2]]) -> f32 {
    if poly.len() < 3 {
        return 0.0;
    }
    let mut sum = 0.0;
    for i in 0..poly.len() {
        let j = (i + 1) % poly.len();
        sum += poly[i][0] * poly[j][1] - poly[j][0] * poly[i][1];
    }
    sum.abs() * 0.5
}

/// Vertex average. Rooms are rectangles, so this is the centre.
pub fn polygon_centroid(poly: &[[f32; 2]]) -> (f32, f32) {
    if poly.is_empty() {
        return (0.0, 0.0);
    }
    let n = poly.len() as f32;
    let (sx, sy) = poly
        .iter()
        .fold((0.0, 0.0), |acc, p| (acc.0 + p[0], acc.1 + p[1]));
    (sx / n, sy / n)
}

pub fn rect_polygon(x0: f32, y0: f32, x1: f32, y1: f32) -> Vec<[f32; 2]> {
    vec![[x0, y0], [x1, y0], [x1, y1], [x0, y1]]
}

/// Overlap of two axis-aligned edges. Shorter than a doorway, so it stays a wall.
const MIN_DOOR_OVERLAP_M: f32 = 1.0;
/// How far apart two edges can sit and still be the same wall line.
const EDGE_TOUCH_M: f32 = 0.8;

/// A straight run of shared wall between two rooms.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EdgeSpan {
    pub start: [f32; 2],
    pub end: [f32; 2],
}

impl EdgeSpan {
    pub fn midpoint(self) -> [f32; 2] {
        [
            (self.start[0] + self.end[0]) * 0.5,
            (self.start[1] + self.end[1]) * 0.5,
        ]
    }

    pub fn length(self) -> f32 {
        let dx = self.end[0] - self.start[0];
        let dy = self.end[1] - self.start[1];
        (dx * dx + dy * dy).sqrt()
    }
}

/// Longest shared edge, or nothing when the rooms only meet at a short nick.
pub fn shared_edge_span(a: &[[f32; 2]], b: &[[f32; 2]]) -> Option<EdgeSpan> {
    let mut best: Option<EdgeSpan> = None;
    for i in 0..a.len() {
        let a0 = a[i];
        let a1 = a[(i + 1) % a.len()];
        for j in 0..b.len() {
            let b0 = b[j];
            let b1 = b[(j + 1) % b.len()];
            let Some(span) = collinear_overlap(a0, a1, b0, b1) else {
                continue;
            };
            if best.is_none_or(|current| span.length() > current.length()) {
                best = Some(span);
            }
        }
    }
    best
}

/// True when the walk from `from` to `to` crosses `span`, including a stop on it.
pub fn segment_crosses_span(from: [f32; 2], to: [f32; 2], span: EdgeSpan) -> bool {
    const EPS: f32 = 1e-3;
    let c = span.start;
    let d = span.end;
    let o1 = orient(from, to, c);
    let o2 = orient(from, to, d);
    let o3 = orient(c, d, from);
    let o4 = orient(c, d, to);
    if o1.abs() <= EPS && point_on_segment(c, from, to) {
        return true;
    }
    if o2.abs() <= EPS && point_on_segment(d, from, to) {
        return true;
    }
    if o3.abs() <= EPS && point_on_segment(from, c, d) {
        return true;
    }
    if o4.abs() <= EPS && point_on_segment(to, c, d) {
        return true;
    }
    let straddles_path = (o1 > EPS && o2 < -EPS) || (o1 < -EPS && o2 > EPS);
    let straddles_span = (o3 > EPS && o4 < -EPS) || (o3 < -EPS && o4 > EPS);
    straddles_path && straddles_span
}

fn orient(a: [f32; 2], b: [f32; 2], c: [f32; 2]) -> f32 {
    (b[0] - a[0]) * (c[1] - a[1]) - (b[1] - a[1]) * (c[0] - a[0])
}

fn point_on_segment(point: [f32; 2], start: [f32; 2], end: [f32; 2]) -> bool {
    let cross =
        (point[1] - start[1]) * (end[0] - start[0]) - (point[0] - start[0]) * (end[1] - start[1]);
    if cross.abs() > 1e-2 {
        return false;
    }
    let dot =
        (point[0] - start[0]) * (end[0] - start[0]) + (point[1] - start[1]) * (end[1] - start[1]);
    if dot < -1e-2 {
        return false;
    }
    let len_sq = (end[0] - start[0]).powi(2) + (end[1] - start[1]).powi(2);
    dot <= len_sq + 1e-2
}

fn collinear_overlap(a0: [f32; 2], a1: [f32; 2], b0: [f32; 2], b1: [f32; 2]) -> Option<EdgeSpan> {
    let a_vertical = (a0[0] - a1[0]).abs() < 0.3;
    let b_vertical = (b0[0] - b1[0]).abs() < 0.3;
    if a_vertical && b_vertical && (a0[0] - b0[0]).abs() <= EDGE_TOUCH_M {
        let a_lo = a0[1].min(a1[1]);
        let a_hi = a0[1].max(a1[1]);
        let b_lo = b0[1].min(b1[1]);
        let b_hi = b0[1].max(b1[1]);
        let lo = a_lo.max(b_lo);
        let hi = a_hi.min(b_hi);
        if hi - lo > MIN_DOOR_OVERLAP_M {
            let x = (a0[0] + b0[0]) * 0.5;
            return Some(EdgeSpan {
                start: [x, lo],
                end: [x, hi],
            });
        }
        return None;
    }
    let a_horizontal = (a0[1] - a1[1]).abs() < 0.3;
    let b_horizontal = (b0[1] - b1[1]).abs() < 0.3;
    if a_horizontal && b_horizontal && (a0[1] - b0[1]).abs() <= EDGE_TOUCH_M {
        let a_lo = a0[0].min(a1[0]);
        let a_hi = a0[0].max(a1[0]);
        let b_lo = b0[0].min(b1[0]);
        let b_hi = b0[0].max(b1[0]);
        let lo = a_lo.max(b_lo);
        let hi = a_hi.min(b_hi);
        if hi - lo > MIN_DOOR_OVERLAP_M {
            let y = (a0[1] + b0[1]) * 0.5;
            return Some(EdgeSpan {
                start: [lo, y],
                end: [hi, y],
            });
        }
    }
    None
}

/// True when an inset of the rectangle lies inside `hull`.
pub fn rect_inside(hull: &[Vec2], x0: f32, y0: f32, x1: f32, y1: f32) -> bool {
    if x1 - x0 < 2.0 || y1 - y0 < 2.0 {
        return false;
    }
    let inset = 0.2;
    let points = [
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
        .all(|(x, y)| point_in_polygon(Vec2::new(x, y), hull))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_shared_edge_is_a_door_and_a_short_overlap_is_a_wall() {
        let cabin = rect_polygon(0.0, 0.0, 8.0, 8.0);
        let corridor = rect_polygon(0.0, 8.0, 16.0, 14.0);
        let span = shared_edge_span(&cabin, &corridor).expect("door");
        assert!(span.length() > 1.0);
        assert!(segment_crosses_span([4.0, 9.2], [4.0, 6.0], span));
        assert!(!segment_crosses_span([4.0, 9.2], [6.0, 9.2], span));

        let nick = rect_polygon(0.0, 8.0, 0.5, 12.0);
        assert!(shared_edge_span(&cabin, &nick).is_none());
        let gap = rect_polygon(0.0, 10.0, 8.0, 14.0);
        assert!(shared_edge_span(&cabin, &gap).is_none());
    }
}
