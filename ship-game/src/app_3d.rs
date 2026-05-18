//! Ship Game — 3D cruise-ship decks with horizontal cut plane (Bevy, WASM).
//!
//! **World space uses SI metres:** one unit of `Vec3`, mesh positions, and camera distance is **1 m**.
//!
//! Deck footprint uses [`crate::ship_hull::SHIP_LENGTH_M`] × [`crate::ship_hull::SHIP_BEAM_M`] at full scale.
//! Tiles are axis-aligned on **XY** (**+X** aft→fore, **+Y** starboard→port; origin aft-starboard); [`crate::deck_layout::deck_sim_footprint_polygon`]
//! matches coarse LOD to the simulated silhouette per deck. Decks stack on **+Z**; the clip shader removes fragments above the cut height.
//!
//! **Deck rendering:** distance-based **LOD** swaps coarse extruded hull + procedural deck texture vs
//! fine cuboid tiles; smaller buckets use **automatic GPU instancing** (shared mesh/material per tile),
//! larger buckets stay **CPU-merged**. [`crate::deck_geometry`] holds mesh helpers.

#![allow(clippy::too_many_arguments, clippy::type_complexity)]

use crate::cell::FloorMaterial;
use crate::cell_box;
use crate::deck_layout::{deck_sim_footprint_polygon, DeckLayouts, CELL_VISUAL_SCALE, NUM_DECKS};
use crate::load_screen::{spawn_load_menu, GamePhase, LoadScreenPlugin};
use crate::shader_embed::ShipShaderEmbedPlugin;
use crate::shared::{
    asset_plugin, cursor_in_game_viewport, deck_info_text_3d, format_cell_hover_line,
    game_camera_viewport, primary_window,
};
use crate::ship_hull::{SHIP_BEAM_M, SHIP_LENGTH_M};
use crate::ship_save::empty_deck_layouts;
use bevy::camera::primitives::Aabb;
use bevy::ecs::system::ParamSet;
use bevy::input::mouse::{MouseMotion, MouseScrollUnit, MouseWheel};
use bevy::pbr::{Material, MaterialPlugin, MeshMaterial3d};
use bevy::prelude::*;
use bevy::render::render_resource::AsBindGroup;
use bevy::shader::ShaderRef;
use std::collections::HashSet;

const SIM_DECK_INDEX: usize = 4; // Deck 5 (human-facing numbering)

const SIM_NPC_SCALE: f32 = 0.6;
const SIM_NPC_SPEED_M_S: f32 = 2.8;
const SIM_TARGET_REACHED_M: f32 = 0.45;
const SIM_NPC_MODEL_PATHS: [&str; 18] = [
    "character-a.glb",
    "character-b.glb",
    "character-c.glb",
    "character-d.glb",
    "character-e.glb",
    "character-f.glb",
    "character-g.glb",
    "character-h.glb",
    "character-i.glb",
    "character-j.glb",
    "character-k.glb",
    "character-l.glb",
    "character-m.glb",
    "character-n.glb",
    "character-o.glb",
    "character-p.glb",
    "character-q.glb",
    "character-r.glb",
];

/// Vertical spacing between deck floors (m along world +Z): **one deck level every 3 m**.
const DECK_FLOOR_SPACING_M: f32 = 3.0;
/// Extruded slab thickness (m); slightly under spacing so slabs do not z-fight deck-to-deck.
const DECK_SLAB_THICKNESS_M: f32 = 2.88;

