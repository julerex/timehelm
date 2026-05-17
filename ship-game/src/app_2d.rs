//! Top-down 2D plan view of the ship (metres in XY; ship-space **+Y** = bow). The view rotates the hull
//! so bow runs **left–right** on screen.
//!
//! All 20 decks are baked to one vertex-coloured `Mesh2d` each at startup and share a single
//! white `ColorMaterial`; deck switching just toggles `Visibility` on the relevant entity.

use crate::cell::{AgentId, CELL_NEIGHBOUR_OFFSETS};
use crate::cell_box::{deck_walk_grid, step_allowed, CellIndex, PlanKey};
use crate::deck_layout::{DeckCells, DeckLayouts, CELL_SIZE_M, NUM_DECKS};
use crate::edit_mode_2d::{spawn_edit_mode_panel, PlanEditPlugin};
use crate::load_screen::{spawn_load_menu, GamePhase, LoadScreenPlugin};
use crate::plan_mesh::{build_deck_mesh, build_deck_wall_mesh, DeckPlanMeshes};
use crate::shared::{
    asset_plugin, cursor_in_game_viewport, deck_info_text_2d, format_cell_hover_line,
    game_camera_viewport, primary_window,
};
use crate::ship_hull::SHIP_LENGTH_M;
use crate::ship_save::empty_deck_layouts;
use bevy::camera::visibility::RenderLayers;
use bevy::camera::{OrthographicProjection, Projection, ScalingMode};
use bevy::input::mouse::{MouseScrollUnit, MouseWheel};
use bevy::prelude::*;
use std::collections::HashMap;
use std::f32::consts::FRAC_PI_2;

const SIM_DECK_INDEX: usize = 4;
const VIEW_WIDTH_M: f32 = SHIP_LENGTH_M * 1.12;

const PLAN_ZOOM_MIN: f32 = 0.5;
const PLAN_ZOOM_MAX: f32 = 4.0;
const PLAN_ZOOM_SCROLL_FACTOR: f32 = 1.1;

const STATUS_TOAST_SECS: f32 = 3.0;

/// Simulated walkers: slight Z offset above the deck cell mesh.
const Z_HUMAN: f32 = 0.012;

const NUM_SIM_HUMANS: usize = 10_000;
/// Light red human footprint (exactly 1 m × 1 m in plan space).
const HUMAN_CELL_COLOR: Color = Color::srgba(0.96, 0.62, 0.62, 0.95);
/// Grid steps per second (cardinal and diagonal).
const SIM_HUMAN_STEPS_PER_S: f32 = 2.8;

/// Layer for deck plan `Mesh2d` geometry; the UI `Camera2d` stays on the default layer.
pub(crate) fn plan_world_render_layers() -> RenderLayers {
    RenderLayers::layer(1)
}

#[derive(Component)]
pub(crate) struct ShipPlan2dRotateRoot;

#[derive(Component)]
pub(crate) struct GamePlanCamera2d;

#[derive(Component)]
struct UiCamera2d;

#[derive(Component)]
struct DeckInfoText2d;

#[derive(Component)]
struct HoverCellText2d;

#[derive(Component)]
struct StatusToastText;

/// Plan camera zoom factor (1.0 = default; larger = zoomed in).
#[derive(Resource)]
struct PlanViewZoom(f32);

impl Default for PlanViewZoom {
    fn default() -> Self {
        Self(1.0)
    }
}

#[derive(Resource, Default)]
struct StatusToast {
    remaining: f32,
}

#[derive(Resource)]
pub(crate) struct CurrentDeck(pub(crate) usize);

#[derive(Resource)]
pub(crate) struct DeckContentEntities(pub(crate) [Entity; NUM_DECKS]);

/// Per-deck map from 1 m grid cell to exact walk cell centre.
#[derive(Resource)]
pub(crate) struct DeckWalkGrids(pub(crate) Vec<HashMap<PlanKey, Vec2>>);

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

#[derive(Resource)]
pub(crate) struct Plan2dVisualAssets {
    pub(crate) rotate_root: Entity,
    shared_material: Handle<ColorMaterial>,
    human_mesh: Handle<Mesh>,
    human_material: Handle<ColorMaterial>,
}

#[derive(Resource)]
struct Plan2dWorldSpawned(bool);

