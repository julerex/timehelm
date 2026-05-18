//! Baked 2D plan meshes (floor quads + wall strokes) per deck.

use crate::cell_box;
use crate::deck_geometry::{
    merged_plan_rectangles_mesh_colored, merged_plan_wall_borders_mesh_varied_colored,
    PlanWallEdge,
};
use crate::deck_layout::{DeckCells, DeckLayouts, CELL_VISUAL_SCALE, NUM_DECKS};
use bevy::prelude::*;

const Z_CELL_PLANE: f32 = 0.0;
const Z_WALL_PLANE: f32 = 0.001;
/// Plan-view wall stroke width (10 cm); each cell draws its own edges.
pub const PLAN_WALL_THICKNESS_M: f32 = 0.10;

#[derive(Resource)]
pub struct DeckPlanMeshes {
    pub floors: [Handle<Mesh>; NUM_DECKS],
    pub walls: [Handle<Mesh>; NUM_DECKS],
}

fn color_to_linear_array(c: Color) -> [f32; 4] {
    let lr: LinearRgba = c.into();
    [lr.red, lr.green, lr.blue, lr.alpha]
}

/// World-plan half extents: **X** = along-ship, **Y** = across beam (matches [`cell_box::CellIndex::to_world_xy`]).
fn plan_cell_half_extents() -> (f32, f32) {
    (
        cell_box::length_cell_m() * CELL_VISUAL_SCALE * 0.5,
        cell_box::beam_cell_m() * CELL_VISUAL_SCALE * 0.5,
    )
}

pub fn build_deck_mesh(deck_index: usize, deck: DeckCells<'_>) -> Mesh {
    let (half_x, half_y) = plan_cell_half_extents();
    let cells: Vec<_> = deck.iter_cells().collect();
    let mut centers = Vec::with_capacity(cells.len());
    let mut colors = Vec::with_capacity(cells.len());
    for (plan, cell) in cells {
        let p = deck.index(plan).to_world_xy();
        centers.push(p);
        colors.push(color_to_linear_array(
            cell.floor.plan_floor_color(deck_index),
        ));
    }
    merged_plan_rectangles_mesh_colored(&centers, &colors, half_x, half_y, Z_CELL_PLANE)
}

fn collect_wall_edges(deck: DeckCells<'_>) -> Vec<(PlanWallEdge, f32, [f32; 4])> {
    let (half_x, half_y) = plan_cell_half_extents();
    let mut edges = Vec::new();
    for (plan, cell) in deck.iter_cells() {
        let c = deck.index(plan).to_world_xy();
        let y0 = c.y - half_y;
        let y1 = c.y + half_y;
        let x0 = c.x - half_x;
        let x1 = c.x + half_x;

        let sides = [
            (cell.side1, PlanWallEdge::Vertical { x: x1, y0, y1 }),
            (cell.side2, PlanWallEdge::Horizontal { y: y1, x0, x1 }),
            (cell.side3, PlanWallEdge::Vertical { x: x0, y0, y1 }),
            (cell.side4, PlanWallEdge::Horizontal { y: y0, x0, x1 }),
        ];
        for (material, edge) in sides {
            if material.draws_plan_stroke() {
                edges.push((
                    edge,
                    PLAN_WALL_THICKNESS_M,
                    color_to_linear_array(material.plan_stroke_color()),
                ));
            }
        }
    }
    edges
}

pub fn build_deck_wall_mesh(deck: DeckCells<'_>) -> Mesh {
    let edges = collect_wall_edges(deck);
    merged_plan_wall_borders_mesh_varied_colored(&edges, Z_WALL_PLANE)
}

/// Fine-LOD 3D wall boxes (centre, half-extents, linear RGBA) for non-open sides.
pub fn collect_deck_side_walls_3d(
    deck: DeckCells<'_>,
    deck_z: f32,
    wall_height_m: f32,
) -> Vec<(Vec3, Vec3, [f32; 4])> {
    let (half_x, half_y) = plan_cell_half_extents();
    let half_t = PLAN_WALL_THICKNESS_M * 0.5;
    let half_h = wall_height_m * 0.5;
    let z_center = deck_z + half_h;
    let mut walls = Vec::new();

    for (plan, cell) in deck.iter_cells() {
        let c = deck.index(plan).to_world_xy();
        let y0 = c.y - half_y;
        let y1 = c.y + half_y;
        let x0 = c.x - half_x;
        let x1 = c.x + half_x;

        let sides = [
            (cell.side1, Vec3::new(x1, c.y, z_center), Vec3::new(half_t, half_y, half_h)),
            (
                cell.side2,
                Vec3::new(c.x, y1, z_center),
                Vec3::new(half_x, half_t, half_h),
            ),
            (cell.side3, Vec3::new(x0, c.y, z_center), Vec3::new(half_t, half_y, half_h)),
            (
                cell.side4,
                Vec3::new(c.x, y0, z_center),
                Vec3::new(half_x, half_t, half_h),
            ),
        ];
        for (material, center, half) in sides {
            if material.draws_plan_stroke() {
                let lr: LinearRgba = material.plan_stroke_color().into();
                walls.push((center, half, lr.to_f32_array()));
            }
        }
    }
    walls
}

pub fn rebuild_plan_deck_meshes(
    deck_i: usize,
    layouts: &DeckLayouts,
    meshes: &mut Assets<Mesh>,
    handles: &DeckPlanMeshes,
) {
    let deck = layouts.deck(deck_i);
    if let Some(mesh) = meshes.get_mut(&handles.floors[deck_i]) {
        *mesh = build_deck_mesh(deck_i, deck);
    }
    if let Some(mesh) = meshes.get_mut(&handles.walls[deck_i]) {
        *mesh = build_deck_wall_mesh(deck);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cell_box::CellIndex;

    #[test]
    fn plan_wall_strokes_are_ten_cm() {
        assert!((PLAN_WALL_THICKNESS_M - 0.10).abs() < 1e-6);
    }

    #[test]
    fn plan_cell_half_extents_tile_without_gaps() {
        let (half_x, half_y) = plan_cell_half_extents();
        let a = CellIndex::new(10, 30, 0).unwrap();
        let port = a.offset(0, 1, 0).unwrap();
        let gap_y = (port.to_world_xy().y - half_y) - (a.to_world_xy().y + half_y);
        assert!(
            gap_y.abs() < 1e-5,
            "beam-axis gap between neighbours: {gap_y}"
        );
        let forward = a.offset(1, 0, 0).unwrap();
        let gap_x = (forward.to_world_xy().x - half_x) - (a.to_world_xy().x + half_x);
        assert!(
            gap_x.abs() < 1e-5,
            "length-axis gap between neighbours: {gap_x}"
        );
    }
}
