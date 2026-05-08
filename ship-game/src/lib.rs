//! Ship Game — 3D cruise-ship decks with horizontal cut plane (Bevy, WASM).
//!
//! **World space uses SI metres:** one unit of `Vec3`, mesh positions, and camera distance is **1 m**.
//!
//! Deck footprint uses [`ship_hull::SHIP_BEAM_M`] × [`ship_hull::SHIP_LENGTH_M`] (60 m beam, ~5.3:1 L/B from
//! `assets/icon_of_the_seas/floorplan_deck_10.png`). Decks 10+ use a courtyard void and U-stern from
//! [`ship_hull::deck_hull_polygon_upper`]. Tiles are axis-aligned on **XY**; **+Y** bow, **±X** port/starboard;
//! decks stack on **+Z**; the clip shader removes fragments above the cut height.

mod shader_embed;
mod ship_hull;

use bevy::input::mouse::{MouseMotion, MouseScrollUnit, MouseWheel};
use bevy::pbr::{Material, MaterialPlugin, MeshMaterial3d};
use bevy::prelude::*;
use bevy::render::render_resource::AsBindGroup;
use bevy::shader::ShaderRef;
use shader_embed::ShipShaderEmbedPlugin;
use ship_hull::{
    deck_tile_centers, deck_tile_centers_upper, FIRST_UPPER_DECK_STYLE_INDEX, SHIP_BEAM_M,
    SHIP_LENGTH_M,
};
use std::collections::HashSet;

const NUM_DECKS: usize = 20;
const SIM_DECK_INDEX: usize = 4; // Deck 5 (human-facing numbering)
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

#[derive(Clone)]
struct DeckTiles {
    centers: Vec<Vec2>,
    occupied: HashSet<(i32, i32)>,
}

#[derive(Resource)]
struct DeckLayouts(Vec<DeckTiles>);

#[derive(Clone, Copy)]
struct DeckProfile {
    half_beam_scale: f32,
    y_aft: f32,
    y_fwd: f32,
    bow_taper: f32,
    stern_taper: f32,
    courtyard_half_width: f32,
    courtyard_y_aft: f32,
    courtyard_y_fwd: f32,
}

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
        CLIP_SHADER_FORWARD.into()
    }

    fn prepass_fragment_shader() -> ShaderRef {
        CLIP_SHADER_PREPASS.into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Opaque
    }
}

