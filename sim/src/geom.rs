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
