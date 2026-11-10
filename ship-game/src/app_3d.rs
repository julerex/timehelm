//! Read-only 3D view of the persistent ship.
//!
//! Camera actions live in a leafwing action map. Press K for the keymap.
//! Interaction actions are registered and left unbound.

use crate::cabin_marks::{
    bulkhead_mesh, corridor_runner_mesh, deck_fitting_mesh, door_frame_mesh, door_leaf_mesh,
    door_placements, opening_width, PARTITION_HEIGHT_M,
};
use crate::deck_geometry::{
    extruded_polygon_deck_mesh, procedural_deck_plan_texture_image, ship_bulwark_mesh,
    ship_hull_plate,
};
use crate::shader_embed::ShipShaderEmbedPlugin;
use crate::shared::{asset_plugin, primary_window};
use crate::venue_props::venue_mesh;
use crate::viewer_net::{drain_inbox, start_inbox, LiveWorld};
use bevy::camera::ScalingMode;
use bevy::input::mouse::{MouseMotion, MouseScrollUnit, MouseWheel};
use bevy::mesh::{Indices, VertexAttributeValues};
use bevy::pbr::{Material, MaterialPlugin, MeshMaterial3d};
use bevy::prelude::*;
use bevy::render::render_resource::AsBindGroup;
use bevy::shader::ShaderRef;
use leafwing_input_manager::prelude::*;
use sim::{
    contents_label, deck_hull, format_unix_utc, point_in_polygon, segment_crosses_span, CrewView,
    EdgeSpan, RoomKind, RoomView, WorldSnapshot, DECK_COUNT, DECK_HEIGHT_M, DECK_NAMES,
    SHIP_BEAM_M, SHIP_LENGTH_M,
};

/// Keyboard pan (metres per second). Slowed from 520 so a hold stays steerable across the hull.
const CAMERA_PAN_SPEED_M_S: f32 = 100.0;
const CAMERA_VERTICAL_SPEED_M_S: f32 = 260.0;
const CAMERA_YAW_SPEED_RAD_S: f32 = 1.75;
const CAMERA_DOLLY_SPEED_M_S: f32 = 520.0;
const CAMERA_SCROLL_ZOOM_FACTOR: f32 = 0.12;
const CAMERA_MOUSE_ORBIT_SENS: f32 = 0.005;
const CAMERA_MOUSE_PAN_SENS: f32 = 0.0022;
const CAM_DEFAULT_DISTANCE_M: f32 = 44.0;
const CAM_MIN_DISTANCE_M: f32 = 5.0;
const CAM_MAX_DISTANCE_M: f32 = 6200.0;
const CAM_PITCH_MIN: f32 = 0.15;
const CAM_PITCH_MAX: f32 = 1.42;
const FLOOR_THICKNESS_M: f32 = 0.22;
const CREW_SCALE: f32 = 0.55;
const CREW_MODELS: [&str; 18] = [
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
/// Diffuse gain in [`ShipClipMaterial::sun`]. Tuned so a sunlit deck stays near the old flat albedo.
const SUN_DIFFUSE_GAIN: f32 = 1.15;
/// Specular gain in [`ShipClipMaterial::eye`], applied only on near-vertical faces.
const SUN_SPECULAR_GAIN: f32 = 0.16;

/// Controls a viewer can perform. [`ViewerAction::Interact`] stays unbound.
#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Debug, Hash, Reflect)]
enum ViewerAction {
    #[actionlike(DualAxis)]
    Pan,
    #[actionlike(Axis)]
    OrbitYaw,
    #[actionlike(Axis)]
    Lift,
    #[actionlike(Axis)]
    Dolly,
    #[actionlike(Button)]
    DeckUp,
    #[actionlike(Button)]
    DeckDown,
    #[actionlike(Button)]
    Keymap,
    #[actionlike(Button)]
    Plan,
    #[actionlike(Button)]
    OrbitDrag,
    #[actionlike(Button)]
    PanDrag,
    #[actionlike(Button)]
    Interact,
}

struct Binding {
    action: &'static str,
    keys: &'static str,
    available: bool,
}

const BINDINGS: &[Binding] = &[
    Binding {
        action: "Pan",
        keys: "W Q S D",
        available: true,
    },
    Binding {
        action: "Orbit",
        keys: "A / E, or right-drag",
        available: true,
    },
    Binding {
        action: "Lift",
        keys: "R / F",
        available: true,
    },
    Binding {
        action: "Dolly",
        keys: "Z / X, or scroll wheel",
        available: true,
    },
    Binding {
        action: "Pan (mouse)",
        keys: "Middle-drag",
        available: true,
    },
    Binding {
        action: "Deck",
        keys: "Page Up / Page Down",
        available: true,
    },
    Binding {
        action: "Keymap",
        keys: "K",
        available: true,
    },
    Binding {
        action: "Plan",
        keys: "P",
        available: true,
    },
    Binding {
        action: "Interact",
        keys: "Not available to viewers",
        available: false,
    },
];

#[derive(Component)]
struct GameCamera3d;

#[derive(Component)]
struct StatusText;

#[derive(Component)]
struct HoverText;

#[derive(Component)]
struct KeymapPanel;

#[derive(Component)]
struct CrewBody {
    id: u32,
    from: Vec3,
    to: Vec3,
    started: f32,
    deck: u8,
}

#[derive(Component)]
struct DoorLeaf {
    deck: u8,
    yaw: f32,
    swing: f32,
    span: EdgeSpan,
    openness: f32,
}

#[derive(Resource)]
struct CameraRig {
    target: Vec3,
    yaw: f32,
    pitch: f32,
    distance: f32,
}

#[derive(Resource)]
struct CurrentDeck(usize);

#[derive(Resource, Default)]
struct KeymapOpen(bool);

#[derive(Resource)]
struct PlanView {
    enabled: bool,
    scale: f32,
}

#[derive(Resource, Default)]
struct ShipSpawned(bool);

#[derive(Resource, Clone)]
struct SharedClipMaterial(Handle<ShipClipMaterial>);

#[derive(Asset, TypePath, AsBindGroup, Clone)]
struct ShipClipMaterial {
    /// `.x` = world Z of the deck cut (m). `.y` = alpha for fragments above the cut.
    #[uniform(0)]
    clip_data: Vec4,
    /// `.xyz` = world direction toward the sun. `.w` = [`SUN_DIFFUSE_GAIN`].
    #[uniform(0)]
    sun: Vec4,
    /// `.xyz` = camera position (m). `.w` = [`SUN_SPECULAR_GAIN`].
    #[uniform(0)]
    eye: Vec4,
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
        .add_plugins(InputManagerPlugin::<ViewerAction>::default())
        .insert_resource(start_inbox())
        .init_resource::<LiveWorld>()
        .insert_resource(CurrentDeck(8))
        .insert_resource(CameraRig::default_for_deck(8))
        .insert_resource(KeymapOpen(false))
        .insert_resource(PlanView {
            enabled: false,
            scale: 1.0,
        })
        .insert_resource(ShipSpawned(false))
        .insert_resource(ClearColor(Color::srgb(0.55, 0.72, 0.84)))
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                drain_inbox,
                spawn_ship_once,
                sync_crew,
                animate_crew,
                animate_doors,
                update_hud,
                hover_room,
            )
                .chain(),
        )
        .add_systems(
            Update,
            (camera_controls, apply_camera, deck_and_keymap, sync_clip).chain(),
        )
        .run();
}