fn primary_window() -> Window {
    #[cfg(target_arch = "wasm32")]
    {
        Window {
            title: "Ship Game - Time Helm".into(),
            canvas: Some("#ship-game-canvas".into()),
            fit_canvas_to_parent: true,
            ..default()
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        Window {
            title: "Ship Game - Time Helm".into(),
            ..default()
        }
    }
}

fn asset_plugin() -> AssetPlugin {
    #[cfg(target_arch = "wasm32")]
    {
        // Skip HTTP fetches for `.meta` sidecars: static hosting has no meta files, and Bevy
        // would fall back to default meta anyway (`AssetReaderError::NotFound`).
        AssetPlugin {
            meta_check: bevy::asset::AssetMetaCheck::Never,
            ..default()
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        AssetPlugin {
            file_path: "../assets".into(),
            ..default()
        }
    }
}

fn run_app() {
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
        .add_plugins(ShipShaderEmbedPlugin)
        .add_plugins(MaterialPlugin::<ShipClipMaterial>::default())
        .insert_resource(CurrentDeck(SIM_DECK_INDEX))
        .insert_resource(CameraRig::default())
        .insert_resource(SimRng::default())
        .insert_resource(ClearColor::default())
        .add_systems(Startup, (setup, spawn_sim_npcs.after(setup)))
        .add_systems(
            Update,
            (
                deck_switch,
                focus_camera_on_current_deck,
                camera_controls,
                sim_npc_wander,
                sync_clip_material,
                cull_npcs_above_cut,
                update_deck_label,
            ),
        )
        .run();
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn run() {
    run_app();
}

#[cfg(not(target_arch = "wasm32"))]
pub fn run_native() {
    run_app();
}

impl Default for CameraRig {
    fn default() -> Self {
        let target = Vec3::new(0.0, 0.0, focused_deck_target_z(SIM_DECK_INDEX));
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

fn deck_profile(deck_index: usize) -> DeckProfile {
    // Hand-authored deck-by-deck silhouettes inspired by the reference plans.
    const P: [DeckProfile; NUM_DECKS] = [
        DeckProfile {
            half_beam_scale: 0.42,
            y_aft: -SHIP_LENGTH_M * 0.43,
            y_fwd: SHIP_LENGTH_M * 0.14,
            bow_taper: 0.55,
            stern_taper: 0.35,
            courtyard_half_width: 0.0,
            courtyard_y_aft: 0.0,
            courtyard_y_fwd: 0.0,
        }, // 1
        DeckProfile {
            half_beam_scale: 0.52,
            y_aft: -SHIP_LENGTH_M * 0.45,
            y_fwd: SHIP_LENGTH_M * 0.20,
            bow_taper: 0.48,
            stern_taper: 0.30,
            courtyard_half_width: 0.0,
            courtyard_y_aft: 0.0,
            courtyard_y_fwd: 0.0,
        }, // 2
        DeckProfile {
            half_beam_scale: 0.70,
            y_aft: -SHIP_LENGTH_M * 0.47,
            y_fwd: SHIP_LENGTH_M * 0.26,
            bow_taper: 0.44,
            stern_taper: 0.25,
            courtyard_half_width: 0.0,
            courtyard_y_aft: 0.0,
            courtyard_y_fwd: 0.0,
        }, // 3
        DeckProfile {
            half_beam_scale: 0.88,
            y_aft: -SHIP_LENGTH_M * 0.49,
            y_fwd: SHIP_LENGTH_M * 0.38,
            bow_taper: 0.32,
            stern_taper: 0.18,
            courtyard_half_width: 0.0,
            courtyard_y_aft: 0.0,
            courtyard_y_fwd: 0.0,
        }, // 4
        DeckProfile {
            half_beam_scale: 0.96,
            y_aft: -SHIP_LENGTH_M * 0.50,
            y_fwd: SHIP_LENGTH_M * 0.44,
            bow_taper: 0.26,
            stern_taper: 0.12,
            courtyard_half_width: 0.0,
            courtyard_y_aft: 0.0,
            courtyard_y_fwd: 0.0,
        }, // 5
        DeckProfile {
            half_beam_scale: 0.98,
            y_aft: -SHIP_LENGTH_M * 0.50,
            y_fwd: SHIP_LENGTH_M * 0.46,
            bow_taper: 0.24,
            stern_taper: 0.11,
            courtyard_half_width: 0.0,
            courtyard_y_aft: 0.0,
            courtyard_y_fwd: 0.0,
        }, // 6
        DeckProfile {
            half_beam_scale: 0.98,
            y_aft: -SHIP_LENGTH_M * 0.50,
            y_fwd: SHIP_LENGTH_M * 0.47,
            bow_taper: 0.23,
            stern_taper: 0.10,
            courtyard_half_width: 0.0,
            courtyard_y_aft: 0.0,
            courtyard_y_fwd: 0.0,
        }, // 7
        DeckProfile {
            half_beam_scale: 0.97,
            y_aft: -SHIP_LENGTH_M * 0.50,
            y_fwd: SHIP_LENGTH_M * 0.47,
            bow_taper: 0.23,
            stern_taper: 0.10,
            courtyard_half_width: 0.0,
            courtyard_y_aft: 0.0,
            courtyard_y_fwd: 0.0,
        }, // 8
        DeckProfile {
            half_beam_scale: 0.96,
            y_aft: -SHIP_LENGTH_M * 0.50,
            y_fwd: SHIP_LENGTH_M * 0.47,
            bow_taper: 0.24,
            stern_taper: 0.10,
            courtyard_half_width: 0.0,
            courtyard_y_aft: 0.0,
            courtyard_y_fwd: 0.0,
        }, // 9
        DeckProfile {
            half_beam_scale: 0.94,
            y_aft: -SHIP_LENGTH_M * 0.49,
            y_fwd: SHIP_LENGTH_M * 0.46,
            bow_taper: 0.25,
            stern_taper: 0.16,
            courtyard_half_width: 9.0,
            courtyard_y_aft: -SHIP_LENGTH_M * 0.26,
            courtyard_y_fwd: SHIP_LENGTH_M * 0.21,
        }, // 10
        DeckProfile {
            half_beam_scale: 0.93,
            y_aft: -SHIP_LENGTH_M * 0.49,
            y_fwd: SHIP_LENGTH_M * 0.45,
            bow_taper: 0.26,
            stern_taper: 0.17,
            courtyard_half_width: 9.5,
            courtyard_y_aft: -SHIP_LENGTH_M * 0.26,
            courtyard_y_fwd: SHIP_LENGTH_M * 0.21,
        }, // 11
        DeckProfile {
            half_beam_scale: 0.92,
            y_aft: -SHIP_LENGTH_M * 0.48,
            y_fwd: SHIP_LENGTH_M * 0.44,
            bow_taper: 0.27,
            stern_taper: 0.18,
            courtyard_half_width: 10.0,
            courtyard_y_aft: -SHIP_LENGTH_M * 0.25,
            courtyard_y_fwd: SHIP_LENGTH_M * 0.20,
        }, // 12
        DeckProfile {
            half_beam_scale: 0.89,
            y_aft: -SHIP_LENGTH_M * 0.47,
            y_fwd: SHIP_LENGTH_M * 0.42,
            bow_taper: 0.29,
            stern_taper: 0.20,
            courtyard_half_width: 9.2,
            courtyard_y_aft: -SHIP_LENGTH_M * 0.23,
            courtyard_y_fwd: SHIP_LENGTH_M * 0.18,
        }, // 13
        DeckProfile {
            half_beam_scale: 0.86,
            y_aft: -SHIP_LENGTH_M * 0.46,
            y_fwd: SHIP_LENGTH_M * 0.40,
            bow_taper: 0.31,
            stern_taper: 0.22,
            courtyard_half_width: 8.2,
            courtyard_y_aft: -SHIP_LENGTH_M * 0.20,
            courtyard_y_fwd: SHIP_LENGTH_M * 0.16,
        }, // 14
        DeckProfile {
            half_beam_scale: 0.82,
            y_aft: -SHIP_LENGTH_M * 0.45,
            y_fwd: SHIP_LENGTH_M * 0.38,
            bow_taper: 0.32,
            stern_taper: 0.23,
            courtyard_half_width: 7.4,
            courtyard_y_aft: -SHIP_LENGTH_M * 0.18,
            courtyard_y_fwd: SHIP_LENGTH_M * 0.14,
        }, // 15
        DeckProfile {
            half_beam_scale: 0.78,
            y_aft: -SHIP_LENGTH_M * 0.43,
            y_fwd: SHIP_LENGTH_M * 0.36,
            bow_taper: 0.34,
            stern_taper: 0.24,
            courtyard_half_width: 6.2,
            courtyard_y_aft: -SHIP_LENGTH_M * 0.15,
            courtyard_y_fwd: SHIP_LENGTH_M * 0.12,
        }, // 16
        DeckProfile {
            half_beam_scale: 0.73,
            y_aft: -SHIP_LENGTH_M * 0.40,
            y_fwd: SHIP_LENGTH_M * 0.34,
            bow_taper: 0.35,
            stern_taper: 0.25,
            courtyard_half_width: 4.8,
            courtyard_y_aft: -SHIP_LENGTH_M * 0.13,
            courtyard_y_fwd: SHIP_LENGTH_M * 0.10,
        }, // 17
        DeckProfile {
            half_beam_scale: 0.68,
            y_aft: -SHIP_LENGTH_M * 0.37,
            y_fwd: SHIP_LENGTH_M * 0.31,
            bow_taper: 0.37,
            stern_taper: 0.27,
            courtyard_half_width: 0.0,
            courtyard_y_aft: 0.0,
            courtyard_y_fwd: 0.0,
        }, // 18
        DeckProfile {
            half_beam_scale: 0.63,
            y_aft: -SHIP_LENGTH_M * 0.34,
            y_fwd: SHIP_LENGTH_M * 0.28,
            bow_taper: 0.40,
            stern_taper: 0.30,
            courtyard_half_width: 0.0,
            courtyard_y_aft: 0.0,
            courtyard_y_fwd: 0.0,
        }, // 19
        DeckProfile {
            half_beam_scale: 0.56,
            y_aft: -SHIP_LENGTH_M * 0.30,
            y_fwd: SHIP_LENGTH_M * 0.24,
            bow_taper: 0.44,
            stern_taper: 0.34,
            courtyard_half_width: 0.0,
            courtyard_y_aft: 0.0,
            courtyard_y_fwd: 0.0,
        }, // 20
    ];
    P[deck_index.min(NUM_DECKS - 1)]
}

fn profile_allows_tile(deck_index: usize, p: Vec2) -> bool {
    let profile = deck_profile(deck_index);
    if p.y < profile.y_aft || p.y > profile.y_fwd {
        return false;
    }

    let fwd_span = (SHIP_LENGTH_M * 0.5 - profile.y_fwd).max(1.0);
    let aft_span = (profile.y_aft + SHIP_LENGTH_M * 0.5).max(1.0);
    let fwd_t = ((p.y - profile.y_fwd) / fwd_span).clamp(0.0, 1.0);
    let aft_t = ((profile.y_aft - p.y) / aft_span).clamp(0.0, 1.0);
    let taper = 1.0 - profile.bow_taper * fwd_t * fwd_t - profile.stern_taper * aft_t * aft_t;
    let beam_limit = SHIP_BEAM_M * 0.5 * profile.half_beam_scale * taper.max(0.2);
    if p.x.abs() > beam_limit {
        return false;
    }

    if profile.courtyard_half_width > 0.0
        && p.y > profile.courtyard_y_aft
        && p.y < profile.courtyard_y_fwd
        && p.x.abs() < profile.courtyard_half_width
    {
        return false;
    }

    // Upper leisure decks: emulate split stern terraces and side tapering.
    if deck_index >= 17 && p.y < -SHIP_LENGTH_M * 0.20 && p.x.abs() < SHIP_BEAM_M * 0.14 {
        return false;
    }
    if deck_index >= 18 && p.y > SHIP_LENGTH_M * 0.15 && p.x.abs() > SHIP_BEAM_M * 0.22 {
        return false;
    }

    true
}

fn fallback_deck_tiles(deck_index: usize, step_m: f32) -> DeckTiles {
    let centers = if deck_index >= FIRST_UPPER_DECK_STYLE_INDEX {
        deck_tile_centers_upper(step_m)
    } else {
        deck_tile_centers(step_m)
    };
    let occupied = centers
        .iter()
        .map(|c| ((c.x / step_m).round() as i32, (c.y / step_m).round() as i32))
        .collect::<HashSet<_>>();
    DeckTiles { centers, occupied }
}

fn deck_layouts(step_m: f32) -> Vec<DeckTiles> {
    let mut out = Vec::with_capacity(NUM_DECKS);
    for deck_i in 0..NUM_DECKS {
        let base = fallback_deck_tiles(deck_i, step_m);
        let centers = base
            .centers
            .into_iter()
            .filter(|p| profile_allows_tile(deck_i, *p))
            .collect::<Vec<_>>();
        let occupied = centers
            .iter()
            .map(|c| ((c.x / step_m).round() as i32, (c.y / step_m).round() as i32))
            .collect::<HashSet<_>>();
        out.push(DeckTiles { centers, occupied });
    }
    out
}

fn is_perimeter_cell(cell: (i32, i32), occupied: &HashSet<(i32, i32)>) -> bool {
    for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
        if !occupied.contains(&(cell.0 + dx, cell.1 + dy)) {
            return true;
        }
    }
    false
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

    let layouts = deck_layouts(TILE_CELL_M);
    commands.insert_resource(DeckLayouts(layouts.clone()));

    let edge_deck = Color::srgb(0.38, 0.3, 0.24);
    let window_color = Color::srgb(0.42, 0.62, 0.9);
    let outer_cabin = Color::srgb(0.95, 0.82, 0.35);
    let inner_cabin = Color::srgb(0.92, 0.55, 0.72);
    let public_deck = Color::srgb(0.78, 0.86, 0.92);

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
                Transform::from_xyz(0.0, 0.0, deck_z),
                Visibility::default(),
                DeckLayer(deck_i),
            ))
            .with_children(|deck| {
                let layout = &layouts[deck_i];

                for c in &layout.centers {
                    let cell = (
                        (c.x / TILE_CELL_M).round() as i32,
                        (c.y / TILE_CELL_M).round() as i32,
                    );
                    let edge = is_perimeter_cell(cell, &layout.occupied);
                    let mesh = if edge {
                        if window_strip_zone(*c) {
                            &mesh_window
                        } else {
                            &mesh_hull
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

                    deck.spawn((
                        Mesh3d((*mesh).clone()),
                        MeshMaterial3d(clip_handle.clone()),
                        Transform::from_xyz(c.x, c.y, DECK_SLAB_THICKNESS_M * 0.5),
                    ));
                }
            });
    }

    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            left: Val::Px(10.0),
            ..default()
        },
        Text::new(""),
        TextFont {
            font_size: 22.0,
            ..default()
        },
        TextColor(Color::WHITE),
        UiTargetCamera(ui_camera),
        DeckLabel,
    ));
}

