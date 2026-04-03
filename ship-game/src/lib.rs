//! Ship Game — 3D cruise-ship decks with horizontal cut plane (Bevy, WASM).
//!
//! **World space uses SI metres:** one unit of `Vec3`, mesh positions, and camera distance is **1 m**.
//!
//! Deck footprint uses [`ship_hull::SHIP_BEAM_M`] × [`ship_hull::SHIP_LENGTH_M`] (60 m beam, ~5.3:1 L/B from
//! `assets/reference_floorplan_deck10.png`). Decks 10+ use a courtyard void and U-stern from
//! [`ship_hull::deck_hull_polygon_upper`]. Tiles are axis-aligned on **XY**; **+Y** bow, **±X** port/starboard;
//! decks stack on **+Z**; the clip shader removes fragments above the cut height.

mod shader_embed;
mod ship_hull;

use bevy::asset::AssetPath;
use bevy::input::mouse::{MouseMotion, MouseScrollUnit, MouseWheel};
use bevy::pbr::{Material, MaterialMeshBundle, MaterialPlugin};
use bevy::prelude::*;
use bevy::render::render_resource::{AsBindGroup, ShaderRef};
use shader_embed::ShipShaderEmbedPlugin;
use ship_hull::{
    deck_hull_polygon, deck_hull_polygon_upper, deck_tile_centers, deck_tile_centers_upper,
    is_perimeter_tile, FIRST_UPPER_DECK_STYLE_INDEX, SHIP_BEAM_M, SHIP_LENGTH_M,
    UPPER_VOID_HALF_WIDTH_M, UPPER_VOID_Y_AFT_M, UPPER_VOID_Y_FWD_M,
};

const NUM_DECKS: usize = 20;

const DECK_NAMES: [&str; NUM_DECKS] = [
    "Engine Deck",
    "Orlop Deck",
    "Hold Deck",
    "Lower Deck",
    "Second Deck",
    "First Deck",
    "Main Deck",
    "Upper Deck",
    "Promenade Deck",
    "Lido Deck",
    "Boat Deck",
    "Bridge Deck",
    "Sports Deck",
    "Observation Deck",
    "Spa Deck",
    "Pool Deck",
    "Sky Deck",
    "Terrace Deck",
    "Crown Deck",
    "Sun Deck",
];

/// Square deck cell size (m). ~3–4 m matches stateroom-scale on the deck plan scale.
const TILE_CELL_M: f32 = 3.8;
/// Slight inset so neighbouring slabs do not z-fight at vertical faces.
const TILE_VISUAL_SCALE: f32 = 0.92;

/// Vertical spacing between deck floors (m along world +Z): **one deck level every 3 m**.
const DECK_FLOOR_SPACING_M: f32 = 3.0;
/// Extruded slab thickness (m); slightly under spacing so slabs do not z-fight deck-to-deck.
const DECK_SLAB_THICKNESS_M: f32 = 2.88;

/// Pan speed (m/s) for WASD and mouse middle-drag.
const CAMERA_PAN_SPEED_M_S: f32 = 520.0;
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
const CAM_MIN_DISTANCE_M: f32 = 180.0;
const CAM_MAX_DISTANCE_M: f32 = 6200.0;
/// Orbit pitch limits (radians from horizontal); keep camera above the XY plane.
const CAM_PITCH_MIN: f32 = 0.15;
const CAM_PITCH_MAX: f32 = 1.42;

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
struct DeckLabel;

#[derive(Component)]
struct DeckLayer(#[allow(dead_code)] usize);

#[derive(Resource)]
struct CurrentDeck(usize);

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

#[derive(Asset, TypePath, AsBindGroup, Clone, Copy)]
struct ShipClipMaterial {
    /// `.x` = world-space Z above which fragments are clipped (rest unused).
    #[uniform(0)]
    clip_data: Vec4,
}

impl Material for ShipClipMaterial {
    fn fragment_shader() -> ShaderRef {
        ShaderRef::Path(AssetPath::from(CLIP_SHADER_FORWARD))
    }

    fn prepass_fragment_shader() -> ShaderRef {
        ShaderRef::Path(AssetPath::from(CLIP_SHADER_PREPASS))
    }

    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Opaque
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn run() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Ship Game - Time Helm".into(),
                        canvas: Some("#ship-game-canvas".into()),
                        fit_canvas_to_parent: true,
                        ..default()
                    }),
                    ..default()
                })
                .set(ImagePlugin::default_nearest()),
        )
        .add_plugins(ShipShaderEmbedPlugin)
        .add_plugins(MaterialPlugin::<ShipClipMaterial>::default())
        .insert_resource(CurrentDeck(NUM_DECKS - 1))
        .insert_resource(CameraRig::default())
        .insert_resource(ClearColor::default())
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                deck_switch,
                camera_controls,
                sync_clip_material,
                update_deck_label,
            ),
        )
        .run();
}