impl CameraRig {
    fn default_for_deck(deck: usize) -> Self {
        // Starboard of Central Park, so the cabin row, the alley, and the trees share the frame.
        let dir = Vec3::new(-0.42, -0.58, 0.70).normalize();
        Self {
            target: Vec3::new(150.0, 18.0, (deck as f32 + 0.45) * DECK_HEIGHT_M),
            yaw: dir.y.atan2(dir.x),
            pitch: dir.z.asin(),
            distance: CAM_DEFAULT_DISTANCE_M,
        }
    }
}

fn setup(
    mut commands: Commands,
    mut materials: ResMut<Assets<ShipClipMaterial>>,
    mut images: ResMut<Assets<Image>>,
    deck: Res<CurrentDeck>,
) {
    let deck_pattern = images.add(procedural_deck_plan_texture_image());
    let sun_rotation = sun_light_rotation();
    let clip_handle = materials.add(ShipClipMaterial {
        clip_data: Vec4::new(cut_z(deck.0), 0.0, 0.0, 0.0),
        sun: sun_uniform(sun_rotation),
        eye: eye_uniform(Vec3::ZERO),
        deck_pattern,
    });
    commands.insert_resource(SharedClipMaterial(clip_handle));

    commands.spawn((
        Camera3d::default(),
        Camera {
            order: 0,
            ..default()
        },
        Transform::default(),
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
        ))
        .id();

    commands.insert_resource(GlobalAmbientLight {
        color: Color::WHITE,
        brightness: 280.0,
        affects_lightmapped_meshes: true,
    });
    commands.spawn((
        DirectionalLight {
            illuminance: 14000.0,
            ..default()
        },
        Transform::from_rotation(sun_rotation),
    ));

    commands.spawn(viewer_input_map());
    spawn_hud(&mut commands, ui_camera);
}

fn pan_binding() -> VirtualDPad {
    // Q strafes, which is what A used to do. A orbits in Q's old direction.
    VirtualDPad::new(KeyCode::KeyW, KeyCode::KeyS, KeyCode::KeyQ, KeyCode::KeyD)
}

fn orbit_binding() -> VirtualAxis {
    VirtualAxis::new(KeyCode::KeyA, KeyCode::KeyE)
}

fn viewer_input_map() -> InputMap<ViewerAction> {
    InputMap::default()
        .with_dual_axis(ViewerAction::Pan, pan_binding())
        .with_axis(ViewerAction::OrbitYaw, orbit_binding())
        .with_axis(
            ViewerAction::Lift,
            VirtualAxis::new(KeyCode::KeyF, KeyCode::KeyR),
        )
        .with_axis(
            ViewerAction::Dolly,
            VirtualAxis::new(KeyCode::KeyZ, KeyCode::KeyX),
        )
        .with(ViewerAction::DeckUp, KeyCode::PageUp)
        .with(ViewerAction::DeckDown, KeyCode::PageDown)
        .with(ViewerAction::Keymap, KeyCode::KeyK)
        .with(ViewerAction::Plan, KeyCode::KeyP)
        .with(ViewerAction::OrbitDrag, MouseButton::Right)
        .with(ViewerAction::PanDrag, MouseButton::Middle)
}

fn spawn_hud(commands: &mut Commands, ui_camera: Entity) {
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                position_type: PositionType::Absolute,
                top: Val::Px(0.0),
                left: Val::Px(0.0),
                flex_direction: FlexDirection::Column,
                padding: UiRect::axes(Val::Px(12.0), Val::Px(6.0)),
                row_gap: Val::Px(2.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.05, 0.08, 0.12, 0.82)),
            UiTargetCamera(ui_camera),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("Time Helm — connecting"),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                StatusText,
            ));
            parent.spawn((
                Text::new("Hover a room"),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::srgb(0.82, 0.88, 0.92)),
                HoverText,
            ));
        });

    let mut lines = String::from("Keymap\n");
    for binding in BINDINGS {
        let state = if binding.available { "" } else { " (locked)" };
        lines.push_str(&format!("{} — {}{state}\n", binding.action, binding.keys));
    }
    lines.push_str(
        "\nViewers cannot change the ship. Interaction stays in the action map for later.",
    );

    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(58.0),
            left: Val::Px(12.0),
            padding: UiRect::all(Val::Px(12.0)),
            max_width: Val::Px(420.0),
            display: Display::None,
            ..default()
        },
        BackgroundColor(Color::srgba(0.06, 0.09, 0.14, 0.92)),
        UiTargetCamera(ui_camera),
        KeymapPanel,
        children![(
            Text::new(lines),
            TextFont {
                font_size: 15.0,
                ..default()
            },
            TextColor(Color::srgb(0.92, 0.95, 0.98)),
        )],
    ));
}

fn spawn_ship_once(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    clip: Option<Res<SharedClipMaterial>>,
    live: Res<LiveWorld>,
    mut spawned: ResMut<ShipSpawned>,
    asset_server: Res<AssetServer>,
    time: Res<Time>,
) {
    if spawned.0 || !live.is_changed() {
        return;
    }
    let Some(clip) = clip else {
        return;
    };
    let Some(snapshot) = live.snapshot.as_ref() else {
        return;
    };
    for deck in 0..DECK_COUNT {
        let rooms: Vec<&RoomView> = snapshot
            .rooms
            .iter()
            .filter(|room| room.deck as usize == deck)
            .collect();
        if rooms.is_empty() {
            continue;
        }
        let deck_z = deck as f32 * DECK_HEIGHT_M;
        let floor_top = deck_z + FLOOR_THICKNESS_M;
        spawn_hull(&mut commands, &mut meshes, &clip.0, deck, deck_z);
        if let Some(mesh) = deck_floor_mesh(&rooms) {
            commands.spawn((
                Mesh3d(meshes.add(mesh)),
                MeshMaterial3d(clip.0.clone()),
                Transform::from_xyz(0.0, 0.0, deck_z + FLOOR_THICKNESS_M * 0.5),
            ));
        }
        if let Some(mesh) = deck_wall_mesh(snapshot, deck as u8) {
            commands.spawn((
                Mesh3d(meshes.add(mesh)),
                MeshMaterial3d(clip.0.clone()),
                Transform::from_xyz(0.0, 0.0, floor_top),
            ));
        }
        for (mesh, z) in [
            bulkhead_mesh(&rooms).map(|mesh| (mesh, floor_top)),
            door_frame_mesh(&rooms).map(|mesh| (mesh, floor_top)),
            corridor_runner_mesh(&rooms).map(|mesh| (mesh, floor_top)),
            deck_fitting_mesh(&rooms).map(|mesh| (mesh, floor_top)),
            venue_mesh(deck, &rooms).map(|mesh| (mesh, floor_top)),
        ]
        .into_iter()
        .flatten()
        {
            commands.spawn((
                Mesh3d(meshes.add(mesh)),
                MeshMaterial3d(clip.0.clone()),
                Transform::from_xyz(0.0, 0.0, z),
            ));
        }
        let placements = door_placements(&rooms);
        if !placements.is_empty() {
            let leaf = meshes.add(door_leaf_mesh());
            for door in placements {
                commands.spawn((
                    Mesh3d(leaf.clone()),
                    MeshMaterial3d(clip.0.clone()),
                    Transform::from_xyz(door.hinge[0], door.hinge[1], floor_top)
                        .with_rotation(Quat::from_rotation_z(door.yaw))
                        .with_scale(Vec3::new(door.width, 1.0, 1.0)),
                    DoorLeaf {
                        deck: deck as u8,
                        yaw: door.yaw,
                        swing: door.swing,
                        span: door.span,
                        openness: 0.0,
                    },
                ));
            }
        }
    }
    let now = time.elapsed_secs();
    for crew in &snapshot.crew {
        let pos = crew_pos(crew);
        let model = CREW_MODELS[crew.id as usize % CREW_MODELS.len()];
        commands.spawn((
            SceneRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset(model))),
            Transform::from_translation(pos).with_scale(Vec3::splat(CREW_SCALE)),
            CrewBody {
                id: crew.id,
                from: pos,
                to: pos,
                started: now,
                deck: crew.deck,
            },
            Name::new(crew.name.clone()),
        ));
    }
    spawned.0 = true;
}