/// Pan speed (m/s) for WASD and mouse middle-drag.
const CAMERA_PAN_SPEED_M_S: f32 = 520.0;
/// Vertical pan speed (m/s) for R/F.
const CAMERA_VERTICAL_SPEED_M_S: f32 = 260.0;
/// Keyboard orbit speed around the focal point (rad/s) on Q/E.
const CAMERA_YAW_SPEED_RAD_S: f32 = 1.75;
/// Dolly speed for Z/X (m/s).
const CAMERA_ZOOM_SPEED_M_S: f32 = 520.0;
/// Scroll wheel: each step scales distance by `(1 - dy * factor)` (dimensionless).
const CAMERA_SCROLL_ZOOM_FACTOR: f32 = 0.12;
const CAMERA_MOUSE_ORBIT_SENS: f32 = 0.005;
const CAMERA_MOUSE_PAN_SENS: f32 = 0.0022;
/// Default camera distance from focal point (m).
const CAM_DEFAULT_DISTANCE_M: f32 = 1180.0;
const CAM_MIN_DISTANCE_M: f32 = 90.0;
const CAM_MAX_DISTANCE_M: f32 = 6200.0;
/// Alpha multiplier for deck slab fragments **above** the cut plane (`clip_data.x` in WGSL).
/// `0.0` = fully transparent upper decks; `1.0` would leave them unchanged below the cut.
const ABOVE_DECK_ALPHA: f32 = 0.0;
/// Orbit pitch limits (radians from horizontal); keep camera above the XY plane.
const CAM_PITCH_MIN: f32 = 0.15;
const CAM_PITCH_MAX: f32 = 1.42;
/// Outside this camera-to-deck distance (m) we prefer the coarse textured extruded hull LOD.
const LOD_COARSE_BEYOND_M: f32 = 820.0;
/// Inside this distance (m) we prefer fine cuboid tiles (merged or batched instances).
const LOD_FINE_WITHIN_M: f32 = 680.0;
/// Cell counts at or below this use per-cell entities (Bevy automatic GPU instancing / batching).
const DECK_CELL_AUTOMATIC_INSTANCE_CAP: usize = 1600;
const CLIP_SHADER_FORWARD: &str = concat!(
    "embedded://",
    env!("CARGO_CRATE_NAME"),
    "/shaders/ship_clip_forward.wgsl"
);
const CLIP_SHADER_PREPASS: &str = concat!(
    "embedded://",
    env!("CARGO_CRATE_NAME"),
    "/shaders/ship_clip_prepass.wgsl"
);

#[derive(Component)]
struct GameCamera3d;

#[derive(Component)]
struct UiCamera;

#[derive(Component)]
struct HudRoot;

#[derive(Component)]
struct DeckInfoText;

#[derive(Component)]
struct HoverCellText;

#[derive(Component)]
struct DeckLayer(#[allow(dead_code)] usize);

#[derive(Component)]
struct DeckLodFineTier(usize);

#[derive(Component)]
struct DeckLodCoarseTier(usize);

#[derive(Resource, Default)]
struct DeckLodFinePreferred([bool; NUM_DECKS]);

#[derive(Component)]
struct SimNpc {
    speed_m_s: f32,
}

#[derive(Component)]
struct WanderState {
    target: Vec3,
}

#[derive(Resource)]
struct CurrentDeck(usize);

#[derive(Resource)]
struct DeckFiveWalkPoints(Vec<Vec3>);

#[derive(Resource)]
struct SimRng {
    state: u64,
}

#[derive(Resource, Default)]
struct NpcHeightLogState {
    logged_roots: HashSet<Entity>,
}

#[derive(Resource)]
struct GameWorldSpawned(bool);

/// Orbit camera: eye looks at `target` (m), offset given by yaw/pitch and `distance` (m).
#[derive(Resource)]
struct CameraRig {
    target: Vec3,
    yaw: f32,
    pitch: f32,
    distance: f32,
}

#[derive(Resource, Clone)]
struct SharedClipMaterial(Handle<ShipClipMaterial>);

#[derive(Asset, TypePath, AsBindGroup, Clone)]
struct ShipClipMaterial {
    /// `.x` = world-space Z cut height, `.y` = alpha for fragments above cut.
    #[uniform(0)]
    clip_data: Vec4,
    /// Multiplies albedo in world XY (coarse hull + fine tiles).
    #[texture(1)]
    #[sampler(2)]
    deck_pattern: Handle<Image>,
}

impl Material for ShipClipMaterial {
    fn fragment_shader() -> ShaderRef {
        CLIP_SHADER_FORWARD.into()
    }

    fn prepass_fragment_shader() -> ShaderRef {
        CLIP_SHADER_PREPASS.into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Blend
    }
}

pub fn run_app_3d() {
    let mut app = App::new();
    app.add_plugins(
        DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(primary_window()),
                ..default()
            })
            .set(asset_plugin())
            .set(ImagePlugin::default_nearest()),
    )
    .add_plugins(ShipShaderEmbedPlugin)
    .add_plugins(MaterialPlugin::<ShipClipMaterial>::default());
    #[cfg(not(target_arch = "wasm32"))]
    app.add_plugins(crate::ship_save::ShipSavePlugin);
    app.add_plugins(LoadScreenPlugin)
        .insert_resource(empty_deck_layouts())
        .insert_resource(CurrentDeck(SIM_DECK_INDEX))
        .insert_resource(CameraRig::default())
        .insert_resource(SimRng::default())
        .insert_resource(NpcHeightLogState::default())
        .insert_resource(ClearColor::default())
        .insert_resource(DeckLodFinePreferred::default())
        .insert_resource(GameWorldSpawned(false))
        .insert_resource(DeckFiveWalkPoints(Vec::new()))
        .add_systems(Startup, setup)
        .add_systems(
            OnEnter(GamePhase::InGame),
            (enter_game_world, init_deck_info_text).chain(),
        )
        .add_systems(
            Update,
            (
                deck_switch,
                camera_controls,
                update_deck_lod,
                sim_npc_wander,
                sync_clip_material,
                cull_npcs_above_cut,
                (sync_game_camera_viewport, update_hover_cell_label).chain(),
                update_deck_label,
                log_npc_heights_once,
            )
                .run_if(in_state(GamePhase::InGame)),
        )
        .add_systems(
            Update,
            (camera_controls, sync_game_camera_viewport).run_if(in_state(GamePhase::LoadMenu)),
        );
    #[cfg(not(target_arch = "wasm32"))]
    app.add_systems(
        Update,
        reload_deck_meshes_after_load.run_if(in_state(GamePhase::InGame)),
    );
    app.run();
}

