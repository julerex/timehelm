//! Top-down 2D plan view of the ship (metres in XY; ship-space **+Y** = bow). The view rotates the hull
//! so bow runs **left–right** on screen.
//!
//! All 20 decks are baked to one vertex-coloured `Mesh2d` each at startup and share a single
//! white `ColorMaterial`; deck switching just toggles `Visibility` on the relevant entity.

use crate::deck_geometry::merged_plan_squares_mesh_colored;
use crate::deck_layout::{
    amenity_overlay, deck_layouts, deck_seven_paint_tone, AmenityKind, DeckLayouts, DeckTileBucket,
    DeckTiles, NUM_DECKS, TILE_CELL_M, TILE_VISUAL_SCALE,
};
use crate::shared::{asset_plugin, primary_window};
use crate::ship_hull::SHIP_LENGTH_M;
use bevy::camera::{OrthographicProjection, Projection, ScalingMode};
use bevy::prelude::*;
use std::collections::HashMap;
use std::f32::consts::FRAC_PI_2;

const SIM_DECK_INDEX: usize = 4;
const VIEW_WIDTH_M: f32 = SHIP_LENGTH_M * 1.12;

const Z_TILE_PLANE: f32 = 0.0;

/// Simulated walkers: slight Z offset above the deck tile mesh.
const Z_HUMAN: f32 = 0.012;

const NUM_SIM_HUMANS: usize = 10_000;
/// Light red human footprint (exactly 1 m × 1 m in plan space).
const HUMAN_TILE_COLOR: Color = Color::srgba(0.96, 0.62, 0.62, 0.95);
/// Orthogonal tile steps per second (discrete motion; interpreted as cadence).
const SIM_HUMAN_STEPS_PER_S: f32 = 2.8;

/// Source-of-truth zone colours for the procedural deck mesh tinting (`build_deck_mesh`).
const COLOR_OUTER_CABIN: Color = Color::srgb(0.95, 0.82, 0.35);
const COLOR_WINDOW: Color = Color::srgb(0.42, 0.62, 0.9);
const COLOR_PUBLIC_DECK: Color = Color::srgb(0.78, 0.86, 0.92);

#[derive(Component)]
struct ShipPlan2dRotateRoot;

#[derive(Resource)]
struct CurrentDeck(usize);

/// Pre-built `Mesh2d` content entity for each deck, parented under the rotate root.
/// Switching decks just toggles `Visibility` on these.
#[derive(Resource)]
struct DeckContentEntities([Entity; NUM_DECKS]);

/// Per-deck map from 1 m grid cell to exact walk tile centre (`DeckTiles::centers`).
#[derive(Resource)]
struct DeckWalkGrids(Vec<HashMap<(i32, i32), Vec2>>);

/// Deterministic PRNG for human spawn paths and wandering (xoroshiroi-style mixing).
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