impl Default for CameraRig {
    fn default() -> Self {
        let target = Vec3::new(0.0, 0.0, ship_stack_mid_z());
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

fn cut_plane_world_z(current_deck: usize) -> f32 {
    (current_deck + 1) as f32 * DECK_FLOOR_SPACING_M
}

fn ship_stack_mid_z() -> f32 {
    NUM_DECKS as f32 * DECK_FLOOR_SPACING_M * 0.5
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
fn deck_tile_cuboid_mesh(cell_m: f32, thickness_m: f32, color: Color) -> Mesh {
    let s = cell_m * TILE_VISUAL_SCALE;
    let mut mesh = Mesh::from(Cuboid::new(s, s, thickness_m));
    let n = mesh.count_vertices();
    let c: LinearRgba = color.into();
    let ca = c.to_f32_array();
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, vec![ca; n]);
    mesh
}

/// Rough zones inspired by the reference floorplan (outer yellow cabins, inner pink block).
fn outer_cabin_zone(p: Vec2) -> bool {
    p.x.abs() > SHIP_BEAM_M * 0.32 && p.y > -SHIP_LENGTH_M * 0.36 && p.y < SHIP_LENGTH_M * 0.26
}

fn inner_cabin_zone(p: Vec2) -> bool {
    p.x.abs() < SHIP_BEAM_M * 0.24 && p.y > -SHIP_LENGTH_M * 0.32 && p.y < SHIP_LENGTH_M * 0.22
}

fn window_strip_zone(p: Vec2) -> bool {
    p.y > SHIP_LENGTH_M * 0.12 && p.x.abs() > SHIP_BEAM_M * 0.34
}

// --- Upper decks (Deck 10+): colours aligned with `reference_floorplan_deck10.png` ---

fn upper_window_strip_zone(p: Vec2) -> bool {
    p.x.abs() > SHIP_BEAM_M * 0.34
        && (p.y > SHIP_LENGTH_M * 0.12
            || (p.y < UPPER_VOID_Y_FWD_M + 6.0 && p.y > UPPER_VOID_Y_AFT_M - SHIP_LENGTH_M * 0.04))
}

fn upper_outer_balcony_zone(p: Vec2) -> bool {
    let inner = UPPER_VOID_HALF_WIDTH_M + 3.2;
    p.x.abs() > inner
        && p.y < UPPER_VOID_Y_FWD_M + SHIP_LENGTH_M * 0.14
        && p.y > UPPER_VOID_Y_AFT_M - SHIP_LENGTH_M * 0.05
}

fn upper_inner_courtyard_zone(p: Vec2) -> bool {
    let hb = SHIP_BEAM_M * 0.5;
    let vw = UPPER_VOID_HALF_WIDTH_M;
    p.y < UPPER_VOID_Y_FWD_M - 1.5
        && p.y > UPPER_VOID_Y_AFT_M + 1.5
        && p.x.abs() > vw + 1.6
        && p.x.abs() < hb - 4.0
}

fn upper_bow_forward_block(p: Vec2) -> bool {
    p.y > UPPER_VOID_Y_FWD_M - 2.0
}

fn upper_stern_wing_body(p: Vec2) -> bool {
    p.y < UPPER_VOID_Y_AFT_M && p.x.abs() > UPPER_VOID_HALF_WIDTH_M + 1.2
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ShipClipMaterial>>,
) {
    let clip_handle = materials.add(ShipClipMaterial {
        clip_data: Vec4::new(cut_plane_world_z(NUM_DECKS - 1), 0.0, 0.0, 0.0),
    });
    commands.insert_resource(SharedClipMaterial(clip_handle.clone()));

    let rig = CameraRig::default();
    commands.spawn((
        Camera3dBundle {
            transform: camera_rig_transform(&rig),
            camera: Camera {
                order: 0,
                ..default()
            },
            ..default()
        },
        GameCamera3d,
    ));

    let ui_camera = commands
        .spawn((
            Camera2dBundle {
                camera: Camera {
                    order: 1,
                    clear_color: ClearColorConfig::None,
                    ..default()
                },
                ..default()
            },
            UiCamera,
        ))
        .id();

    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 0.35,
    });
    commands.spawn(DirectionalLightBundle {
        transform: Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.9, 0.5, 0.0)),
        directional_light: DirectionalLight {
            illuminance: 12000.0,
            ..default()
        },
        ..default()
    });

    let hull_lower = deck_hull_polygon();
    let hull_upper = deck_hull_polygon_upper();
    let tile_centers_lower = deck_tile_centers(TILE_CELL_M);
    let tile_centers_upper = deck_tile_centers_upper(TILE_CELL_M);

    let edge_deck = Color::srgb(0.38, 0.3, 0.24);
    let window_color = Color::srgb(0.42, 0.62, 0.9);
    let outer_cabin = Color::srgb(0.95, 0.82, 0.35);
    let inner_cabin = Color::srgb(0.92, 0.55, 0.72);
    let public_deck = Color::srgb(0.78, 0.86, 0.92);

    let upper_outer_red = Color::srgb(0.78, 0.2, 0.22);
    let upper_inner_peach = Color::srgb(0.93, 0.68, 0.55);
    let upper_stern_teal = Color::srgb(0.16, 0.44, 0.48);
    let upper_bow_side = Color::srgb(0.9, 0.8, 0.28);
    let upper_bow_core = Color::srgb(0.86, 0.42, 0.66);
    let upper_corridor = Color::srgb(0.55, 0.53, 0.51);

    let mesh_hull = meshes.add(deck_tile_cuboid_mesh(
        TILE_CELL_M,
        DECK_SLAB_THICKNESS_M,
        edge_deck,
    ));
    let mesh_window = meshes.add(deck_tile_cuboid_mesh(
        TILE_CELL_M,
        DECK_SLAB_THICKNESS_M,
        window_color,
    ));
    let mesh_outer = meshes.add(deck_tile_cuboid_mesh(
        TILE_CELL_M,
        DECK_SLAB_THICKNESS_M,
        outer_cabin,
    ));
    let mesh_inner = meshes.add(deck_tile_cuboid_mesh(
        TILE_CELL_M,
        DECK_SLAB_THICKNESS_M,
        inner_cabin,
    ));
    let mesh_public = meshes.add(deck_tile_cuboid_mesh(
        TILE_CELL_M,
        DECK_SLAB_THICKNESS_M,
        public_deck,
    ));
    let mesh_upper_red = meshes.add(deck_tile_cuboid_mesh(
        TILE_CELL_M,
        DECK_SLAB_THICKNESS_M,
        upper_outer_red,
    ));
    let mesh_upper_peach = meshes.add(deck_tile_cuboid_mesh(
        TILE_CELL_M,
        DECK_SLAB_THICKNESS_M,
        upper_inner_peach,
    ));
    let mesh_upper_teal = meshes.add(deck_tile_cuboid_mesh(
        TILE_CELL_M,
        DECK_SLAB_THICKNESS_M,
        upper_stern_teal,
    ));
    let mesh_upper_bow_side = meshes.add(deck_tile_cuboid_mesh(
        TILE_CELL_M,
        DECK_SLAB_THICKNESS_M,
        upper_bow_side,
    ));
    let mesh_upper_bow_core = meshes.add(deck_tile_cuboid_mesh(
        TILE_CELL_M,
        DECK_SLAB_THICKNESS_M,
        upper_bow_core,
    ));
    let mesh_upper_corridor = meshes.add(deck_tile_cuboid_mesh(
        TILE_CELL_M,
        DECK_SLAB_THICKNESS_M,
        upper_corridor,
    ));

    for deck_i in 0..NUM_DECKS {
        let hue = 0.52 + (deck_i as f32 * 0.012);
        let base_tint = Color::hsl(hue * 360.0 % 360.0, 0.28, 0.42);
        let mesh_deck_base = meshes.add(deck_tile_cuboid_mesh(
            TILE_CELL_M,
            DECK_SLAB_THICKNESS_M,
            base_tint,
        ));

        let deck_z = deck_i as f32 * DECK_FLOOR_SPACING_M;

        commands
            .spawn((
                TransformBundle::from_transform(Transform::from_xyz(0.0, 0.0, deck_z)),
                VisibilityBundle::default(),
                DeckLayer(deck_i),
            ))
            .with_children(|deck| {
                let hull: &[Vec2] = if deck_i >= FIRST_UPPER_DECK_STYLE_INDEX {
                    hull_upper.as_slice()
                } else {
                    hull_lower.as_slice()
                };
                let centers: &[Vec2] = if deck_i >= FIRST_UPPER_DECK_STYLE_INDEX {
                    tile_centers_upper.as_slice()
                } else {
                    tile_centers_lower.as_slice()
                };

                for c in centers {
                    let edge = is_perimeter_tile(*c, TILE_CELL_M, hull);
                    let mesh = if edge {
                        if deck_i >= FIRST_UPPER_DECK_STYLE_INDEX {
                            if upper_window_strip_zone(*c) {
                                &mesh_window
                            } else {
                                &mesh_hull
                            }
                        } else if window_strip_zone(*c) {
                            &mesh_window
                        } else {
                            &mesh_hull
                        }
                    } else if deck_i >= FIRST_UPPER_DECK_STYLE_INDEX {
                        if upper_stern_wing_body(*c) {
                            &mesh_upper_teal
                        } else if upper_bow_forward_block(*c) {
                            if c.x.abs() > SHIP_BEAM_M * 0.26 {
                                &mesh_upper_bow_side
                            } else {
                                &mesh_upper_bow_core
                            }
                        } else if upper_inner_courtyard_zone(*c) {
                            &mesh_upper_peach
                        } else if upper_outer_balcony_zone(*c) {
                            &mesh_upper_red
                        } else if c.y < -SHIP_LENGTH_M * 0.3 {
                            &mesh_public
                        } else {
                            &mesh_upper_corridor
                        }
                    } else if inner_cabin_zone(*c) {
                        &mesh_inner
                    } else if outer_cabin_zone(*c) {
                        &mesh_outer
                    } else if c.y < -SHIP_LENGTH_M * 0.28 {
                        &mesh_public
                    } else {
                        &mesh_deck_base
                    };

                    deck.spawn(MaterialMeshBundle {
                        mesh: (*mesh).clone(),
                        material: clip_handle.clone(),
                        transform: Transform::from_xyz(c.x, c.y, DECK_SLAB_THICKNESS_M * 0.5),
                        ..default()
                    });
                }
            });
    }

    commands.spawn((
        TextBundle {
            style: Style {
                position_type: PositionType::Absolute,
                top: Val::Px(10.0),
                left: Val::Px(10.0),
                ..default()
            },
            text: Text::from_section(
                "",
                TextStyle {
                    font_size: 22.0,
                    color: Color::WHITE,
                    ..default()
                },
            ),
            ..default()
        },
        TargetCamera(ui_camera),
        DeckLabel,
    ));
}