impl Default for CameraRig {
    fn default() -> Self {
        let target = Vec3::new(
            SHIP_LENGTH_M * 0.5,
            SHIP_BEAM_M * 0.5,
            focused_deck_target_z(SIM_DECK_INDEX),
        );
        let dir0 = Vec3::new(0.82, -1.02, 0.68).normalize();
        let yaw = dir0.y.atan2(dir0.x);
        let pitch = dir0.z.clamp(-1.0, 1.0).asin();
        Self {
            target,
            yaw,
            pitch,
            distance: CAM_DEFAULT_DISTANCE_M,
        }
    }
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
        // xorshift64*
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

fn cut_plane_world_z(current_deck: usize) -> f32 {
    (current_deck + 1) as f32 * DECK_FLOOR_SPACING_M
}

/// Camera focus height for a given deck index: middle of the deck slab (m).
fn focused_deck_target_z(deck_index: usize) -> f32 {
    (deck_index as f32 + 0.5) * DECK_FLOOR_SPACING_M
}

/// World Z of the top face of a deck slab (m).
fn deck_top_z(deck_index: usize) -> f32 {
    deck_index as f32 * DECK_FLOOR_SPACING_M + DECK_SLAB_THICKNESS_M
}

fn ray_hit_plane_z(ray: Ray3d, plane_z: f32) -> Option<Vec3> {
    let dir = ray.direction.as_vec3();
    if dir.z.abs() < 1e-6 {
        return None;
    }
    let t = (plane_z - ray.origin.z) / dir.z;
    if t < 0.0 {
        return None;
    }
    Some(ray.origin + dir * t)
}

fn dir_from_yaw_pitch(yaw: f32, pitch: f32) -> Vec3 {
    let cp = pitch.cos();
    Vec3::new(cp * yaw.cos(), cp * yaw.sin(), pitch.sin())
}

fn camera_rig_transform(rig: &CameraRig) -> Transform {
    let dir = dir_from_yaw_pitch(rig.yaw, rig.pitch);
    let cam_pos = rig.target + dir * rig.distance;
    Transform::from_translation(cam_pos).looking_at(rig.target, Vec3::Z)
}

/// Axis-aligned deck slab (XY footprint, +Z up), vertex colours for the clip shader.
fn floor_from_idx(i: usize) -> FloorMaterial {
    FloorMaterial::ALL[i % FloorMaterial::COUNT]
}

fn deck_cell_cuboid_mesh(thickness_m: f32, color: Color) -> Mesh {
    let half_beam = cell_box::beam_cell_m() * CELL_VISUAL_SCALE * 0.5;
    let half_length = cell_box::length_cell_m() * CELL_VISUAL_SCALE * 0.5;
    let mut mesh = Mesh::from(Cuboid::new(half_length * 2.0, half_beam * 2.0, thickness_m));
    let n = mesh.count_vertices();
    let c: LinearRgba = color.into();
    let ca = c.to_f32_array();
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, vec![ca; n]);
    mesh
}

