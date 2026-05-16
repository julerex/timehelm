//! Top-down 2D plan view of the ship (metres in XY; ship-space **+Y** = bow). The view rotates the hull
//! so bow runs **left–right** on screen.
//!
//! All 20 decks are baked to one vertex-coloured `Mesh2d` each at startup and share a single
//! white `ColorMaterial`; deck switching just toggles `Visibility` on the relevant entity.

use crate::deck_geometry::merged_plan_squares_mesh_colored;
use crate::deck_layout::{
    amenity_overlay, deck_layouts, AmenityKind, DeckLayouts, DeckTileBucket, DeckTiles, NUM_DECKS,
    TILE_CELL_M, TILE_VISUAL_SCALE,
};
use crate::shared::{asset_plugin, primary_window};
use crate::ship_hull::{SHIP_BEAM_M, SHIP_LENGTH_M};
use bevy::camera::visibility::RenderLayers;
use bevy::camera::{OrthographicProjection, Projection, ScalingMode};
use bevy::ecs::hierarchy::ChildSpawnerCommands;
use bevy::prelude::*;
use bevy::sprite::Anchor;
use std::f32::consts::FRAC_PI_2;

const SIM_DECK_INDEX: usize = 4;
const VERSION_NUMBER: i64 = 128;
const VIEW_WIDTH_M: f32 = SHIP_LENGTH_M * 1.12;

const Z_TILE_PLANE: f32 = 0.0;

/// Source-of-truth zone colours (shared by the mesh builder and on-map labels).
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

#[derive(Component)]
struct DeckLabel;

#[derive(Component)]
struct UiCamera;

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