pub fn run_app_2d() {
    let mut app = App::new();
    app.add_plugins(
        DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(primary_window()),
                ..default()
            })
            .set(asset_plugin())
            .set(ImagePlugin::default_nearest()),
    );
    #[cfg(not(target_arch = "wasm32"))]
    app.add_plugins(crate::ship_save::ShipSavePlugin);
    app.add_plugins((LoadScreenPlugin, PlanEditPlugin))
        .insert_resource(empty_deck_layouts())
        .insert_resource(CurrentDeck(SIM_DECK_INDEX))
        .insert_resource(SimRng::default())
        .insert_resource(NextAgentId(1))
        .insert_resource(Plan2dWorldSpawned(false))
        .insert_resource(PlanViewZoom::default())
        .insert_resource(StatusToast::default())
        .insert_resource(ClearColor(Color::srgb(0.04, 0.09, 0.16)))
        .add_systems(Startup, setup_2d)
        .add_systems(
            OnEnter(GamePhase::InGame),
            (enter_plan_world_2d, init_deck_info_text_2d).chain(),
        )
        .add_systems(
            Update,
            (
                deck_switch_2d,
                sync_plan_deck_visibility,
                human_wander_2d,
                (
                    sync_plan_camera_viewport,
                    plan_mouse_wheel_zoom,
                    apply_plan_view_zoom,
                    update_hover_cell_label_2d,
                )
                    .chain(),
                update_deck_info_text_2d,
                update_status_toast,
            )
                .run_if(in_state(GamePhase::InGame)),
        )
        .add_systems(
            Update,
            sync_plan_camera_viewport.run_if(in_state(GamePhase::LoadMenu)),
        );
    #[cfg(not(target_arch = "wasm32"))]
    app.add_systems(
        Update,
        (reload_plan_decks_after_load, on_ship_save_succeeded).run_if(in_state(GamePhase::InGame)),
    );
    app.run();
}

fn spawn_plan_deck_entities(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    layouts: &mut DeckLayouts,
    assets: &Plan2dVisualAssets,
    current_deck: usize,
    rng: &mut SimRng,
    next_agent: &mut NextAgentId,
) -> ([Entity; NUM_DECKS], DeckPlanMeshes) {
    let walk_grids = deck_walk_grids(layouts);
    let per_deck = humans_per_deck(&walk_grids, layouts, rng);
    let step_period = SIM_HUMAN_STEPS_PER_S.recip();

    let mut deck_entities = [Entity::PLACEHOLDER; NUM_DECKS];
    let mut floor_meshes: [Handle<Mesh>; NUM_DECKS] = std::array::from_fn(|_| Handle::default());
    let mut wall_meshes: [Handle<Mesh>; NUM_DECKS] = std::array::from_fn(|_| Handle::default());
    for (deck_i, slot) in deck_entities.iter_mut().enumerate() {
        let mesh_handle = meshes.add(build_deck_mesh(deck_i, layouts.deck(deck_i)));
        floor_meshes[deck_i] = mesh_handle.clone();
        let visibility = if deck_i == current_deck {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
        let wall_mesh_handle = meshes.add(build_deck_wall_mesh(layouts.deck(deck_i)));
        wall_meshes[deck_i] = wall_mesh_handle.clone();
        *slot = commands
            .spawn((
                Mesh2d(mesh_handle),
                MeshMaterial2d(assets.shared_material.clone()),
                Transform::IDENTITY,
                visibility,
                plan_world_render_layers(),
                ChildOf(assets.rotate_root),
            ))
            .with_children(|plan| {
                plan.spawn((
                    Mesh2d(wall_mesh_handle),
                    MeshMaterial2d(assets.shared_material.clone()),
                    Transform::IDENTITY,
                    plan_world_render_layers(),
                ));
                let deck_placements = &per_deck[deck_i];
                for &(pos_xy, tgt_xy) in deck_placements {
                    let agent_id = AgentId(next_agent.0);
                    next_agent.0 += 1;
                    let cell =
                        DeckCells::cell_coords_deck(pos_xy, deck_i as u8).expect("walk cell");
                    register_agent_on_cell(layouts, deck_i, cell, agent_id);
                    let stagger =
                        (rng.next_u32() as f32 / u32::MAX as f32).clamp(0.0, 1.0) * step_period;
                    plan.spawn((
                        Mesh2d(assets.human_mesh.clone()),
                        MeshMaterial2d(assets.human_material.clone()),
                        Transform::from_xyz(pos_xy.x, pos_xy.y, Z_HUMAN),
                        plan_world_render_layers(),
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
    (
        deck_entities,
        DeckPlanMeshes {
            floors: floor_meshes,
            walls: wall_meshes,
        },
    )
}

#[cfg(not(target_arch = "wasm32"))]
fn reload_plan_decks_after_load(
    mut events: MessageReader<crate::ship_save::ShipLayoutsReplaced>,
    mut commands: Commands,
    mut layouts: ResMut<DeckLayouts>,
    mut meshes: ResMut<Assets<Mesh>>,
    assets: Res<Plan2dVisualAssets>,
    current_deck: Res<CurrentDeck>,
    mut deck_entities: ResMut<DeckContentEntities>,
    mut walk_grids: ResMut<DeckWalkGrids>,
    mut rng: ResMut<SimRng>,
    mut next_agent: ResMut<NextAgentId>,
) {
    for _ in events.read() {
        for &entity in &deck_entities.0 {
            commands.entity(entity).despawn();
        }
        for (_, cell) in layouts.cells.iter_occupied_mut() {
            cell.contents = crate::cell::Bag::default();
        }
        next_agent.0 = 1;
        let (new_entities, new_meshes) = spawn_plan_deck_entities(
            &mut commands,
            &mut meshes,
            &mut layouts,
            &assets,
            current_deck.0,
            &mut rng,
            &mut next_agent,
        );
        *walk_grids = deck_walk_grids(&layouts);
        *deck_entities = DeckContentEntities(new_entities);
        commands.insert_resource(new_meshes);
    }
}

fn setup_2d(
    mut commands: Commands,
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
        plan_world_render_layers(),
        GamePlanCamera2d,
    ));

    let ui_camera = commands
        .spawn((
            Camera2d,
            IsDefaultUiCamera,
            Camera {
                order: 1,
                clear_color: ClearColorConfig::None,
                ..default()
            },
            UiCamera2d,
        ))
        .id();

    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(50.0),
                position_type: PositionType::Absolute,
                top: Val::Px(0.0),
                left: Val::Px(0.0),
                flex_direction: FlexDirection::Row,
                ..default()
            },
            UiTargetCamera(ui_camera),
        ))
        .with_children(|hud| {
            hud.spawn((
                Node {
                    flex_grow: 1.0,
                    height: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    padding: UiRect::all(Val::Px(12.0)),
                    row_gap: Val::Px(8.0),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.05, 0.08, 0.12, 0.88)),
            ))
            .with_children(|left| {
                left.spawn((
                    Text::new(""),
                    TextFont {
                        font_size: 20.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                    DeckInfoText2d,
                ));
                left.spawn((
                    Text::new("Hover: —"),
                    TextFont {
                        font_size: 18.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.75, 0.82, 0.9)),
                    HoverCellText2d,
                ));
                left.spawn((
                    Text::new(""),
                    TextFont {
                        font_size: 17.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.45, 0.92, 0.55)),
                    Visibility::Hidden,
                    StatusToastText,
                ));
            });
        });
    spawn_load_menu(&mut commands, ui_camera);
    spawn_edit_mode_panel(&mut commands, ui_camera);

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
    commands.insert_resource(Plan2dVisualAssets {
        rotate_root,
        shared_material,
        human_mesh,
        human_material,
    });
}