fn spawn_deck_meshes(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    clip_handle: &Handle<ShipClipMaterial>,
    layouts: &DeckLayouts,
) {
    let material_protos: [Mesh; FloorMaterial::COUNT] = std::array::from_fn(|i| {
        deck_cell_cuboid_mesh(DECK_SLAB_THICKNESS_M, floor_from_idx(i).color())
    });
    let material_mesh_handles: [Handle<Mesh>; FloorMaterial::COUNT] =
        std::array::from_fn(|i| meshes.add(material_protos[i].clone()));

    let slab_local_z = DECK_SLAB_THICKNESS_M * 0.5;

    for deck_i in 0..NUM_DECKS {
        let layout = layouts.deck(deck_i);
        let deck_z = deck_i as f32 * DECK_FLOOR_SPACING_M;

        let hull_outline = deck_sim_footprint_polygon(deck_i);
        let coarse_mesh = crate::deck_geometry::extruded_polygon_deck_mesh(
            &hull_outline,
            DECK_SLAB_THICKNESS_M,
            SHIP_LENGTH_M,
            SHIP_BEAM_M,
        );
        let coarse_handle = meshes.add(coarse_mesh);
        commands.spawn((
            Mesh3d(coarse_handle),
            MeshMaterial3d(clip_handle.clone()),
            Transform::from_xyz(0.0, 0.0, deck_z),
            Visibility::Inherited,
            DeckLayer(deck_i),
            DeckLodCoarseTier(deck_i),
        ));

        let mut buckets: [Vec<Vec3>; FloorMaterial::COUNT] = std::array::from_fn(|_| Vec::new());

        for (plan, cell) in layout.iter_cells() {
            let w = layout.index(plan).to_world_xy();
            let p = Vec3::new(w.x, w.y, slab_local_z);
            buckets[cell.floor.idx()].push(p);
        }

        for (mi, bucket_centers) in buckets.into_iter().enumerate() {
            if bucket_centers.is_empty() {
                continue;
            }
            let proto = &material_protos[mi];
            let mesh_h = material_mesh_handles[mi].clone();
            if bucket_centers.len() <= DECK_CELL_AUTOMATIC_INSTANCE_CAP {
                let cells = bucket_centers;
                let clip_inst = clip_handle.clone();
                commands.spawn_batch(cells.into_iter().map(move |t| {
                    (
                        Mesh3d(mesh_h.clone()),
                        MeshMaterial3d(clip_inst.clone()),
                        Transform::from_xyz(t.x, t.y, deck_z + t.z),
                        Visibility::Hidden,
                        DeckLayer(deck_i),
                        DeckLodFineTier(deck_i),
                    )
                }));
            } else if let Some(merged) =
                crate::deck_geometry::accumulate_translated_cell_instances(proto, &bucket_centers)
            {
                let handle = meshes.add(merged);
                commands.spawn((
                    Mesh3d(handle),
                    MeshMaterial3d(clip_handle.clone()),
                    Transform::from_xyz(0.0, 0.0, deck_z),
                    Visibility::Hidden,
                    DeckLayer(deck_i),
                    DeckLodFineTier(deck_i),
                ));
            }
        }

        let wall_height_m = DECK_SLAB_THICKNESS_M * 0.92;
        let side_walls =
            crate::plan_mesh::collect_deck_side_walls_3d(layout, deck_z, wall_height_m);
        if !side_walls.is_empty() {
            let wall_mesh = meshes.add(crate::deck_geometry::merged_cell_side_walls_mesh_3d(
                &side_walls,
            ));
            commands.spawn((
                Mesh3d(wall_mesh),
                MeshMaterial3d(clip_handle.clone()),
                Transform::IDENTITY,
                Visibility::Hidden,
                DeckLayer(deck_i),
                DeckLodFineTier(deck_i),
            ));
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn reload_deck_meshes_after_load(
    mut events: MessageReader<crate::ship_save::ShipLayoutsReplaced>,
    mut commands: Commands,
    layouts: Res<DeckLayouts>,
    mut meshes: ResMut<Assets<Mesh>>,
    clip: Res<SharedClipMaterial>,
    deck_entities: Query<Entity, With<DeckLayer>>,
    mut walk_points: ResMut<DeckFiveWalkPoints>,
) {
    for _ in events.read() {
        for entity in &deck_entities {
            commands.entity(entity).despawn();
        }
        spawn_deck_meshes(&mut commands, &mut meshes, &clip.0, &layouts);
        let deck_five_z = SIM_DECK_INDEX as f32 * DECK_FLOOR_SPACING_M;
        walk_points.0 = layouts
            .deck(SIM_DECK_INDEX)
            .centers()
            .into_iter()
            .map(|p| Vec3::new(p.x, p.y, deck_five_z))
            .collect();
    }
}

fn setup(
    mut commands: Commands,
    mut materials: ResMut<Assets<ShipClipMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    let deck_pattern = images.add(crate::deck_geometry::procedural_deck_plan_texture_image());
    let clip_handle = materials.add(ShipClipMaterial {
        clip_data: Vec4::new(cut_plane_world_z(NUM_DECKS - 1), ABOVE_DECK_ALPHA, 0.0, 0.0),
        deck_pattern,
    });
    commands.insert_resource(SharedClipMaterial(clip_handle.clone()));

    let rig = CameraRig::default();
    commands.spawn((
        Camera3d::default(),
        Camera {
            order: 0,
            ..default()
        },
        camera_rig_transform(&rig),
        GameCamera3d,
    ));

    let ui_camera = commands
        .spawn((
            Camera2d,
            Camera {
                order: 1,
                clear_color: ClearColorConfig::None,
                ..default()
            },
            UiCamera,
        ))
        .id();

    commands.insert_resource(GlobalAmbientLight {
        color: Color::WHITE,
        brightness: 40.0,
        affects_lightmapped_meshes: true,
    });
    commands.spawn((
        DirectionalLight {
            illuminance: 12000.0,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.9, 0.5, 0.0)),
    ));

    spawn_load_menu(&mut commands, ui_camera);

    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(50.0),
                position_type: PositionType::Absolute,
                top: Val::Px(0.0),
                left: Val::Px(0.0),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(12.0)),
                row_gap: Val::Px(8.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.05, 0.08, 0.12, 0.88)),
            UiTargetCamera(ui_camera),
            HudRoot,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new(""),
                TextFont {
                    font_size: 20.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                DeckInfoText,
            ));
            parent.spawn((
                Text::new("Hover: —"),
                TextFont {
                    font_size: 18.0,
                    ..default()
                },
                TextColor(Color::srgb(0.75, 0.82, 0.9)),
                HoverCellText,
            ));
        });
}