fn sync_crew(time: Res<Time>, live: Res<LiveWorld>, mut crew: Query<&mut CrewBody>) {
    if !live.is_changed() {
        return;
    }
    let Some(snapshot) = live.snapshot.as_ref() else {
        return;
    };
    let now = time.elapsed_secs();
    for mut body in &mut crew {
        let Some(view) = snapshot.crew.iter().find(|crew| crew.id == body.id) else {
            continue;
        };
        let next = crew_pos(view);
        if view.deck != body.deck {
            body.from = next;
        } else {
            body.from = sample_crew(&body, now);
        }
        body.to = next;
        body.deck = view.deck;
        body.started = now;
    }
}

fn animate_doors(
    time: Res<Time>,
    crew: Query<&CrewBody>,
    mut doors: Query<(&mut DoorLeaf, &mut Transform)>,
) {
    let now = time.elapsed_secs();
    let walking: Vec<Passing> = crew
        .iter()
        .map(|body| {
            let pos = sample_crew(body, now);
            Passing {
                deck: body.deck,
                from: [body.from.x, body.from.y],
                to: [body.to.x, body.to.y],
                pos: [pos.x, pos.y],
            }
        })
        .collect();
    let blend = 1.0 - (-5.0 * time.delta_secs()).exp();
    for (mut door, mut transform) in &mut doors {
        let open = walking.iter().any(|crew| {
            crew.deck == door.deck
                && (near_span(crew.pos, door.span) < 1.35
                    || (segment_crosses_span(crew.from, crew.to, door.span)
                        && near_span(crew.pos, door.span) < 5.5))
        });
        let target = if open { 1.0 } else { 0.0 };
        door.openness += (target - door.openness) * blend;
        if (door.openness - target).abs() < 0.001 {
            door.openness = target;
        }
        transform.rotation = Quat::from_rotation_z(door.yaw + door.openness * door.swing);
    }
}

struct Passing {
    deck: u8,
    from: [f32; 2],
    to: [f32; 2],
    pos: [f32; 2],
}

fn near_span(point: [f32; 2], span: EdgeSpan) -> f32 {
    let abx = span.end[0] - span.start[0];
    let aby = span.end[1] - span.start[1];
    let len2 = abx * abx + aby * aby;
    if len2 < 1e-6 {
        return (point[0] - span.start[0]).hypot(point[1] - span.start[1]);
    }
    let t = (((point[0] - span.start[0]) * abx + (point[1] - span.start[1]) * aby) / len2)
        .clamp(0.0, 1.0);
    let x = span.start[0] + abx * t;
    let y = span.start[1] + aby * t;
    (point[0] - x).hypot(point[1] - y)
}