fn enter_plan_world_2d(
    mut commands: Commands,
    mut layouts: ResMut<DeckLayouts>,
    mut meshes: ResMut<Assets<Mesh>>,
    assets: Res<Plan2dVisualAssets>,
    mut rng: ResMut<SimRng>,
    mut next_agent: ResMut<NextAgentId>,
    current_deck: Res<CurrentDeck>,
    mut spawned: ResMut<Plan2dWorldSpawned>,
) {
    if spawned.0 {
        return;
    }
    let (deck_entities, deck_meshes) = spawn_plan_deck_entities(
        &mut commands,
        &mut meshes,
        &mut layouts,
        &assets,
        current_deck.0,
        &mut rng,
        &mut next_agent,
    );
    commands.insert_resource(deck_walk_grids(&layouts));
    commands.insert_resource(DeckContentEntities(deck_entities));
    commands.insert_resource(deck_meshes);
    spawned.0 = true;
}

fn sync_plan_camera_viewport(
    window: Single<&Window>,
    mut cameras: Query<&mut Camera, With<GamePlanCamera2d>>,
) {
    let viewport = game_camera_viewport(&window);
    for mut camera in &mut cameras {
        camera.viewport = Some(viewport.clone());
    }
}

fn hover_cell_line_2d(
    window: &Window,
    current_deck: usize,
    layouts: &DeckLayouts,
    cameras: &Query<(&Camera, &GlobalTransform), With<GamePlanCamera2d>>,
    rotate_roots: &Query<&GlobalTransform, With<ShipPlan2dRotateRoot>>,
) -> String {
    let Ok((camera, cam_tf)) = cameras.single() else {
        return "Hover: —".to_string();
    };
    let Some(cursor) = cursor_in_game_viewport(window, camera) else {
        return "Hover: —".to_string();
    };
    let Ok(world_xy) = camera.viewport_to_world_2d(cam_tf, cursor) else {
        return "Hover: —".to_string();
    };
    let Ok(plan_root_tf) = rotate_roots.single() else {
        return "Hover: —".to_string();
    };
    let hull_xy = plan_root_tf
        .affine()
        .inverse()
        .transform_point3(world_xy.extend(0.0))
        .truncate();
    format_cell_hover_line(hull_xy, current_deck, layouts)
}