fn enter_game_world(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    layouts: Res<DeckLayouts>,
    clip: Res<SharedClipMaterial>,
    mut spawned: ResMut<GameWorldSpawned>,
    mut walk_points: ResMut<DeckFiveWalkPoints>,
    asset_server: Res<AssetServer>,
    mut rng: ResMut<SimRng>,
) {
    if spawned.0 {
        return;
    }
    spawn_deck_meshes(&mut commands, &mut meshes, &clip.0, &layouts);
    let deck_five_z = SIM_DECK_INDEX as f32 * DECK_FLOOR_SPACING_M;
    walk_points.0 = layouts
        .deck(SIM_DECK_INDEX)
        .centers()
        .into_iter()
        .map(|p| Vec3::new(p.x, p.y, deck_five_z))
        .collect();
    spawn_sim_npcs_inner(&mut commands, &asset_server, &mut rng, &walk_points);
    spawned.0 = true;
}

fn sync_game_camera_viewport(
    window: Single<&Window>,
    mut cameras: Query<&mut Camera, With<GameCamera3d>>,
) {
    let viewport = game_camera_viewport(&window);
    for mut camera in &mut cameras {
        camera.viewport = Some(viewport.clone());
    }
}

fn hover_cell_line(
    window: &Window,
    current_deck: usize,
    layouts: &DeckLayouts,
    cameras: &Query<(&Camera, &GlobalTransform), With<GameCamera3d>>,
) -> String {
    let Ok((camera, cam_tf)) = cameras.single() else {
        return "Hover: —".to_string();
    };
    let Some(cursor) = cursor_in_game_viewport(window, camera) else {
        return "Hover: —".to_string();
    };
    let Ok(ray) = camera.viewport_to_world(cam_tf, cursor) else {
        return "Hover: —".to_string();
    };
    let deck_z = deck_top_z(current_deck);
    let Some(hit) = ray_hit_plane_z(ray, deck_z) else {
        return "Hover: —".to_string();
    };
    let hull_xy = Vec2::new(hit.x, hit.y);
    format_cell_hover_line(hull_xy, current_deck, layouts)
}