fn deck_label_text(deck: usize) -> String {
    format!(
        "Version {VERSION_NUMBER} — 2D plan\nDeck {}/{}: {} | hull {:.0} m × {:.0} m\nPgUp/PgDn: decks | Bow → right\nZone & amenity labels match the visible deck only.",
        deck + 1,
        NUM_DECKS,
        DECK_NAMES[deck],
        SHIP_LENGTH_M,
        SHIP_BEAM_M,
    )
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
        let color = if let Some(amenity) = amenity_overlay(deck_index, *c) {
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

/// Slightly above tile mesh; below ad-hoc compass glyphs (`0.025`).
const Z_PLAN_LABELS: f32 = 0.035;

/// On-map text sizes (world `TextFont` units). Kept small so labels stay readable without dominating the plan.
const FONT_COMPASS: f32 = 12.5;
const FONT_PLAN_ZONE: f32 = 7.75;
const FONT_PLAN_AMENITY: f32 = 6.75;

/// Parent [`ShipPlan2dRotateRoot`] applies −90° Z so bow runs left→right on screen; counter-spin
/// keeps zone text horizontal while positions stay in ship metres (+Y bow).
fn plan_label_bundle(
    text: impl Into<String>,
    translation: Vec3,
    font_size: f32,
    color: Color,
) -> impl Bundle {
    (
        Text2d::new(text),
        TextFont {
            font_size,
            ..default()
        },
        TextColor(color),
        Anchor::CENTER,
        Transform::from_translation(translation).with_rotation(Quat::from_rotation_z(FRAC_PI_2)),
    )
}

/// Zone + amenity names parented under each deck mesh so [`sync_plan_deck_visibility`] shows only the
/// active deck’s overlays.
fn spawn_deck_overlay_labels(parent: &mut ChildSpawnerCommands<'_>, deck_i: usize) {
    parent.spawn(plan_label_bundle(
        "Cabins & hull perimeter (plan)",
        Vec3::new(-SHIP_BEAM_M * 0.34, 14.0, Z_PLAN_LABELS),
        FONT_PLAN_ZONE,
        Color::srgb(0.16, 0.12, 0.04),
    ));
    parent.spawn(plan_label_bundle(
        "Forward windows",
        Vec3::new(SHIP_BEAM_M * 0.36, SHIP_LENGTH_M * 0.21, Z_PLAN_LABELS),
        FONT_PLAN_ZONE,
        Color::srgba(0.98, 0.99, 1.0, 0.96),
    ));
    parent.spawn(plan_label_bundle(
        "Public / aft venues",
        Vec3::new(0.0, -SHIP_LENGTH_M * 0.36, Z_PLAN_LABELS),
        FONT_PLAN_ZONE,
        Color::srgb(0.1, 0.14, 0.2),
    ));
    parent.spawn(plan_label_bundle(
        "Interior structure (tint ↑ with deck)",
        Vec3::new(0.0, SHIP_LENGTH_M * 0.04, Z_PLAN_LABELS),
        FONT_PLAN_ZONE * 0.97,
        Color::srgba(0.96, 0.98, 1.0, 0.96),
    ));

    if (5..=10).contains(&deck_i) {
        parent.spawn(plan_label_bundle(
            AmenityKind::Theatre.label(),
            Vec3::new(0.0, SHIP_LENGTH_M * 0.425, Z_PLAN_LABELS),
            FONT_PLAN_AMENITY,
            Color::srgba(0.98, 0.97, 1.0, 0.96),
        ));
    }
    if (4..=7).contains(&deck_i) {
        parent.spawn(plan_label_bundle(
            AmenityKind::MainDining.label(),
            Vec3::new(0.0, -SHIP_LENGTH_M * 0.23, Z_PLAN_LABELS),
            FONT_PLAN_AMENITY,
            Color::srgb(0.12, 0.06, 0.02),
        ));
    }
    if (8..=12).contains(&deck_i) {
        parent.spawn(plan_label_bundle(
            AmenityKind::Buffet.label(),
            Vec3::new(-SHIP_BEAM_M * 0.28, SHIP_LENGTH_M * 0.17, Z_PLAN_LABELS),
            FONT_PLAN_AMENITY * 0.97,
            Color::srgb(0.05, 0.12, 0.06),
        ));
    }
    if (11..=16).contains(&deck_i) {
        parent.spawn(plan_label_bundle(
            AmenityKind::Pools.label(),
            Vec3::new(0.0, SHIP_LENGTH_M * 0.28, Z_PLAN_LABELS),
            FONT_PLAN_AMENITY,
            Color::srgb(0.04, 0.1, 0.14),
        ));
    }
    if (6..=9).contains(&deck_i) {
        parent.spawn(plan_label_bundle(
            AmenityKind::Casino.label(),
            Vec3::new(SHIP_BEAM_M * 0.33, SHIP_LENGTH_M * 0.04, Z_PLAN_LABELS),
            FONT_PLAN_AMENITY * 0.97,
            Color::srgba(1.0, 0.96, 0.98, 0.96),
        ));
    }
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
        .insert_resource(ClearColor(Color::srgb(0.04, 0.09, 0.16)))
        .add_systems(Startup, setup_2d)
        .add_systems(
            Update,
            (
                deck_switch_2d,
                sync_plan_deck_visibility,
                update_deck_label_2d,
            ),
        )
        .run();
}

fn setup_2d(
    mut commands: Commands,
    layouts: Res<DeckLayouts>,
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

    // World plan meshes and Text2d use the default render layer (0). The UI camera must *not* draw
    // those: its orthographic projection is pixel/window space, so sharing layer 0 would composite a
    // second, wrongly scaled copy of the ship (a "ghost" miniature). UI still reaches this camera via
    // `UiTargetCamera`; extraction does not rely on matching `RenderLayers`.
    let ui_camera = commands
        .spawn((
            Camera2d,
            Camera {
                order: 1,
                clear_color: ClearColorConfig::None,
                ..default()
            },
            RenderLayers::layer(1),
            UiCamera,
        ))
        .id();

    let rotate_root = commands
        .spawn((
            ShipPlan2dRotateRoot,
            Transform::from_rotation(Quat::from_rotation_z(-FRAC_PI_2)),
            Visibility::Inherited,
            GlobalTransform::default(),
        ))
        .id();

    commands.entity(rotate_root).with_children(|markers| {
        for (label, pos) in [
            ("Bow (+Y)", Vec3::new(0.0, SHIP_LENGTH_M * 0.53, 0.025)),
            ("Stern (−Y)", Vec3::new(0.0, -SHIP_LENGTH_M * 0.53, 0.025)),
            ("Port (−X)", Vec3::new(-SHIP_BEAM_M * 0.56, -6.0, 0.025)),
            ("Starboard (+X)", Vec3::new(SHIP_BEAM_M * 0.56, -6.0, 0.025)),
        ] {
            markers.spawn((
                Text2d::new(label),
                TextFont {
                    font_size: FONT_COMPASS,
                    ..default()
                },
                TextColor(Color::srgba(0.96, 0.97, 1.0, 0.93)),
                Anchor::CENTER,
                Transform::from_translation(pos),
            ));
        }
    });

    // One shared white material for every deck — vertex colours do all the work, so we never need
    // to allocate a `ColorMaterial` per bucket / per deck switch.
    let shared_material = materials.add(ColorMaterial::from(Color::WHITE));

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
                spawn_deck_overlay_labels(plan, deck_i);
            })
            .id();
    }
    commands.insert_resource(DeckContentEntities(deck_entities));

    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            left: Val::Px(10.0),
            ..default()
        },
        Text::new(deck_label_text(SIM_DECK_INDEX)),
        TextFont {
            font_size: 22.0,
            ..default()
        },
        TextColor(Color::WHITE),
        UiTargetCamera(ui_camera),
        DeckLabel,
    ));
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

fn update_deck_label_2d(
    current_deck: Res<CurrentDeck>,
    mut query: Query<&mut Text, With<DeckLabel>>,
) {
    if !current_deck.is_changed() {
        return;
    }
    for mut text in &mut query {
        text.0 = deck_label_text(current_deck.0);
    }
}