fn spawn_sim_npcs(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    layouts: Res<DeckLayouts>,
    mut rng: ResMut<SimRng>,
) {
    let deck_five_z = SIM_DECK_INDEX as f32 * DECK_FLOOR_SPACING_M + DECK_SLAB_THICKNESS_M;
    let walk_points: Vec<Vec3> = layouts.0[SIM_DECK_INDEX]
        .centers
        .clone()
        .into_iter()
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
            Transform::from_translation(spawn_point),
            SimNpc {
                speed_m_s: SIM_NPC_SPEED_M_S,
            },
            WanderState {
                target: target_point,
            },
            Name::new(format!("SimNpc{}", idx + 1)),
        ));
    }

    commands.insert_resource(DeckFiveWalkPoints(walk_points));
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
    m.clip_data = Vec4::new(cut_plane_world_z(current.0), 0.0, 0.0, 0.0);
}

/// Slide the orbit target's Z to the middle of the focused deck so deck switches
/// visibly re-frame the cross-section instead of just shifting the cut plane by 3 m
/// at the top of a 60 m stack (imperceptible from default zoom).
fn focus_camera_on_current_deck(current: Res<CurrentDeck>, mut rig: ResMut<CameraRig>) {
    if !current.is_changed() {
        return;
    }
    rig.target.z = focused_deck_target_z(current.0);
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
    mut query: Query<&mut Text, With<DeckLabel>>,
) {
    if !current_deck.is_changed() && !rig.is_changed() {
        return;
    }
    for mut text in &mut query {
        text.0 = format!(
            "Deck {}/{}: {} | hull {:.0} m × {:.0} m\nQ/E: orbit | WASD: pan | Z/X: zoom | RMB: orbit | MMB: pan | wheel: zoom | PgUp/PgDn: deck",
            current_deck.0 + 1,
            NUM_DECKS,
            DECK_NAMES[current_deck.0],
            SHIP_LENGTH_M,
            SHIP_BEAM_M,
        );
    }
}