fn update_hover_cell_label(
    window: Single<&Window>,
    current_deck: Res<CurrentDeck>,
    layouts: Res<DeckLayouts>,
    cameras: Query<(&Camera, &GlobalTransform), With<GameCamera3d>>,
    mut texts: Query<&mut Text, With<HoverCellText>>,
) {
    let hover_line = hover_cell_line(&window, current_deck.0, &layouts, &cameras);
    for mut text in &mut texts {
        text.0 = hover_line.clone();
    }
}

fn spawn_sim_npcs_inner(
    commands: &mut Commands,
    asset_server: &AssetServer,
    rng: &mut SimRng,
    walk_points: &DeckFiveWalkPoints,
) {
    let deck_five_z = SIM_DECK_INDEX as f32 * DECK_FLOOR_SPACING_M + DECK_SLAB_THICKNESS_M;
    let walk_points: Vec<Vec3> = walk_points
        .0
        .iter()
        .map(|p| Vec3::new(p.x, p.y, deck_five_z))
        .collect();
    if walk_points.is_empty() {
        return;
    }

    let mut shuffled_indices: Vec<usize> = (0..walk_points.len()).collect();
    for i in (1..shuffled_indices.len()).rev() {
        let j = rng.next_usize(i + 1);
        shuffled_indices.swap(i, j);
    }

    for (idx, model_path) in SIM_NPC_MODEL_PATHS.iter().enumerate() {
        let spawn_idx = shuffled_indices[idx % shuffled_indices.len()];
        let spawn_point = walk_points[spawn_idx];
        let target_point = walk_points[rng.next_usize(walk_points.len())];
        commands.spawn((
            SceneRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset(*model_path))),
            Transform::from_translation(spawn_point).with_scale(Vec3::splat(SIM_NPC_SCALE)),
            SimNpc {
                speed_m_s: SIM_NPC_SPEED_M_S,
            },
            WanderState {
                target: target_point,
            },
            Name::new(format!("SimNpc{}", idx + 1)),
        ));
    }
}

