//! Read-only 3D view of the persistent ship.
//!
//! Camera actions live in a leafwing action map. Press K for the keymap.
//! Interaction actions are registered and left unbound.

use crate::cabin_marks::{deck_door_mesh, deck_fitting_mesh};
use crate::deck_geometry::{extruded_polygon_deck_mesh, procedural_deck_plan_texture_image};
use crate::shader_embed::ShipShaderEmbedPlugin;
use crate::shared::{asset_plugin, primary_window};
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
    contents_label, format_unix_utc, point_in_polygon, CrewView, RoomKind, RoomView, WorldSnapshot,
    DECK_COUNT, DECK_HEIGHT_M, DECK_NAMES, SHIP_BEAM_M, SHIP_LENGTH_M,
};

const CAMERA_PAN_SPEED_M_S: f32 = 520.0;
const CAMERA_VERTICAL_SPEED_M_S: f32 = 260.0;
const CAMERA_YAW_SPEED_RAD_S: f32 = 1.75;
const CAMERA_DOLLY_SPEED_M_S: f32 = 520.0;
const CAMERA_SCROLL_ZOOM_FACTOR: f32 = 0.12;
const CAMERA_MOUSE_ORBIT_SENS: f32 = 0.005;
const CAMERA_MOUSE_PAN_SENS: f32 = 0.0022;
const CAM_DEFAULT_DISTANCE_M: f32 = 340.0;
const CAM_MIN_DISTANCE_M: f32 = 28.0;
const CAM_MAX_DISTANCE_M: f32 = 6200.0;
const CAM_PITCH_MIN: f32 = 0.15;
const CAM_PITCH_MAX: f32 = 1.42;
const FLOOR_THICKNESS_M: f32 = 0.22;
const WALL_HEIGHT_M: f32 = 1.15;
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
        keys: "W A S D",
        available: true,
    },
    Binding {
        action: "Orbit",
        keys: "Q / E, or right-drag",
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
    #[uniform(0)]
    clip_data: Vec4,
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
        .insert_resource(CurrentDeck(6))
        .insert_resource(CameraRig::default_for_deck(6))
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
        let dir0 = Vec3::new(0.82, -1.02, 0.55).normalize();
        Self {
            target: Vec3::new(
                SHIP_LENGTH_M * 0.5,
                SHIP_BEAM_M * 0.5,
                (deck as f32 + 0.4) * DECK_HEIGHT_M,
            ),
            yaw: dir0.y.atan2(dir0.x),
            pitch: dir0.z.clamp(-1.0, 1.0).asin(),
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
    let clip_handle = materials.add(ShipClipMaterial {
        clip_data: Vec4::new(cut_z(deck.0), 0.0, 0.0, 0.0),
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
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.9, 0.4, 0.0)),
    ));

    commands.spawn(viewer_input_map());
    spawn_hud(&mut commands, ui_camera);
}

fn viewer_input_map() -> InputMap<ViewerAction> {
    InputMap::default()
        .with_dual_axis(ViewerAction::Pan, VirtualDPad::wasd())
        .with_axis(
            ViewerAction::OrbitYaw,
            VirtualAxis::new(KeyCode::KeyQ, KeyCode::KeyE),
        )
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
                Transform::from_xyz(0.0, 0.0, deck_z + FLOOR_THICKNESS_M),
            ));
        }
        if let Some(mesh) = deck_door_mesh(&rooms) {
            commands.spawn((
                Mesh3d(meshes.add(mesh)),
                MeshMaterial3d(clip.0.clone()),
                Transform::from_xyz(0.0, 0.0, deck_z + FLOOR_THICKNESS_M),
            ));
        }
        if let Some(mesh) = deck_fitting_mesh(&rooms) {
            commands.spawn((
                Mesh3d(meshes.add(mesh)),
                MeshMaterial3d(clip.0.clone()),
                Transform::from_xyz(0.0, 0.0, deck_z + FLOOR_THICKNESS_M),
            ));
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
    rig.target += right * pan.x * CAMERA_PAN_SPEED_M_S * dt;
    rig.target += forward * pan.y * CAMERA_PAN_SPEED_M_S * dt;

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
            *projection = Projection::Orthographic(OrthographicProjection {
                scaling_mode: ScalingMode::FixedVertical {
                    viewport_height: SHIP_BEAM_M * 1.25,
                },
                scale: plan.scale,
                near: 0.05,
                far: 80.0,
                ..OrthographicProjection::default_3d()
            });
            let eye = Vec3::new(rig.target.x, rig.target.y, deck_z + 24.0);
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
    mut rig: ResMut<CameraRig>,
    mut keymap: ResMut<KeymapOpen>,
    mut panels: Query<&mut Node, With<KeymapPanel>>,
) {
    let Ok(action) = actions.single() else {
        return;
    };
    if action.just_pressed(&ViewerAction::DeckUp) && deck.0 + 1 < DECK_COUNT {
        deck.0 += 1;
        rig.target.z = (deck.0 as f32 + 0.4) * DECK_HEIGHT_M;
    }
    if action.just_pressed(&ViewerAction::DeckDown) && deck.0 > 0 {
        deck.0 -= 1;
        rig.target.z = (deck.0 as f32 + 0.4) * DECK_HEIGHT_M;
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
) {
    let Some(shared) = shared else {
        return;
    };
    if !deck.is_changed() && !shared.is_changed() {
        return;
    }
    if let Some(material) = materials.get_mut(&shared.0) {
        material.clip_data.x = cut_z(deck.0);
    }
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
    let gap = 1.6_f32.min((hi - lo) * 0.7);
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
    let nx = -dy / length;
    let ny = dx / length;
    let inset = 0.08;
    let x0 = start[0] + nx * inset;
    let y0 = start[1] + ny * inset;
    let x1 = end[0] + nx * inset;
    let y1 = end[1] + ny * inset;
    let base = parts.positions.len() as u32;
    let z1 = WALL_HEIGHT_M;
    for (x, y, z) in [(x0, y0, 0.0), (x1, y1, 0.0), (x1, y1, z1), (x0, y0, z1)] {
        parts.positions.push([x, y, z]);
        parts.normals.push([nx, ny, 0.0]);
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