fn update_hover_cell_label_2d(
    window: Single<&Window>,
    current_deck: Res<CurrentDeck>,
    layouts: Res<DeckLayouts>,
    cameras: Query<(&Camera, &GlobalTransform), With<GamePlanCamera2d>>,
    rotate_roots: Query<&GlobalTransform, With<ShipPlan2dRotateRoot>>,
    mut texts: Query<&mut Text, With<HoverCellText2d>>,
) {
    let hover_line = hover_cell_line_2d(&window, current_deck.0, &layouts, &cameras, &rotate_roots);
    for mut text in &mut texts {
        text.0 = hover_line.clone();
    }
}

fn update_deck_info_text_2d(
    current_deck: Res<CurrentDeck>,
    mut query: Query<&mut Text, With<DeckInfoText2d>>,
) {
    if !current_deck.is_changed() {
        return;
    }
    for mut text in &mut query {
        text.0 = deck_info_text_2d(current_deck.0);
    }
}

fn init_deck_info_text_2d(
    current_deck: Res<CurrentDeck>,
    mut query: Query<&mut Text, With<DeckInfoText2d>>,
) {
    for mut text in &mut query {
        text.0 = deck_info_text_2d(current_deck.0);
    }
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
    edit_mode: Res<crate::edit_mode_2d::PlanEditMode>,
    mut layouts: ResMut<DeckLayouts>,
    grids: Res<DeckWalkGrids>,
    mut rng: ResMut<SimRng>,
    mut humans: Query<(&mut SimHuman, &mut Transform, &mut HumanWander)>,
) {
    if edit_mode.active {
        return;
    }
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

        let plan =
            CellIndex::from_world_xy_deck(tf.translation.xy(), deck_idx as u8).map(CellIndex::plan);
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

        let target_cell =
            CellIndex::from_world_xy_deck(wander.target, deck_idx as u8).map(CellIndex::plan);
        if target_cell == Some(plan) {
            if let Some(t) = random_walk_point(grid, &mut rng) {
                wander.target = t;
            }
            continue;
        }

        let next_pos =
            pick_strictly_closer_neighbour(&layouts, deck_idx, grid, &mut rng, plan, wander.target);
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

fn plan_mouse_wheel_zoom(
    window: Single<&Window>,
    cameras: Query<(&Camera, &GlobalTransform), With<GamePlanCamera2d>>,
    mut zoom: ResMut<PlanViewZoom>,
    mut scroll: MessageReader<MouseWheel>,
) {
    let Ok((camera, _)) = cameras.single() else {
        return;
    };
    if cursor_in_game_viewport(&window, camera).is_none() {
        return;
    }
    for ev in scroll.read() {
        let dy = match ev.unit {
            MouseScrollUnit::Line => ev.y,
            MouseScrollUnit::Pixel => ev.y * 0.015,
        };
        if dy > 0.0 {
            zoom.0 = (zoom.0 * PLAN_ZOOM_SCROLL_FACTOR).clamp(PLAN_ZOOM_MIN, PLAN_ZOOM_MAX);
        } else if dy < 0.0 {
            zoom.0 = (zoom.0 / PLAN_ZOOM_SCROLL_FACTOR).clamp(PLAN_ZOOM_MIN, PLAN_ZOOM_MAX);
        }
    }
}

fn apply_plan_view_zoom(
    zoom: Res<PlanViewZoom>,
    mut projections: Query<&mut Projection, With<GamePlanCamera2d>>,
) {
    for mut projection in &mut projections {
        let Projection::Orthographic(ref mut ortho) = *projection else {
            continue;
        };
        ortho.scaling_mode = ScalingMode::FixedHorizontal {
            viewport_width: VIEW_WIDTH_M / zoom.0,
        };
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn on_ship_save_succeeded(
    mut events: MessageReader<crate::ship_save::ShipSaveSucceeded>,
    mut toast: ResMut<StatusToast>,
) {
    if events.read().next().is_some() {
        toast.remaining = STATUS_TOAST_SECS;
    }
}

fn update_status_toast(
    time: Res<Time>,
    mut toast: ResMut<StatusToast>,
    mut text_query: Query<(&mut Text, &mut Visibility), With<StatusToastText>>,
) {
    let mut show = false;
    if toast.remaining > 0.0 {
        toast.remaining -= time.delta_secs();
        show = true;
    }
    for (mut text, mut vis) in &mut text_query {
        if show {
            *vis = Visibility::Inherited;
            text.0 = "Saved successfully".to_string();
        } else {
            *vis = Visibility::Hidden;
        }
    }
}
