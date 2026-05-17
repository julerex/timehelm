//! Baked 2D plan meshes (floor quads + wall strokes) per deck.

use crate::cell::Material;
use crate::cell_box;
use crate::deck_geometry::{
    merged_plan_squares_mesh_colored, merged_plan_wall_borders_mesh_varied, PlanWallEdge,
};
use crate::deck_layout::{DeckCells, DeckLayouts, CELL_VISUAL_SCALE, NUM_DECKS};
use bevy::prelude::*;

const Z_CELL_PLANE: f32 = 0.0;
const Z_WALL_PLANE: f32 = 0.001;
/// Plan-view wall stroke width (10 cm); each cell draws its own edges.
const PLAN_WALL_THICKNESS_M: f32 = 0.10;
const WALL_BORDER_COLOR: [f32; 4] = [0.0, 0.0, 0.0, 1.0];

#[derive(Resource)]
pub struct DeckPlanMeshes {
    pub floors: [Handle<Mesh>; NUM_DECKS],
    pub walls: [Handle<Mesh>; NUM_DECKS],
}

fn color_to_linear_array(c: Color) -> [f32; 4] {
    let lr: LinearRgba = c.into();
    [lr.red, lr.green, lr.blue, lr.alpha]
}

pub fn build_deck_mesh(deck_index: usize, deck: DeckCells<'_>) -> Mesh {
    let half_x = cell_box::length_cell_m() * CELL_VISUAL_SCALE * 0.5;
    let half_y = cell_box::beam_cell_m() * CELL_VISUAL_SCALE * 0.5;
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
    merged_plan_squares_mesh_colored(&centers, &colors, half_x.min(half_y), Z_CELL_PLANE)
}

fn collect_wall_edges(deck: DeckCells<'_>) -> Vec<(PlanWallEdge, f32)> {
    let half_x = cell_box::length_cell_m() * CELL_VISUAL_SCALE * 0.5;
    let half_y = cell_box::beam_cell_m() * CELL_VISUAL_SCALE * 0.5;
    let mut edges = Vec::new();
    for (plan, cell) in deck.iter_cells() {
        let c = deck.index(plan).to_world_xy();
        let y0 = c.y - half_x;
        let y1 = c.y + half_x;
        let x0 = c.x - half_y;
        let x1 = c.x + half_y;

        if cell.wall1 != Material::Open {
            edges.push((
                PlanWallEdge::Vertical { x: x1, y0, y1 },
                PLAN_WALL_THICKNESS_M,
            ));
        }
        if cell.wall2 != Material::Open {
            edges.push((
                PlanWallEdge::Horizontal { y: y1, x0, x1 },
                PLAN_WALL_THICKNESS_M,
            ));
        }
        if cell.wall3 != Material::Open {
            edges.push((
                PlanWallEdge::Vertical { x: x0, y0, y1 },
                PLAN_WALL_THICKNESS_M,
            ));
        }
        if cell.wall4 != Material::Open {
            edges.push((
                PlanWallEdge::Horizontal { y: y0, x0, x1 },
                PLAN_WALL_THICKNESS_M,
            ));
        }
    }
    edges
}

pub fn build_deck_wall_mesh(deck: DeckCells<'_>) -> Mesh {
    let edges = collect_wall_edges(deck);
    merged_plan_wall_borders_mesh_varied(&edges, Z_WALL_PLANE, WALL_BORDER_COLOR)
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
