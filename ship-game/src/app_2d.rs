//! Top-down 2D plan view of the ship (metres in XY; ship-space **+Y** = bow). The view rotates the hull
//! so bow runs **left–right** on screen.
//!
//! All 20 decks are baked to one vertex-coloured `Mesh2d` each at startup and share a single
//! white `ColorMaterial`; deck switching just toggles `Visibility` on the relevant entity.

use crate::cell::{AgentId, Material, CELL_NEIGHBOUR_OFFSETS};
use crate::cell_box::{self, deck_walk_grid, step_allowed, CellIndex, PlanKey};
use crate::deck_geometry::{
    merged_plan_squares_mesh_colored, merged_plan_wall_borders_mesh, PlanWallEdge,
};
use crate::deck_layout::{
    deck_cell_layouts, DeckCells, DeckLayouts, CELL_SIZE_M, CELL_VISUAL_SCALE, NUM_DECKS,
};
use crate::shared::{asset_plugin, primary_window};
use crate::ship_hull::SHIP_LENGTH_M;
use bevy::camera::{OrthographicProjection, Projection, ScalingMode};
use bevy::prelude::*;
use std::collections::HashMap;
use std::f32::consts::FRAC_PI_2;

const SIM_DECK_INDEX: usize = 4;
const VIEW_WIDTH_M: f32 = SHIP_LENGTH_M * 1.12;

const Z_CELL_PLANE: f32 = 0.0;
/// Wall strokes sit slightly above floor quads to avoid z-fighting.
const Z_WALL_PLANE: f32 = 0.001;
/// Plan-view wall border thickness (m).
const WALL_BORDER_THICKNESS_M: f32 = 0.05;
const WALL_BORDER_COLOR: [f32; 4] = [0.0, 0.0, 0.0, 1.0];

/// Simulated walkers: slight Z offset above the deck cell mesh.
const Z_HUMAN: f32 = 0.012;

const NUM_SIM_HUMANS: usize = 10_000;
/// Light red human footprint (exactly 1 m × 1 m in plan space).
const HUMAN_CELL_COLOR: Color = Color::srgba(0.96, 0.62, 0.62, 0.95);
/// Grid steps per second (cardinal and diagonal).
const SIM_HUMAN_STEPS_PER_S: f32 = 2.8;

#[derive(Component)]
struct ShipPlan2dRotateRoot;

#[derive(Resource)]
struct CurrentDeck(usize);

#[derive(Resource)]
struct DeckContentEntities([Entity; NUM_DECKS]);

/// Per-deck map from 1 m grid cell to exact walk cell centre.
#[derive(Resource)]
struct DeckWalkGrids(Vec<HashMap<PlanKey, Vec2>>);

#[derive(Resource)]
struct SimRng {
    state: u64,
}

impl Default for SimRng {
    fn default() -> Self {
        Self {
            state: 0x2f50_794f_dcae_b5a3,
        }
    }
}

impl SimRng {
    fn next_u32(&mut self) -> u32 {
        self.state ^= self.state >> 12;
        self.state ^= self.state << 25;
        self.state ^= self.state >> 27;
        let mixed = self.state.wrapping_mul(0x2545_f491_4f6c_dd1d);
        (mixed >> 32) as u32
    }

    fn next_usize(&mut self, upper_exclusive: usize) -> usize {
        if upper_exclusive <= 1 {
            0
        } else {
            (self.next_u32() as usize) % upper_exclusive
        }
    }
}

#[derive(Resource)]
struct NextAgentId(u64);

#[derive(Component)]
struct SimHuman {
    agent_id: AgentId,
    deck_idx: usize,
    last_cell: Option<PlanKey>,
    steps_per_second: f32,
    time_until_step: f32,
}

#[derive(Component)]
struct HumanWander {
    target: Vec2,
}

fn color_to_linear_array(c: Color) -> [f32; 4] {
    let lr: LinearRgba = c.into();
    [lr.red, lr.green, lr.blue, lr.alpha]
}

/// Bake all of a deck's cells into one vertex-coloured mesh.
fn build_deck_mesh(deck_index: usize, deck: DeckCells<'_>) -> Mesh {
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

/// Collect axis-aligned wall edges once per shared boundary (5 cm strokes in plan view).
fn collect_wall_edges(deck: DeckCells<'_>) -> Vec<PlanWallEdge> {
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
            edges.push(PlanWallEdge::Vertical {
                x: x1,
                y0,
                y1,
            });
        }
        if cell.wall2 != Material::Open {
            edges.push(PlanWallEdge::Horizontal {
                y: y1,
                x0,
                x1,
            });
        }
        if cell.wall3 != Material::Open {
            let west_draws = deck
                .index(plan)
                .offset(-1, 0, 0)
                .and_then(|i| deck.get(i.plan()))
                .is_none_or(|w| w.wall1 == Material::Open);
            if west_draws {
                edges.push(PlanWallEdge::Vertical {
                    x: x0,
                    y0,
                    y1,
                });
            }
        }
        if cell.wall4 != Material::Open {
            let south_draws = deck
                .index(plan)
                .offset(0, -1, 0)
                .and_then(|i| deck.get(i.plan()))
                .is_none_or(|s| s.wall2 == Material::Open);
            if south_draws {
                edges.push(PlanWallEdge::Horizontal {
                    y: y0,
                    x0,
                    x1,
                });
            }
        }
    }
    edges
}