#[derive(Component)]
struct SimHuman {
    deck_idx: usize,
    steps_per_second: f32,
    /// Counts down until the next orthogonal hop on this agent’s cadence (`steps_per_second`).
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

/// Per-deck base tint (slight hue walk so adjacent decks read as distinct).
fn deck_base_tint(deck_index: usize) -> Color {
    let hue = 0.52 + (deck_index as f32 * 0.012);
    Color::hsla((hue * 360.0) % 360.0, 0.28, 0.42, 1.0)
}

/// Bake all of a deck's tiles into one vertex-coloured mesh.
fn build_deck_mesh(deck_index: usize, layout: &DeckTiles) -> Mesh {
    let half_tile = TILE_CELL_M * TILE_VISUAL_SCALE * 0.5;

    let outer_cabin = color_to_linear_array(COLOR_OUTER_CABIN);
    let bucket_colour: [[f32; 4]; DeckTileBucket::COUNT] = [
        outer_cabin,
        color_to_linear_array(COLOR_WINDOW),
        outer_cabin,
        outer_cabin,
        color_to_linear_array(COLOR_PUBLIC_DECK),
        color_to_linear_array(deck_base_tint(deck_index)),
    ];
    let amenity_colour: [[f32; 4]; AmenityKind::COUNT] =
        std::array::from_fn(|i| color_to_linear_array(AmenityKind::ALL[i].color()));

    let mut tile_colors = Vec::with_capacity(layout.centers.len());
    for c in &layout.centers {
        let cell = (
            (c.x / TILE_CELL_M).round() as i32,
            (c.y / TILE_CELL_M).round() as i32,
        );
        let color = if let Some(tone) = deck_seven_paint_tone(deck_index, *c, cell, layout) {
            color_to_linear_array(tone.color())
        } else if let Some(amenity) = amenity_overlay(deck_index, *c) {
            amenity_colour[amenity.idx()]
        } else {
            // Plan view: outer brown perimeter + pink/yellow cabin zoning reads like two nested hulls.
            // Fold hull-edge and inner blocks into the outboard cabin colour so the outline is one mass (3D keeps zones).
            let mut b = DeckTileBucket::classify(*c, layout.perimeter.contains(&cell));
            if matches!(b, DeckTileBucket::HullEdge | DeckTileBucket::InnerCabin) {
                b = DeckTileBucket::OuterCabin;
            }
            bucket_colour[b.idx()]
        };
        tile_colors.push(color);
    }

    merged_plan_squares_mesh_colored(&layout.centers, &tile_colors, half_tile, Z_TILE_PLANE)
}

/// Spawn assignments: `(initial_xy, wander_target_xy)` per human, partitioned by deck.
fn humans_per_deck(layouts: &DeckLayouts, rng: &mut SimRng) -> [Vec<(Vec2, Vec2)>; NUM_DECKS] {
    fn nonempty_deck(rng: &mut SimRng, layouts: &DeckLayouts) -> usize {
        for _ in 0..128 {
            let d = rng.next_usize(NUM_DECKS);
            if !layouts.0[d].centers.is_empty() {
                return d;
            }
        }
        (0..NUM_DECKS)
            .find(|i| !layouts.0[*i].centers.is_empty())
            .unwrap_or(0)
    }

    let mut per_deck: [Vec<(Vec2, Vec2)>; NUM_DECKS] = std::array::from_fn(|_| Vec::new());

    for _ in 0..NUM_SIM_HUMANS {
        let deck_idx = rng.next_usize(NUM_DECKS);
        let centers = &layouts.0[deck_idx].centers;
        let (deck_idx, centers) = if centers.is_empty() {
            let d = nonempty_deck(rng, layouts);
            (d, &layouts.0[d].centers)
        } else {
            (deck_idx, centers)
        };
        if centers.is_empty() {
            continue;
        }
        let spawn_i = rng.next_usize(centers.len());
        let tgt_i = rng.next_usize(centers.len());
        per_deck[deck_idx].push((centers[spawn_i], centers[tgt_i]));
    }

    per_deck
}

#[inline]
fn deck_tile_cell(p: Vec2) -> (i32, i32) {
    (
        (p.x / TILE_CELL_M).round() as i32,
        (p.y / TILE_CELL_M).round() as i32,
    )
}

fn deck_walk_grids(layouts: &DeckLayouts) -> DeckWalkGrids {
    DeckWalkGrids(
        layouts
            .0
            .iter()
            .map(|deck| {
                let mut m = HashMap::new();
                for &p in &deck.centers {
                    m.insert(deck_tile_cell(p), p);
                }
                m
            })
            .collect(),
    )
}

const CELL_NEIGHBOURS: [(i32, i32); 4] = [(1, 0), (-1, 0), (0, 1), (0, -1)];

/// Chooses among orthogonal neighbours that strictly reduce distance-to-target (`None` dead end).
fn pick_strictly_closer_neighbour(
    grid: &HashMap<(i32, i32), Vec2>,
    rng: &mut SimRng,
    ix: i32,
    iy: i32,
    target: Vec2,
) -> Option<Vec2> {
    let &current = grid.get(&(ix, iy))?;
    let d0 = current.distance_squared(target);

    let mut best_d = f32::INFINITY;
    let mut best: Vec<Vec2> = Vec::new();

    for (dx, dy) in CELL_NEIGHBOURS {
        let Some(&p) = grid.get(&(ix + dx, iy + dy)) else {
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
        .insert_resource(DeckLayouts(deck_layouts(TILE_CELL_M)))
        .insert_resource(SimRng::default())
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
    layouts: Res<DeckLayouts>,
    mut rng: ResMut<SimRng>,
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

    // One shared white material for every deck — vertex colours do all the work, so we never need
    // to allocate a `ColorMaterial` per bucket / per deck switch.
    let shared_material = materials.add(ColorMaterial::from(Color::WHITE));
    let human_mesh = meshes.add(Mesh::from(Rectangle::new(TILE_CELL_M, TILE_CELL_M)));
    let human_material = materials.add(ColorMaterial::from(HUMAN_TILE_COLOR));
    let per_deck = humans_per_deck(&layouts, &mut rng);
    commands.insert_resource(deck_walk_grids(&layouts));
    let step_period = SIM_HUMAN_STEPS_PER_S.recip();

    let mut deck_entities = [Entity::PLACEHOLDER; NUM_DECKS];
    for (deck_i, slot) in deck_entities.iter_mut().enumerate() {
        let mesh_handle = meshes.add(build_deck_mesh(deck_i, &layouts.0[deck_i]));
        let visibility = if deck_i == SIM_DECK_INDEX {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
        *slot = commands
            .spawn((
                Mesh2d(mesh_handle),
                MeshMaterial2d(shared_material.clone()),
                Transform::IDENTITY,
                visibility,
                ChildOf(rotate_root),
            ))
            .with_children(|plan| {
                let deck_placements = &per_deck[deck_i];
                for &(pos_xy, tgt_xy) in deck_placements {
                    let stagger =
                        (rng.next_u32() as f32 / u32::MAX as f32).clamp(0.0, 1.0) * step_period;
                    plan.spawn((
                        Mesh2d(human_mesh.clone()),
                        MeshMaterial2d(human_material.clone()),
                        Transform::from_xyz(pos_xy.x, pos_xy.y, Z_HUMAN),
                        SimHuman {
                            deck_idx: deck_i,
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

/// Hide the previously-visible deck entity and show the new one. All deck meshes/materials are
/// already on the GPU from startup, so deck switching is essentially free.
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
    layouts: Res<DeckLayouts>,
    grids: Res<DeckWalkGrids>,
    mut rng: ResMut<SimRng>,
    mut humans: Query<(&mut SimHuman, &mut Transform, &mut HumanWander)>,
) {
    let dt = time.delta_secs();

    for (mut human, mut tf, mut wander) in &mut humans {
        let centers = &layouts.0[human.deck_idx].centers;
        let Some(grid) = grids.0.get(human.deck_idx) else {
            continue;
        };
        if centers.is_empty() || grid.is_empty() {
            continue;
        }

        human.time_until_step -= dt;
        if human.time_until_step > 0.0 {
            continue;
        }

        human.time_until_step += human.steps_per_second.recip().max(1e-4);

        let (ix, iy) = deck_tile_cell(tf.translation.xy());
        let Some(pos_snapped) = grid.get(&(ix, iy)).copied() else {
            let p = centers[rng.next_usize(centers.len())];
            tf.translation.x = p.x;
            tf.translation.y = p.y;
            wander.target = centers[rng.next_usize(centers.len())];
            continue;
        };
        tf.translation.x = pos_snapped.x;
        tf.translation.y = pos_snapped.y;

        let target_cell = deck_tile_cell(wander.target);
        if (ix, iy) == target_cell {
            wander.target = centers[rng.next_usize(centers.len())];
            continue;
        }

        if let Some(next_pos) =
            pick_strictly_closer_neighbour(grid, &mut rng, ix, iy, wander.target)
        {
            tf.translation.x = next_pos.x;
            tf.translation.y = next_pos.y;
        } else {
            wander.target = centers[rng.next_usize(centers.len())];
        }
    }
}