fn animate_crew(
    time: Res<Time>,
    deck: Res<CurrentDeck>,
    mut crew: Query<(&CrewBody, &mut Transform, &mut Visibility)>,
) {
    let now = time.elapsed_secs();
    let cut = cut_z(deck.0);
    for (body, mut transform, mut visibility) in &mut crew {
        let pos = sample_crew(body, now);
        let flat = Vec3::new(body.to.x - body.from.x, body.to.y - body.from.y, 0.0);
        transform.translation = pos;
        if flat.length_squared() > 0.05 {
            transform.look_to(flat.normalize(), Vec3::Z);
        }
        transform.scale = Vec3::splat(CREW_SCALE);
        *visibility = if pos.z <= cut + 0.2 {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }
}

fn camera_controls(
    time: Res<Time>,
    actions: Query<&ActionState<ViewerAction>>,
    mut scroll: MessageReader<MouseWheel>,
    mut motion: MessageReader<MouseMotion>,
    mut rig: ResMut<CameraRig>,
    mut plan: ResMut<PlanView>,
) {
    let Ok(action) = actions.single() else {
        return;
    };
    if action.just_pressed(&ViewerAction::Plan) {
        plan.enabled = !plan.enabled;
    }
    let dt = time.delta_secs();
    let pan = action.axis_pair(&ViewerAction::Pan);
    let (right, forward) = if plan.enabled {
        (Vec3::X, Vec3::Y)
    } else {
        pan_basis(&rig)
    };
    rig.target += keyboard_pan_delta(pan, dt, right, forward);

    let mut scroll_y = 0.0;
    for event in scroll.read() {
        scroll_y += match event.unit {
            MouseScrollUnit::Line => event.y,
            MouseScrollUnit::Pixel => event.y * 0.015,
        };
    }
    let zoom_factor = 1.0 - scroll_y * CAMERA_SCROLL_ZOOM_FACTOR;

    if plan.enabled {
        let dolly = action.clamped_value(&ViewerAction::Dolly);
        plan.scale = (plan.scale * zoom_factor * (1.0 + dolly * dt)).clamp(0.18, 4.5);
        for _ in motion.read() {}
    } else {
        rig.yaw += action.clamped_value(&ViewerAction::OrbitYaw) * CAMERA_YAW_SPEED_RAD_S * dt;
        rig.target.z += action.clamped_value(&ViewerAction::Lift) * CAMERA_VERTICAL_SPEED_M_S * dt;
        rig.distance = (rig.distance
            + action.clamped_value(&ViewerAction::Dolly) * CAMERA_DOLLY_SPEED_M_S * dt)
            .clamp(CAM_MIN_DISTANCE_M, CAM_MAX_DISTANCE_M);
        rig.distance = (rig.distance * zoom_factor).clamp(CAM_MIN_DISTANCE_M, CAM_MAX_DISTANCE_M);

        let orbiting = action.pressed(&ViewerAction::OrbitDrag);
        let panning = action.pressed(&ViewerAction::PanDrag);
        if orbiting {
            for event in motion.read() {
                rig.yaw -= event.delta.x * CAMERA_MOUSE_ORBIT_SENS;
                rig.pitch = (rig.pitch - event.delta.y * CAMERA_MOUSE_ORBIT_SENS)
                    .clamp(CAM_PITCH_MIN, CAM_PITCH_MAX);
            }
        } else if panning {
            let (right, forward) = pan_basis(&rig);
            let scale = rig.distance * CAMERA_MOUSE_PAN_SENS;
            for event in motion.read() {
                rig.target += right * -event.delta.x * scale + forward * event.delta.y * scale;
            }
        } else {
            for _ in motion.read() {}
        }
    }
}

fn apply_camera(
    rig: Res<CameraRig>,
    plan: Res<PlanView>,
    deck: Res<CurrentDeck>,
    mut cameras: Query<(&mut Transform, &mut Projection), With<GameCamera3d>>,
) {
    let deck_z = deck.0 as f32 * DECK_HEIGHT_M;
    for (mut transform, mut projection) in &mut cameras {
        if plan.enabled {
            // Fixed above the sea. Page Up / Page Down change the cut, not this height.
            // Orthographic scale does not depend on distance, so the plan stays framed.
            let eye_z = plan_camera_z();
            *projection = Projection::Orthographic(OrthographicProjection {
                scaling_mode: ScalingMode::FixedVertical {
                    viewport_height: SHIP_BEAM_M * 1.25,
                },
                scale: plan.scale,
                near: 0.05,
                far: eye_z + 16.0,
                ..OrthographicProjection::default_3d()
            });
            let eye = Vec3::new(rig.target.x, rig.target.y, eye_z);
            *transform = Transform::from_translation(eye)
                .looking_at(Vec3::new(rig.target.x, rig.target.y, deck_z), Vec3::Y);
        } else if !projection.is_perspective() {
            *projection = Projection::Perspective(PerspectiveProjection::default());
            *transform = rig_transform(&rig);
        } else {
            *transform = rig_transform(&rig);
        }
    }
}

fn deck_and_keymap(
    actions: Query<&ActionState<ViewerAction>>,
    mut deck: ResMut<CurrentDeck>,
    mut keymap: ResMut<KeymapOpen>,
    mut panels: Query<&mut Node, With<KeymapPanel>>,
) {
    let Ok(action) = actions.single() else {
        return;
    };
    // The cut moves with the deck. The camera target stays where the player left it.
    if action.just_pressed(&ViewerAction::DeckUp) {
        if let Some(next) = stepped_deck(deck.0, true) {
            deck.0 = next;
        }
    }
    if action.just_pressed(&ViewerAction::DeckDown) {
        if let Some(next) = stepped_deck(deck.0, false) {
            deck.0 = next;
        }
    }
    if action.just_pressed(&ViewerAction::Keymap) {
        keymap.0 = !keymap.0;
    }
    let display = if keymap.0 {
        Display::Flex
    } else {
        Display::None
    };
    for mut node in &mut panels {
        node.display = display;
    }
}

fn sync_clip(
    deck: Res<CurrentDeck>,
    shared: Option<Res<SharedClipMaterial>>,
    mut materials: ResMut<Assets<ShipClipMaterial>>,
    lights: Query<&Transform, With<DirectionalLight>>,
    cameras: Query<&Transform, With<GameCamera3d>>,
) {
    let Some(shared) = shared else {
        return;
    };
    let Some(material) = materials.get(&shared.0) else {
        return;
    };
    let mut clip_data = material.clip_data;
    clip_data.x = cut_z(deck.0);
    let sun = if let Ok(light) = lights.single() {
        sun_uniform(light.rotation)
    } else {
        material.sun
    };
    let eye = if let Ok(camera) = cameras.single() {
        eye_uniform(camera.translation)
    } else {
        material.eye
    };
    if clip_data == material.clip_data && sun == material.sun && eye == material.eye {
        return;
    }
    let Some(material) = materials.get_mut(&shared.0) else {
        return;
    };
    material.clip_data = clip_data;
    material.sun = sun;
    material.eye = eye;
}

fn sun_light_rotation() -> Quat {
    Quat::from_euler(EulerRot::XYZ, -0.9, 0.4, 0.0)
}

/// Directional lights emit along local -Z. Daylight comes from local +Z.
fn to_sun_direction(rotation: Quat) -> Vec3 {
    Transform::from_rotation(rotation).back().as_vec3()
}

fn sun_uniform(rotation: Quat) -> Vec4 {
    let to_sun = to_sun_direction(rotation);
    Vec4::new(to_sun.x, to_sun.y, to_sun.z, SUN_DIFFUSE_GAIN)
}

fn eye_uniform(position: Vec3) -> Vec4 {
    Vec4::new(position.x, position.y, position.z, SUN_SPECULAR_GAIN)
}

fn update_hud(
    live: Res<LiveWorld>,
    deck: Res<CurrentDeck>,
    plan: Res<PlanView>,
    mut status: Query<&mut Text, With<StatusText>>,
) {
    let line = status_line(&live, deck.0, plan.enabled);
    for mut text in &mut status {
        if text.0 != line {
            text.0 = line.clone();
        }
    }
}

fn hover_room(
    live: Res<LiveWorld>,
    deck: Res<CurrentDeck>,
    windows: Query<&Window>,
    cameras: Query<(&Camera, &GlobalTransform), With<GameCamera3d>>,
    mut hover: Query<&mut Text, With<HoverText>>,
) {
    let line = hover_line(&live, deck.0, &windows, &cameras);
    for mut text in &mut hover {
        if text.0 != line {
            text.0 = line.clone();
        }
    }
}

fn status_line(live: &LiveWorld, deck: usize, plan: bool) -> String {
    let deck_name = DECK_NAMES.get(deck).copied().unwrap_or("Deck");
    let clock = wall_utc();
    let view = if plan { "plan" } else { "orbit" };
    if let Some(reason) = live.unavailable.as_ref() {
        return format!("{clock}  ·  world unavailable  ·  {reason}");
    }
    let Some(snapshot) = live.snapshot.as_ref() else {
        let state = if live.ever_connected {
            "reconnecting"
        } else {
            "connecting"
        };
        return format!("{clock}  ·  {state}  ·  {deck_name}  ·  {view}  ·  P plan  ·  K keymap");
    };
    let totals = &snapshot.totals;
    format!(
        "{clock}  ·  {deck_name}  ·  {view}  ·  {} crew  ·  food {:.0} d  ·  water {:.0} d  ·  power {:.2}/{:.2} kWh/min  ·  P plan",
        totals.crew,
        totals.food_days,
        totals.water_days,
        snapshot.power.supply_kwh,
        snapshot.power.demand_kwh
    )
}

fn hover_line(
    live: &LiveWorld,
    deck: usize,
    windows: &Query<&Window>,
    cameras: &Query<(&Camera, &GlobalTransform), With<GameCamera3d>>,
) -> String {
    let Some(snapshot) = live.snapshot.as_ref() else {
        return "Hover a room".to_string();
    };
    let Some(hit) = cursor_on_deck(deck, windows, cameras) else {
        return format!("Deck {} — hover a room", deck_label(deck));
    };
    let Some(room) = snapshot
        .rooms
        .iter()
        .find(|room| room.deck as usize == deck && point_in_room(&room.polygon, hit.x, hit.y))
    else {
        return format!(
            "{} — open deck ({:.0}, {:.0}) m",
            deck_label(deck),
            hit.x,
            hit.y
        );
    };
    let stocks: Vec<String> = snapshot
        .stocks
        .iter()
        .filter(|stock| stock.room_id == room.id && stock.qty > 0.05)
        .take(4)
        .map(|stock| format!("{} {:.0}", stock.good.label(), stock.qty))
        .collect();
    let people: Vec<&str> = snapshot
        .crew
        .iter()
        .filter(|crew| crew.room_id == room.id)
        .take(4)
        .map(|crew| crew.name.as_str())
        .collect();
    let stock_text = if stocks.is_empty() {
        "empty".to_string()
    } else {
        stocks.join(", ")
    };
    let people_text = if people.is_empty() {
        "nobody here".to_string()
    } else {
        people.join(", ")
    };
    let contents = contents_label(room.kind);
    let kind = if contents.is_empty() {
        room.kind.label().to_string()
    } else {
        format!("{} · {contents}", room.kind.label())
    };
    format!("{} — {kind} — {stock_text} — {people_text}", room.name)
}

fn cursor_on_deck(
    deck: usize,
    windows: &Query<&Window>,
    cameras: &Query<(&Camera, &GlobalTransform), With<GameCamera3d>>,
) -> Option<Vec2> {
    let window = windows.iter().next()?;
    let cursor = window.cursor_position()?;
    let (camera, transform) = cameras.iter().next()?;
    let ray = camera.viewport_to_world(transform, cursor).ok()?;
    let plane_z = deck as f32 * DECK_HEIGHT_M + FLOOR_THICKNESS_M;
    let direction = ray.direction.as_vec3();
    if direction.z.abs() < 1e-5 {
        return None;
    }
    let t = (plane_z - ray.origin.z) / direction.z;
    if t < 0.0 {
        return None;
    }
    let hit = ray.origin + direction * t;
    Some(Vec2::new(hit.x, hit.y))
}

fn point_in_room(polygon: &[[f32; 2]], x: f32, y: f32) -> bool {
    let hull: Vec<Vec2> = polygon
        .iter()
        .map(|point| Vec2::new(point[0], point[1]))
        .collect();
    point_in_polygon(Vec2::new(x, y), &hull)
}

fn spawn_hull(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    clip: &Handle<ShipClipMaterial>,
    deck: usize,
    deck_z: f32,
) {
    let hull = deck_hull(deck);
    let outlines: Vec<Vec<[f32; 2]>> = hull
        .outlines
        .iter()
        .map(|ring| ring.iter().map(|point| [point.x, point.y]).collect())
        .collect();
    let holes: Vec<Vec<[f32; 2]>> = hull
        .holes
        .iter()
        .map(|ring| ring.iter().map(|point| [point.x, point.y]).collect())
        .collect();
    let floor_top = deck_z + FLOOR_THICKNESS_M;
    if outlines.len() == 1 {
        spawn_plate(commands, meshes, clip, &outlines[0], &holes, floor_top);
    } else {
        for outline in &outlines {
            spawn_plate(commands, meshes, clip, outline, &[], floor_top);
        }
    }
    spawn_band(
        commands,
        meshes,
        clip,
        ship_bulwark_mesh(&outlines, 0.0, 0.62, 0.8, [0.14, 0.17, 0.22, 1.0]),
        floor_top,
    );
    spawn_band(
        commands,
        meshes,
        clip,
        ship_bulwark_mesh(&outlines, 0.62, 2.45, 0.55, [0.90, 0.92, 0.94, 1.0]),
        floor_top,
    );
    if !holes.is_empty() {
        spawn_band(
            commands,
            meshes,
            clip,
            ship_bulwark_mesh(&holes, 0.0, 1.1, 0.28, [0.68, 0.80, 0.76, 1.0]),
            floor_top,
        );
    }
}

fn spawn_plate(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    clip: &Handle<ShipClipMaterial>,
    outline: &[[f32; 2]],
    holes: &[Vec<[f32; 2]>],
    floor_top: f32,
) {
    if let Some(mesh) = ship_hull_plate(outline, holes) {
        commands.spawn((
            Mesh3d(meshes.add(mesh)),
            MeshMaterial3d(clip.clone()),
            Transform::from_xyz(0.0, 0.0, floor_top - 0.12),
        ));
    }
}

fn spawn_band(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    clip: &Handle<ShipClipMaterial>,
    mesh: Option<Mesh>,
    floor_top: f32,
) {
    if let Some(mesh) = mesh {
        commands.spawn((
            Mesh3d(meshes.add(mesh)),
            MeshMaterial3d(clip.clone()),
            Transform::from_xyz(0.0, 0.0, floor_top),
        ));
    }
}

fn deck_floor_mesh(rooms: &[&RoomView]) -> Option<Mesh> {
    let mut parts = Vec::new();
    for room in rooms {
        let polygon: Vec<Vec2> = room
            .polygon
            .iter()
            .map(|point| Vec2::new(point[0], point[1]))
            .collect();
        let mesh = tint_mesh(
            extruded_polygon_deck_mesh(&polygon, FLOOR_THICKNESS_M, SHIP_LENGTH_M, SHIP_BEAM_M),
            room.kind.color_rgba(),
        );
        parts.push(mesh);
    }
    merge_meshes(parts)
}

fn deck_wall_mesh(snapshot: &WorldSnapshot, deck: u8) -> Option<Mesh> {
    let on_deck: Vec<&RoomView> = snapshot
        .rooms
        .iter()
        .filter(|room| room.deck == deck)
        .collect();
    let rooms: Vec<&RoomView> = on_deck
        .iter()
        .copied()
        .filter(|room| room.kind != RoomKind::Corridor)
        .collect();
    if rooms.is_empty() {
        return None;
    }
    let mut parts = MeshParts::default();
    for room in &rooms {
        let color = room.kind.color_rgba();
        let polygon = &room.polygon;
        for index in 0..polygon.len() {
            let start = polygon[index];
            let end = polygon[(index + 1) % polygon.len()];
            for (a, b) in wall_segments(start, end, &on_deck, room.id) {
                push_wall(&mut parts, a, b, color);
            }
        }
    }
    if parts.indices.is_empty() {
        None
    } else {
        Some(parts.into_mesh())
    }
}

#[derive(Default)]
struct MeshParts {
    positions: Vec<[f32; 3]>,
    normals: Vec<[f32; 3]>,
    uvs: Vec<[f32; 2]>,
    colors: Vec<[f32; 4]>,
    indices: Vec<u32>,
}

impl MeshParts {
    fn into_mesh(self) -> Mesh {
        mesh_from_parts(
            self.positions,
            self.normals,
            self.uvs,
            self.colors,
            self.indices,
        )
    }
}

fn wall_segments(
    start: [f32; 2],
    end: [f32; 2],
    rooms: &[&RoomView],
    self_id: u32,
) -> Vec<([f32; 2], [f32; 2])> {
    let vertical = (start[0] - end[0]).abs() < 0.3;
    let mut overlaps: Vec<(f32, f32)> = Vec::new();
    for room in rooms {
        if room.id == self_id {
            continue;
        }
        let polygon = &room.polygon;
        for index in 0..polygon.len() {
            let other_start = polygon[index];
            let other_end = polygon[(index + 1) % polygon.len()];
            if let Some(span) = shared_span(start, end, other_start, other_end, vertical) {
                overlaps.push(span);
            }
        }
    }
    if overlaps.is_empty() {
        return vec![(start, end)];
    }
    let (lo, hi) = overlaps
        .into_iter()
        .max_by(|left, right| {
            (left.1 - left.0)
                .partial_cmp(&(right.1 - right.0))
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .unwrap_or((0.0, 0.0));
    let gap = opening_width(hi - lo);
    if hi - lo < 1.2 || gap < 0.8 {
        return vec![(start, end)];
    }
    let mid = (lo + hi) * 0.5;
    let gap_lo = mid - gap * 0.5;
    let gap_hi = mid + gap * 0.5;
    let mut parts = Vec::new();
    push_span(&mut parts, start, vertical, lo.min(gap_lo), gap_lo);
    push_span(&mut parts, start, vertical, gap_hi, hi.max(gap_hi));
    if parts.is_empty() {
        vec![(start, end)]
    } else {
        parts
    }
}

fn push_span(
    parts: &mut Vec<([f32; 2], [f32; 2])>,
    start: [f32; 2],
    vertical: bool,
    from: f32,
    to: f32,
) {
    if to - from < 0.4 {
        return;
    }
    if vertical {
        let x = start[0];
        parts.push(([x, from], [x, to]));
    } else {
        let y = start[1];
        parts.push(([from, y], [to, y]));
    }
}

fn shared_span(
    start: [f32; 2],
    end: [f32; 2],
    other_start: [f32; 2],
    other_end: [f32; 2],
    vertical: bool,
) -> Option<(f32, f32)> {
    if vertical {
        let other_vertical = (other_start[0] - other_end[0]).abs() < 0.3;
        if !other_vertical || (start[0] - other_start[0]).abs() > 0.8 {
            return None;
        }
        let a0 = start[1].min(end[1]);
        let a1 = start[1].max(end[1]);
        let b0 = other_start[1].min(other_end[1]);
        let b1 = other_start[1].max(other_end[1]);
        let lo = a0.max(b0);
        let hi = a1.min(b1);
        (hi - lo > 1.0).then_some((lo, hi))
    } else {
        let other_horizontal = (other_start[1] - other_end[1]).abs() < 0.3;
        if !other_horizontal || (start[1] - other_start[1]).abs() > 0.8 {
            return None;
        }
        let a0 = start[0].min(end[0]);
        let a1 = start[0].max(end[0]);
        let b0 = other_start[0].min(other_end[0]);
        let b1 = other_start[0].max(other_end[0]);
        let lo = a0.max(b0);
        let hi = a1.min(b1);
        (hi - lo > 1.0).then_some((lo, hi))
    }
}

fn push_wall(parts: &mut MeshParts, start: [f32; 2], end: [f32; 2], color: [f32; 4]) {
    let dx = end[0] - start[0];
    let dy = end[1] - start[1];
    let length = (dx * dx + dy * dy).sqrt();
    if length < 0.35 {
        return;
    }
    let tx = dx / length;
    let ty = dy / length;
    let nx = -ty;
    let ny = tx;
    let half = 0.09;
    let z1 = PARTITION_HEIGHT_M;
    let at = |along: f32, side: f32, z: f32| {
        [
            start[0] + tx * along + nx * side,
            start[1] + ty * along + ny * side,
            z,
        ]
    };
    // Top, then the two long faces and the two ends. Winding faces outward.
    push_quad(
        parts,
        [
            at(0.0, half, z1),
            at(0.0, -half, z1),
            at(length, -half, z1),
            at(length, half, z1),
        ],
        [0.0, 0.0, 1.0],
        color,
    );
    push_quad(
        parts,
        [
            at(0.0, half, 0.0),
            at(length, half, 0.0),
            at(length, half, z1),
            at(0.0, half, z1),
        ],
        [nx, ny, 0.0],
        color,
    );
    push_quad(
        parts,
        [
            at(length, -half, 0.0),
            at(0.0, -half, 0.0),
            at(0.0, -half, z1),
            at(length, -half, z1),
        ],
        [-nx, -ny, 0.0],
        color,
    );
    push_quad(
        parts,
        [
            at(0.0, -half, 0.0),
            at(0.0, half, 0.0),
            at(0.0, half, z1),
            at(0.0, -half, z1),
        ],
        [-tx, -ty, 0.0],
        color,
    );
    push_quad(
        parts,
        [
            at(length, half, 0.0),
            at(length, -half, 0.0),
            at(length, -half, z1),
            at(length, half, z1),
        ],
        [tx, ty, 0.0],
        color,
    );
}

fn push_quad(parts: &mut MeshParts, quad: [[f32; 3]; 4], normal: [f32; 3], color: [f32; 4]) {
    let base = parts.positions.len() as u32;
    for point in quad {
        parts.positions.push(point);
        parts.normals.push(normal);
        parts.uvs.push([0.0, 0.0]);
        parts.colors.push(color);
    }
    parts
        .indices
        .extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
}

fn tint_mesh(mut mesh: Mesh, color: [f32; 4]) -> Mesh {
    if let Some(VertexAttributeValues::Float32x4(colors)) =
        mesh.attribute_mut(Mesh::ATTRIBUTE_COLOR)
    {
        for slot in colors.iter_mut() {
            *slot = color;
        }
    }
    mesh
}

fn merge_meshes(meshes: Vec<Mesh>) -> Option<Mesh> {
    let mut positions = Vec::<[f32; 3]>::new();
    let mut normals = Vec::<[f32; 3]>::new();
    let mut uvs = Vec::<[f32; 2]>::new();
    let mut colors = Vec::<[f32; 4]>::new();
    let mut indices = Vec::<u32>::new();
    for mesh in meshes {
        let base = positions.len() as u32;
        if let Some(VertexAttributeValues::Float32x3(values)) =
            mesh.attribute(Mesh::ATTRIBUTE_POSITION)
        {
            positions.extend(values);
        }
        if let Some(VertexAttributeValues::Float32x3(values)) =
            mesh.attribute(Mesh::ATTRIBUTE_NORMAL)
        {
            normals.extend(values);
        }
        if let Some(VertexAttributeValues::Float32x2(values)) = mesh.attribute(Mesh::ATTRIBUTE_UV_0)
        {
            uvs.extend(values);
        }
        if let Some(VertexAttributeValues::Float32x4(values)) =
            mesh.attribute(Mesh::ATTRIBUTE_COLOR)
        {
            colors.extend(values);
        }
        if let Some(Indices::U32(values)) = mesh.indices() {
            indices.extend(values.iter().map(|index| index + base));
        }
    }
    if indices.is_empty() {
        None
    } else {
        Some(mesh_from_parts(positions, normals, uvs, colors, indices))
    }
}

fn mesh_from_parts(
    positions: Vec<[f32; 3]>,
    normals: Vec<[f32; 3]>,
    uvs: Vec<[f32; 2]>,
    colors: Vec<[f32; 4]>,
    indices: Vec<u32>,
) -> Mesh {
    Mesh::new(
        bevy::mesh::PrimitiveTopology::TriangleList,
        bevy::asset::RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
    .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, colors)
    .with_inserted_indices(Indices::U32(indices))
}

fn crew_pos(crew: &CrewView) -> Vec3 {
    Vec3::new(
        crew.x,
        crew.y,
        crew.deck as f32 * DECK_HEIGHT_M + FLOOR_THICKNESS_M + 0.05,
    )
}

fn sample_crew(body: &CrewBody, now: f32) -> Vec3 {
    let t = ((now - body.started) / 60.0).clamp(0.0, 1.0);
    body.from.lerp(body.to, t)
}

fn cut_z(deck: usize) -> f32 {
    (deck as f32 + 1.0) * DECK_HEIGHT_M - 0.08
}

fn wall_utc() -> String {
    format_unix_utc(wall_unix())
}

fn wall_unix() -> i64 {
    #[cfg(target_arch = "wasm32")]
    {
        (js_sys::Date::now() / 1000.0) as i64
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|duration| i64::try_from(duration.as_secs()).unwrap_or(0))
            .unwrap_or(0)
    }
}

fn deck_label(deck: usize) -> &'static str {
    DECK_NAMES.get(deck).copied().unwrap_or("Deck")
}

fn rig_transform(rig: &CameraRig) -> Transform {
    let direction = dir_from_yaw_pitch(rig.yaw, rig.pitch);
    Transform::from_translation(rig.target + direction * rig.distance)
        .looking_at(rig.target, Vec3::Z)
}

fn dir_from_yaw_pitch(yaw: f32, pitch: f32) -> Vec3 {
    let cp = pitch.cos();
    Vec3::new(cp * yaw.cos(), cp * yaw.sin(), pitch.sin())
}

fn keyboard_pan_delta(pan: Vec2, dt: f32, right: Vec3, forward: Vec3) -> Vec3 {
    right * pan.x * CAMERA_PAN_SPEED_M_S * dt + forward * pan.y * CAMERA_PAN_SPEED_M_S * dt
}

/// Next deck index, or `None` when the press would move past an end deck.
fn stepped_deck(deck: usize, up: bool) -> Option<usize> {
    if up && deck + 1 < DECK_COUNT {
        Some(deck + 1)
    } else if !up && deck > 0 {
        Some(deck - 1)
    } else {
        None
    }
}

/// Plan camera height above the sea. Independent of the selected deck.
fn plan_camera_z() -> f32 {
    DECK_COUNT as f32 * DECK_HEIGHT_M + 24.0
}

fn pan_basis(rig: &CameraRig) -> (Vec3, Vec3) {
    let direction = dir_from_yaw_pitch(rig.yaw, rig.pitch);
    let forward = Vec3::new(direction.x, direction.y, 0.0);
    if forward.length_squared() < 1e-8 {
        return (Vec3::X, Vec3::Y);
    }
    let forward = forward.normalize();
    let right = Vec3::new(forward.y, -forward.x, 0.0);
    (right, forward)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::input::InputPlugin;
    use bevy::time::TimeUpdateStrategy;
    use std::time::Duration;

    const HELD_KEYS: [KeyCode; 10] = [
        KeyCode::KeyW,
        KeyCode::KeyA,
        KeyCode::KeyS,
        KeyCode::KeyD,
        KeyCode::KeyQ,
        KeyCode::KeyE,
        KeyCode::KeyR,
        KeyCode::KeyF,
        KeyCode::PageUp,
        KeyCode::PageDown,
    ];

    fn camera_app() -> App {
        let mut app = App::new();
        app.add_plugins((
            MinimalPlugins,
            InputPlugin,
            InputManagerPlugin::<ViewerAction>::default(),
        ));
        app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs(1)));
        app.insert_resource(CameraRig::default_for_deck(6));
        app.insert_resource(CurrentDeck(6));
        app.insert_resource(KeymapOpen(false));
        app.insert_resource(PlanView {
            enabled: false,
            scale: 1.0,
        });
        app.world_mut()
            .spawn((viewer_input_map(), ActionState::<ViewerAction>::default()));
        app.add_systems(Update, (camera_controls, deck_and_keymap).chain());
        app
    }

    fn rig(app: &mut App) -> (Vec3, f32) {
        let rig = app.world().resource::<CameraRig>();
        (rig.target, rig.yaw)
    }

    fn hold(app: &mut App, key: KeyCode) {
        for held in HELD_KEYS {
            held.release(app.world_mut());
        }
        app.update();
        key.press(app.world_mut());
        app.update();
    }

    #[test]
    fn keyboard_pan_is_100_metres_per_second_and_stays_level() {
        let delta = keyboard_pan_delta(Vec2::new(1.0, -0.5), 2.0, Vec3::X, Vec3::Y);
        assert_eq!(delta, Vec3::new(200.0, -100.0, 0.0));

        let mut app = camera_app();
        app.update();
        let (start, yaw) = rig(&mut app);
        let (_, forward) = pan_basis(app.world().resource::<CameraRig>());
        hold(&mut app, KeyCode::KeyW);
        let (end, end_yaw) = rig(&mut app);
        let moved = end - start;
        let dt = app.world().resource::<Time>().delta_secs();
        assert!(dt > 0.0);
        assert!((moved.dot(forward) - CAMERA_PAN_SPEED_M_S * dt).abs() < 1e-2);
        assert!(moved.z.abs() < 1e-4);
        assert_eq!(end.z, start.z);
        assert_eq!(end_yaw, yaw);
        assert_eq!(CAMERA_PAN_SPEED_M_S, 100.0);
    }

    #[test]
    fn q_strafes_and_a_orbits_without_changing_height() {
        let mut app = camera_app();
        app.update();
        let (start, yaw) = rig(&mut app);
        let (right, _) = pan_basis(app.world().resource::<CameraRig>());

        hold(&mut app, KeyCode::KeyQ);
        let (after_q, yaw_after_q) = rig(&mut app);
        let q_move = after_q - start;
        let dt = app.world().resource::<Time>().delta_secs();
        assert!((q_move.dot(-right) - CAMERA_PAN_SPEED_M_S * dt).abs() < 1e-2);
        assert!(q_move.z.abs() < 1e-4);
        assert_eq!(yaw_after_q, yaw);

        let (before_a, _) = rig(&mut app);
        hold(&mut app, KeyCode::KeyA);
        let (after_a, yaw_after_a) = rig(&mut app);
        assert_eq!(after_a, before_a);
        let orbit_dt = app.world().resource::<Time>().delta_secs();
        assert!((yaw_after_a - yaw + CAMERA_YAW_SPEED_RAD_S * orbit_dt).abs() < 1e-3);
    }

    #[test]
    fn other_camera_keys_keep_their_directions() {
        let mut app = camera_app();
        app.update();
        let (start, _) = rig(&mut app);
        let (right, forward) = pan_basis(app.world().resource::<CameraRig>());

        hold(&mut app, KeyCode::KeyD);
        let (after_d, _) = rig(&mut app);
        let dt = app.world().resource::<Time>().delta_secs();
        assert!(((after_d - start).dot(right) - CAMERA_PAN_SPEED_M_S * dt).abs() < 1e-2);

        hold(&mut app, KeyCode::KeyS);
        let (after_s, _) = rig(&mut app);
        let dt = app.world().resource::<Time>().delta_secs();
        assert!(((after_s - after_d).dot(-forward) - CAMERA_PAN_SPEED_M_S * dt).abs() < 1e-2);

        let (_, yaw_before_e) = rig(&mut app);
        hold(&mut app, KeyCode::KeyE);
        let (_, yaw_after_e) = rig(&mut app);
        let dt = app.world().resource::<Time>().delta_secs();
        assert!((yaw_after_e - yaw_before_e - CAMERA_YAW_SPEED_RAD_S * dt).abs() < 1e-3);

        let (before_r, _) = rig(&mut app);
        hold(&mut app, KeyCode::KeyR);
        let (after_r, _) = rig(&mut app);
        let dt = app.world().resource::<Time>().delta_secs();
        assert!((after_r.z - before_r.z - CAMERA_VERTICAL_SPEED_M_S * dt).abs() < 1e-2);
        assert_eq!(after_r.x, before_r.x);
        assert_eq!(after_r.y, before_r.y);

        hold(&mut app, KeyCode::KeyF);
        let (after_f, _) = rig(&mut app);
        let dt = app.world().resource::<Time>().delta_secs();
        assert!((after_r.z - after_f.z - CAMERA_VERTICAL_SPEED_M_S * dt).abs() < 1e-2);
    }

    #[test]
    fn page_up_and_page_down_change_deck_without_moving_the_camera() {
        let mut app = camera_app();
        app.update();
        let (start, yaw) = rig(&mut app);
        let eye_z = rig_transform(app.world().resource::<CameraRig>())
            .translation
            .z;

        hold(&mut app, KeyCode::PageUp);
        assert_eq!(app.world().resource::<CurrentDeck>().0, 7);
        let (after_up, yaw_after_up) = rig(&mut app);
        assert_eq!(after_up, start);
        assert_eq!(yaw_after_up, yaw);
        assert_eq!(
            rig_transform(app.world().resource::<CameraRig>())
                .translation
                .z,
            eye_z
        );

        app.world_mut().resource_mut::<CurrentDeck>().0 = DECK_COUNT - 1;
        hold(&mut app, KeyCode::PageUp);
        assert_eq!(app.world().resource::<CurrentDeck>().0, DECK_COUNT - 1);
        assert_eq!(rig(&mut app).0, start);

        app.world_mut().resource_mut::<CurrentDeck>().0 = 0;
        hold(&mut app, KeyCode::PageDown);
        assert_eq!(app.world().resource::<CurrentDeck>().0, 0);
        assert_eq!(rig(&mut app).0, start);

        app.world_mut().resource_mut::<CurrentDeck>().0 = 7;
        hold(&mut app, KeyCode::PageDown);
        assert_eq!(app.world().resource::<CurrentDeck>().0, 6);
        assert_eq!(rig(&mut app).0, start);
        assert_eq!(
            rig_transform(app.world().resource::<CameraRig>())
                .translation
                .z,
            eye_z
        );
    }

    #[test]
    fn plan_camera_height_does_not_follow_the_deck() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(CameraRig::default_for_deck(6));
        app.insert_resource(CurrentDeck(0));
        app.insert_resource(PlanView {
            enabled: true,
            scale: 1.0,
        });
        app.world_mut().spawn((
            GameCamera3d,
            Transform::default(),
            Projection::Perspective(PerspectiveProjection::default()),
        ));
        app.add_systems(Update, apply_camera);

        app.update();
        let mut query = app.world_mut().query::<&Transform>();
        let low = query.single(app.world()).unwrap().translation.z;

        app.world_mut().resource_mut::<CurrentDeck>().0 = DECK_COUNT - 1;
        app.update();
        let mut query = app.world_mut().query::<&Transform>();
        let high = query.single(app.world()).unwrap().translation.z;

        assert_eq!(low, plan_camera_z());
        assert_eq!(high, low);
        assert!(plan_camera_z() > (DECK_COUNT - 1) as f32 * DECK_HEIGHT_M);
    }

    #[test]
    fn keymap_names_the_swapped_keys() {
        let pan = BINDINGS
            .iter()
            .find(|binding| binding.action == "Pan")
            .unwrap();
        let orbit = BINDINGS
            .iter()
            .find(|binding| binding.action == "Orbit")
            .unwrap();
        assert_eq!(pan.keys, "W Q S D");
        assert_eq!(orbit.keys, "A / E, or right-drag");
        assert_eq!(
            format!("{:?}", pan_binding()),
            format!(
                "{:?}",
                viewer_input_map()
                    .get_dual_axislike(&ViewerAction::Pan)
                    .unwrap()[0]
            )
        );
        assert_eq!(
            format!("{:?}", orbit_binding()),
            format!(
                "{:?}",
                viewer_input_map()
                    .get_axislike(&ViewerAction::OrbitYaw)
                    .unwrap()[0]
            )
        );
    }

    #[test]
    fn directional_light_sun_shades_decks_and_bulkheads() {
        let rotation = sun_light_rotation();
        let to_sun = to_sun_direction(rotation);
        assert_eq!(to_sun, Transform::from_rotation(rotation).back().as_vec3());
        assert!(
            to_sun.z > 0.45,
            "deck tops should receive direct sun, got {to_sun:?}"
        );
        assert!(
            to_sun.xy().length() > 0.65,
            "port and starboard bulkheads should shade differently, got {to_sun:?}"
        );
        let packed = sun_uniform(rotation);
        assert_eq!(packed.truncate(), to_sun);
        assert!((packed.w - super::SUN_DIFFUSE_GAIN).abs() < 1e-6);
    }
}