fn build_deck_wall_mesh(deck: DeckCells<'_>) -> Mesh {
    let edges = collect_wall_edges(deck);
    merged_plan_wall_borders_mesh(
        &edges,
        WALL_BORDER_THICKNESS_M,
        Z_WALL_PLANE,
        WALL_BORDER_COLOR,
    )
}

fn random_walk_point(grid: &HashMap<PlanKey, Vec2>, rng: &mut SimRng) -> Option<Vec2> {
    if grid.is_empty() {
        return None;
    }
    let skip = rng.next_usize(grid.len());
    grid.values().nth(skip).copied()
}

fn humans_per_deck(
    grids: &DeckWalkGrids,
    layouts: &DeckLayouts,
    rng: &mut SimRng,
) -> [Vec<(Vec2, Vec2)>; NUM_DECKS] {
    fn nonempty_deck(rng: &mut SimRng, layouts: &DeckLayouts) -> usize {
        for _ in 0..128 {
            let d = rng.next_usize(NUM_DECKS);
            if layouts.cells.deck_occupied(d as u8) > 0 {
                return d;
            }
        }
        (0..NUM_DECKS)
            .find(|i| layouts.cells.deck_occupied(*i as u8) > 0)
            .unwrap_or(0)
    }

    let mut per_deck: [Vec<(Vec2, Vec2)>; NUM_DECKS] = std::array::from_fn(|_| Vec::new());

    for _ in 0..NUM_SIM_HUMANS {
        let deck_idx = rng.next_usize(NUM_DECKS);
        let grid = &grids.0[deck_idx];
        let (deck_idx, grid) = if grid.is_empty() {
            let d = nonempty_deck(rng, layouts);
            (d, &grids.0[d])
        } else {
            (deck_idx, grid)
        };
        let Some(spawn) = random_walk_point(grid, rng) else {
            continue;
        };
        let Some(target) = random_walk_point(grid, rng) else {
            continue;
        };
        per_deck[deck_idx].push((spawn, target));
    }

    per_deck
}

fn deck_walk_grids(layouts: &DeckLayouts) -> DeckWalkGrids {
    DeckWalkGrids(
        (0..NUM_DECKS)
            .map(|i| deck_walk_grid(&layouts.cells, i as u8))
            .collect(),
    )
}

fn register_agent_on_cell(
    layouts: &mut DeckLayouts,
    deck_idx: usize,
    plan: PlanKey,
    agent: AgentId,
) {
    let index = CellIndex::with_plan(deck_idx as u8, plan).expect("plan in box");
    if let Some(cell) = layouts.cell_mut(index) {
        cell.contents.insert(agent);
    }
}

fn unregister_agent_from_cell(
    layouts: &mut DeckLayouts,
    deck_idx: usize,
    plan: PlanKey,
    agent: AgentId,
) {
    let index = CellIndex::with_plan(deck_idx as u8, plan).expect("plan in box");
    if let Some(cell) = layouts.cell_mut(index) {
        cell.contents.remove(agent);
    }
}

/// Chooses among neighbours (8-way with corner cutting) that strictly reduce distance-to-target.
fn pick_strictly_closer_neighbour(
    layouts: &DeckLayouts,
    deck_idx: usize,
    grid: &HashMap<PlanKey, Vec2>,
    rng: &mut SimRng,
    plan: PlanKey,
    target: Vec2,
) -> Option<Vec2> {
    let index = CellIndex::with_plan(deck_idx as u8, plan).expect("plan in box");
    let &current = grid.get(&plan)?;
    let d0 = current.distance_squared(target);

    let mut best_d = f32::INFINITY;
    let mut best: Vec<Vec2> = Vec::new();

    for (dx, dy) in CELL_NEIGHBOUR_OFFSETS {
        if !step_allowed(&layouts.cells, index, dx, dy) {
            continue;
        }
        let Some(nb) = index.offset(dx, dy, 0) else {
            continue;
        };
        let Some(&p) = grid.get(&nb.plan()) else {
            continue;
        };
        let d = p.distance_squared(target);
        if d + 1e-5 >= d0 {
            continue;
        }
        if d + 1e-5 < best_d {
            best_d = d;
            best.clear();
            best.push(p);
        } else if (d - best_d).abs() < 1e-5 {
            best.push(p);
        }
    }

    (!best.is_empty()).then(|| best[rng.next_usize(best.len())])
}