fn deck_switch(keyboard: Res<ButtonInput<KeyCode>>, mut current_deck: ResMut<CurrentDeck>) {
    if keyboard.just_pressed(KeyCode::PageUp) && current_deck.0 < NUM_DECKS - 1 {
        current_deck.0 += 1;
    }
    if keyboard.just_pressed(KeyCode::PageDown) && current_deck.0 > 0 {
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
    m.clip_data = Vec4::new(cut_plane_world_z(current.0), 0.0, 0.0, 0.0);
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
    mut scroll_evr: EventReader<MouseWheel>,
    mut motion_evr: EventReader<MouseMotion>,
    time: Res<Time>,
    mut rig: ResMut<CameraRig>,
    mut cameras: Query<&mut Transform, With<GameCamera3d>>,
) {
    let dt = time.delta_seconds();

    if keyboard.pressed(KeyCode::KeyQ) {
        rig.yaw += CAMERA_YAW_SPEED_RAD_S * dt;
    }
    if keyboard.pressed(KeyCode::KeyE) {
        rig.yaw -= CAMERA_YAW_SPEED_RAD_S * dt;
    }

    {
        let (right_flat, forward_flat) = pan_basis_xy(&rig);
        let pan_step = CAMERA_PAN_SPEED_M_S * dt;
        if keyboard.pressed(KeyCode::KeyW) {
            rig.target += forward_flat * pan_step;
        }
        if keyboard.pressed(KeyCode::KeyS) {
            rig.target -= forward_flat * pan_step;
        }
        if keyboard.pressed(KeyCode::KeyD) {
            rig.target += right_flat * pan_step;
        }
        if keyboard.pressed(KeyCode::KeyA) {
            rig.target -= right_flat * pan_step;
        }
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

fn update_deck_label(
    current_deck: Res<CurrentDeck>,
    rig: Res<CameraRig>,
    mut query: Query<&mut Text, With<DeckLabel>>,
) {
    if !current_deck.is_changed() && !rig.is_changed() {
        return;
    }
    for mut text in &mut query {
        text.sections[0].value = format!(
            "Deck {}/{}: {} | hull {:.0} m × {:.0} m\nQ/E: orbit | WASD: pan | Z/X: zoom | RMB: orbit | MMB: pan | wheel: zoom | PgUp/PgDn: deck",
            current_deck.0 + 1,
            NUM_DECKS,
            DECK_NAMES[current_deck.0],
            SHIP_LENGTH_M,
            SHIP_BEAM_M,
        );
    }
}