fn update_deck_lod(
    rig: Res<CameraRig>,
    mut pref: ResMut<DeckLodFinePreferred>,
    mut lod_queries: ParamSet<(
        Query<(&DeckLodFineTier, &mut Visibility)>,
        Query<(&DeckLodCoarseTier, &mut Visibility)>,
    )>,
) {
    let eye = camera_rig_transform(&rig).translation;
    for deck_i in 0..NUM_DECKS {
        let deck_mid_z = deck_i as f32 * DECK_FLOOR_SPACING_M + DECK_SLAB_THICKNESS_M * 0.5;
        let anchor = Vec3::new(rig.target.x, rig.target.y, deck_mid_z);
        let d = eye.distance(anchor);

        let mut want_fine = pref.0[deck_i];
        if d < LOD_FINE_WITHIN_M {
            want_fine = true;
        } else if d > LOD_COARSE_BEYOND_M {
            want_fine = false;
        }
        pref.0[deck_i] = want_fine;
    }

    for (tier, mut vis) in lod_queries.p0().iter_mut() {
        *vis = if pref.0[tier.0] {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }
    for (tier, mut vis) in lod_queries.p1().iter_mut() {
        *vis = if pref.0[tier.0] {
            Visibility::Hidden
        } else {
            Visibility::Inherited
        };
    }
}

fn deck_switch(keyboard: Res<ButtonInput<KeyCode>>, mut current_deck: ResMut<CurrentDeck>) {
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

fn sync_clip_material(
    current: Res<CurrentDeck>,
    shared: Res<SharedClipMaterial>,
    mut materials: ResMut<Assets<ShipClipMaterial>>,
) {
    if !current.is_changed() {
        return;
    }
    let Some(m) = materials.get_mut(&shared.0) else {
        return;
    };
    m.clip_data = Vec4::new(cut_plane_world_z(current.0), ABOVE_DECK_ALPHA, 0.0, 0.0);
}

/// NPCs use unlit GLB materials, so the [`ShipClipMaterial`] cut plane does not
/// reach them. Toggle [`Visibility`] manually so a character on Deck 5 disappears
/// when the user descends to a deck below it (otherwise the slab beneath their
/// feet is clipped away and they appear to float).
fn cull_npcs_above_cut(
    current: Res<CurrentDeck>,
    mut npcs: Query<(&Transform, &mut Visibility), With<SimNpc>>,
) {
    let cut_z = cut_plane_world_z(current.0);
    for (tf, mut vis) in &mut npcs {
        let next = if tf.translation.z <= cut_z + 0.05 {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
        if *vis != next {
            *vis = next;
        }
    }
}

/// Horizontal pan basis in world XY from current view (for WASD / MMB pan).
fn pan_basis_xy(rig: &CameraRig) -> (Vec3, Vec3) {
    let dir = dir_from_yaw_pitch(rig.yaw, rig.pitch);
    let forward_flat = Vec3::new(dir.x, dir.y, 0.0);
    if forward_flat.length_squared() < 1e-8 {
        return (Vec3::X, Vec3::Y);
    }
    let forward_flat = forward_flat.normalize();
    let right_flat = Vec3::new(-forward_flat.y, forward_flat.x, 0.0);
    (right_flat, forward_flat)
}

fn camera_controls(
    keyboard: Res<ButtonInput<KeyCode>>,
    mouse_btn: Res<ButtonInput<MouseButton>>,
    mut scroll_evr: MessageReader<MouseWheel>,
    mut motion_evr: MessageReader<MouseMotion>,
    time: Res<Time>,
    mut rig: ResMut<CameraRig>,
    mut cameras: Query<&mut Transform, With<GameCamera3d>>,
) {
    let dt = time.delta_secs();

    if keyboard.pressed(KeyCode::KeyQ) {
        rig.yaw -= CAMERA_YAW_SPEED_RAD_S * dt;
    }
    if keyboard.pressed(KeyCode::KeyE) {
        rig.yaw += CAMERA_YAW_SPEED_RAD_S * dt;
    }

    {
        let (right_flat, forward_flat) = pan_basis_xy(&rig);
        let pan_step = CAMERA_PAN_SPEED_M_S * dt;
        if keyboard.pressed(KeyCode::KeyW) {
            rig.target -= forward_flat * pan_step;
        }
        if keyboard.pressed(KeyCode::KeyS) {
            rig.target += forward_flat * pan_step;
        }
        if keyboard.pressed(KeyCode::KeyD) {
            rig.target += right_flat * pan_step;
        }
        if keyboard.pressed(KeyCode::KeyA) {
            rig.target -= right_flat * pan_step;
        }
    }
    if keyboard.pressed(KeyCode::KeyR) {
        rig.target.z += CAMERA_VERTICAL_SPEED_M_S * dt;
    }
    if keyboard.pressed(KeyCode::KeyF) {
        rig.target.z -= CAMERA_VERTICAL_SPEED_M_S * dt;
    }

    let zoom_linear = CAMERA_ZOOM_SPEED_M_S * dt;
    if keyboard.pressed(KeyCode::KeyZ) {
        rig.distance = (rig.distance - zoom_linear).clamp(CAM_MIN_DISTANCE_M, CAM_MAX_DISTANCE_M);
    }
    if keyboard.pressed(KeyCode::KeyX) {
        rig.distance = (rig.distance + zoom_linear).clamp(CAM_MIN_DISTANCE_M, CAM_MAX_DISTANCE_M);
    }

    for ev in scroll_evr.read() {
        let dy = match ev.unit {
            MouseScrollUnit::Line => ev.y,
            MouseScrollUnit::Pixel => ev.y * 0.015,
        };
        let factor = 1.0 - dy * CAMERA_SCROLL_ZOOM_FACTOR;
        rig.distance = (rig.distance * factor).clamp(CAM_MIN_DISTANCE_M, CAM_MAX_DISTANCE_M);
    }

    if mouse_btn.pressed(MouseButton::Right) {
        for ev in motion_evr.read() {
            rig.yaw -= ev.delta.x * CAMERA_MOUSE_ORBIT_SENS;
            rig.pitch = (rig.pitch - ev.delta.y * CAMERA_MOUSE_ORBIT_SENS)
                .clamp(CAM_PITCH_MIN, CAM_PITCH_MAX);
        }
    } else if mouse_btn.pressed(MouseButton::Middle) {
        let (right_flat, forward_flat) = pan_basis_xy(&rig);
        let scale = rig.distance * CAMERA_MOUSE_PAN_SENS;
        for ev in motion_evr.read() {
            rig.target += -right_flat * ev.delta.x * scale + forward_flat * ev.delta.y * scale;
        }
    } else {
        for _ in motion_evr.read() {}
    }

    let tf = camera_rig_transform(&rig);
    for mut cam_tf in &mut cameras {
        *cam_tf = tf;
    }
}

fn sim_npc_wander(
    time: Res<Time>,
    walk_points: Option<Res<DeckFiveWalkPoints>>,
    mut rng: ResMut<SimRng>,
    mut npcs: Query<(&SimNpc, &mut Transform, &mut WanderState)>,
) {
    let Some(walk_points) = walk_points else {
        return;
    };
    if walk_points.0.is_empty() {
        return;
    }

    for (npc, mut tf, mut state) in &mut npcs {
        let mut to_target = state.target - tf.translation;
        if to_target.length() <= SIM_TARGET_REACHED_M {
            state.target = walk_points.0[rng.next_usize(walk_points.0.len())];
            to_target = state.target - tf.translation;
        }

        let distance = to_target.length();
        if distance <= f32::EPSILON {
            continue;
        }

        let move_step = (npc.speed_m_s * time.delta_secs()).min(distance);
        let dir = to_target / distance;
        tf.translation += dir * move_step;
        tf.look_to(Vec3::new(dir.x, dir.y, 0.0), Vec3::Z);
    }
}

fn update_deck_label(
    current_deck: Res<CurrentDeck>,
    rig: Res<CameraRig>,
    mut query: Query<&mut Text, With<DeckInfoText>>,
) {
    if !current_deck.is_changed() && !rig.is_changed() {
        return;
    }
    for mut text in &mut query {
        text.0 = deck_info_text_3d(current_deck.0);
    }
}

fn init_deck_info_text(
    current_deck: Res<CurrentDeck>,
    mut query: Query<&mut Text, With<DeckInfoText>>,
) {
    for mut text in &mut query {
        text.0 = deck_info_text_3d(current_deck.0);
    }
}

fn log_npc_heights_once(
    mut log_state: ResMut<NpcHeightLogState>,
    npcs: Query<(Entity, &Name, Option<&Children>), With<SimNpc>>,
    children_query: Query<&Children>,
    aabbs: Query<(&GlobalTransform, &Aabb)>,
) {
    for (npc_entity, name, children) in &npcs {
        if log_state.logged_roots.contains(&npc_entity) {
            continue;
        }
        let Some(children) = children else {
            continue;
        };

        let mut stack: Vec<Entity> = children.iter().collect();
        let mut min_z = f32::INFINITY;
        let mut max_z = f32::NEG_INFINITY;
        let mut found_mesh_bounds = false;

        while let Some(entity) = stack.pop() {
            if let Ok(children) = children_query.get(entity) {
                stack.extend(children.iter());
            }
            let Ok((global_tf, aabb)) = aabbs.get(entity) else {
                continue;
            };

            let center = aabb.center;
            let half = aabb.half_extents;
            for sx in [-1.0, 1.0] {
                for sy in [-1.0, 1.0] {
                    for sz in [-1.0, 1.0] {
                        let local_corner =
                            center + Vec3A::new(sx * half.x, sy * half.y, sz * half.z);
                        let world_corner = global_tf.affine().transform_point3(local_corner.into());
                        min_z = min_z.min(world_corner.z);
                        max_z = max_z.max(world_corner.z);
                    }
                }
            }
            found_mesh_bounds = true;
        }

        if found_mesh_bounds {
            let height_m = (max_z - min_z).max(0.0);
            println!("NPC {} measured height: {height_m:.2} m", name.as_str());
            log_state.logged_roots.insert(npc_entity);
        }
    }
}