pub fn run_app_2d() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(primary_window()),
                    ..default()
                })
                .set(asset_plugin())
                .set(ImagePlugin::default_nearest()),
        )
        .insert_resource(CurrentDeck(SIM_DECK_INDEX))
        .insert_resource(deck_cell_layouts(crate::deck_layout::CELL_SIZE_M))
        .insert_resource(SimRng::default())
        .insert_resource(NextAgentId(1))
        .insert_resource(ClearColor(Color::srgb(0.04, 0.09, 0.16)))
        .add_systems(Startup, setup_2d)
        .add_systems(
            Update,
            (deck_switch_2d, sync_plan_deck_visibility, human_wander_2d),
        )
        .run();
}

fn setup_2d(
    mut commands: Commands,
    mut layouts: ResMut<DeckLayouts>,
    mut rng: ResMut<SimRng>,
    mut next_agent: ResMut<NextAgentId>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let mut world_ortho = OrthographicProjection::default_2d();
    world_ortho.scaling_mode = ScalingMode::FixedHorizontal {
        viewport_width: VIEW_WIDTH_M,
    };
    commands.spawn((
        Camera2d,
        Camera {
            order: 0,
            ..default()
        },
        Projection::from(world_ortho),
    ));

    let rotate_root = commands
        .spawn((
            ShipPlan2dRotateRoot,
            Transform::from_rotation(Quat::from_rotation_z(-FRAC_PI_2)),
            Visibility::Inherited,
            GlobalTransform::default(),
        ))
        .id();

    let shared_material = materials.add(ColorMaterial::from(Color::WHITE));
    let human_mesh = meshes.add(Mesh::from(Rectangle::new(CELL_SIZE_M, CELL_SIZE_M)));
    let human_material = materials.add(ColorMaterial::from(HUMAN_CELL_COLOR));
    let walk_grids = deck_walk_grids(&layouts);
    let per_deck = humans_per_deck(&walk_grids, &layouts, &mut rng);
    commands.insert_resource(walk_grids);
    let step_period = SIM_HUMAN_STEPS_PER_S.recip();

    let mut deck_entities = [Entity::PLACEHOLDER; NUM_DECKS];
    for (deck_i, slot) in deck_entities.iter_mut().enumerate() {
        let mesh_handle = meshes.add(build_deck_mesh(deck_i, layouts.deck(deck_i)));
        let visibility = if deck_i == SIM_DECK_INDEX {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
        let wall_mesh_handle = meshes.add(build_deck_wall_mesh(layouts.deck(deck_i)));
        *slot = commands
            .spawn((
                Mesh2d(mesh_handle),
                MeshMaterial2d(shared_material.clone()),
                Transform::IDENTITY,
                visibility,
                ChildOf(rotate_root),
            ))
            .with_children(|plan| {
                plan.spawn((
                    Mesh2d(wall_mesh_handle),
                    MeshMaterial2d(shared_material.clone()),
                    Transform::IDENTITY,
                ));
                let deck_placements = &per_deck[deck_i];
                for &(pos_xy, tgt_xy) in deck_placements {
                    let agent_id = AgentId(next_agent.0);
                    next_agent.0 += 1;
                    let cell = DeckCells::cell_coords_deck(pos_xy, deck_i as u8).expect("walk cell");
                    register_agent_on_cell(&mut layouts, deck_i, cell, agent_id);
                    let stagger =
                        (rng.next_u32() as f32 / u32::MAX as f32).clamp(0.0, 1.0) * step_period;
                    plan.spawn((
                        Mesh2d(human_mesh.clone()),
                        MeshMaterial2d(human_material.clone()),
                        Transform::from_xyz(pos_xy.x, pos_xy.y, Z_HUMAN),
                        SimHuman {
                            agent_id,
                            deck_idx: deck_i,
                            last_cell: Some(cell),
                            steps_per_second: SIM_HUMAN_STEPS_PER_S,
                            time_until_step: stagger,
                        },
                        HumanWander { target: tgt_xy },
                    ));
                }
            })
            .id();
    }
    commands.insert_resource(DeckContentEntities(deck_entities));
}

fn deck_switch_2d(keyboard: Res<ButtonInput<KeyCode>>, mut current_deck: ResMut<CurrentDeck>) {
    let deck_up =
        keyboard.just_pressed(KeyCode::PageUp) || keyboard.just_pressed(KeyCode::BracketRight);
    let deck_down =
        keyboard.just_pressed(KeyCode::PageDown) || keyboard.just_pressed(KeyCode::BracketLeft);

    if deck_up && current_deck.0 < NUM_DECKS - 1 {
        current_deck.0 += 1;
    }
    if deck_down && current_deck.0 > 0 {
        current_deck.0 -= 1;
    }
}

fn sync_plan_deck_visibility(
    current: Res<CurrentDeck>,
    entities: Res<DeckContentEntities>,
    mut visibility: Query<&mut Visibility>,
    mut last: Local<Option<usize>>,
) {
    if Some(current.0) == *last {
        return;
    }
    if let Some(prev) = *last {
        if let Ok(mut vis) = visibility.get_mut(entities.0[prev]) {
            *vis = Visibility::Hidden;
        }
    }
    if let Ok(mut vis) = visibility.get_mut(entities.0[current.0]) {
        *vis = Visibility::Inherited;
    }
    *last = Some(current.0);
}

fn human_wander_2d(
    time: Res<Time>,
    mut layouts: ResMut<DeckLayouts>,
    grids: Res<DeckWalkGrids>,
    mut rng: ResMut<SimRng>,
    mut humans: Query<(&mut SimHuman, &mut Transform, &mut HumanWander)>,
) {
    let dt = time.delta_secs();

    for (mut human, mut tf, mut wander) in &mut humans {
        let deck_idx = human.deck_idx;
        let Some(grid) = grids.0.get(deck_idx) else {
            continue;
        };
        if grid.is_empty() {
            continue;
        }

        human.time_until_step -= dt;
        if human.time_until_step > 0.0 {
            continue;
        }

        human.time_until_step += human.steps_per_second.recip().max(1e-4);

        let plan = CellIndex::from_world_xy_deck(tf.translation.xy(), deck_idx as u8)
            .map(CellIndex::plan);
        let Some(plan) = plan else {
            let Some(p) = random_walk_point(grid, &mut rng) else {
                continue;
            };
            if let Some(prev) = human.last_cell {
                unregister_agent_from_cell(&mut layouts, deck_idx, prev, human.agent_id);
            }
            let cell = CellIndex::from_world_xy_deck(p, deck_idx as u8)
                .expect("walk cell")
                .plan();
            register_agent_on_cell(&mut layouts, deck_idx, cell, human.agent_id);
            human.last_cell = Some(cell);
            tf.translation.x = p.x;
            tf.translation.y = p.y;
            if let Some(t) = random_walk_point(grid, &mut rng) {
                wander.target = t;
            }
            continue;
        };
        let Some(pos_snapped) = grid.get(&plan).copied() else {
            let Some(p) = random_walk_point(grid, &mut rng) else {
                continue;
            };
            if let Some(prev) = human.last_cell {
                unregister_agent_from_cell(&mut layouts, deck_idx, prev, human.agent_id);
            }
            let cell = CellIndex::from_world_xy_deck(p, deck_idx as u8)
                .expect("walk cell")
                .plan();
            register_agent_on_cell(&mut layouts, deck_idx, cell, human.agent_id);
            human.last_cell = Some(cell);
            tf.translation.x = p.x;
            tf.translation.y = p.y;
            if let Some(t) = random_walk_point(grid, &mut rng) {
                wander.target = t;
            }
            continue;
        };
        tf.translation.x = pos_snapped.x;
        tf.translation.y = pos_snapped.y;

        if human.last_cell != Some(plan) {
            if let Some(prev) = human.last_cell {
                unregister_agent_from_cell(&mut layouts, deck_idx, prev, human.agent_id);
            }
            register_agent_on_cell(&mut layouts, deck_idx, plan, human.agent_id);
            human.last_cell = Some(plan);
        }

        let target_cell = CellIndex::from_world_xy_deck(wander.target, deck_idx as u8)
            .map(CellIndex::plan);
        if target_cell == Some(plan) {
            if let Some(t) = random_walk_point(grid, &mut rng) {
                wander.target = t;
            }
            continue;
        }

        let next_pos = pick_strictly_closer_neighbour(
            &layouts,
            deck_idx,
            grid,
            &mut rng,
            plan,
            wander.target,
        );
        if let Some(next_pos) = next_pos {
            let next_cell = CellIndex::from_world_xy_deck(next_pos, deck_idx as u8)
                .expect("walk cell")
                .plan();
            if human.last_cell != Some(next_cell) {
                if let Some(prev) = human.last_cell {
                    unregister_agent_from_cell(&mut layouts, deck_idx, prev, human.agent_id);
                }
                register_agent_on_cell(&mut layouts, deck_idx, next_cell, human.agent_id);
                human.last_cell = Some(next_cell);
            }
            tf.translation.x = next_pos.x;
            tf.translation.y = next_pos.y;
        } else if let Some(t) = random_walk_point(grid, &mut rng) {
            wander.target = t;
        }
    }
}
