//! Top-down 2D plan view of the ship (metres in XY; ship-space **+Y** = bow). The view rotates the hull
//! so bow runs **left–right** on screen.

use crate::deck_geometry::merged_plan_squares_mesh;
use crate::deck_layout::{
    amenity_overlay, deck_layouts, DeckLayouts, DeckTileBucket, NUM_DECKS, TILE_CELL_M,
    TILE_VISUAL_SCALE,
};
use crate::shared::{asset_plugin, primary_window};
use crate::ship_hull::{
    deck_hull_polygon, deck_hull_polygon_upper, FIRST_UPPER_DECK_STYLE_INDEX, SHIP_BEAM_M,
    SHIP_LENGTH_M,
};
use bevy::asset::RenderAssetUsages;
use bevy::camera::{OrthographicProjection, Projection, ScalingMode};
use bevy::mesh::{Indices, Mesh, PrimitiveTopology};
use bevy::prelude::*;
use std::collections::HashMap;
use std::f32::consts::FRAC_PI_2;

const SIM_DECK_INDEX: usize = 4;
const VERSION_NUMBER: i64 = 128;
const VIEW_WIDTH_M: f32 = SHIP_LENGTH_M * 1.12;

/// Local quad **Z** layering (camera looks toward **−Z**, positive Z is nearer / above previous draw).
const Z_HULL_FILL: f32 = -0.002;
const Z_TILE_BUCKETS: f32 = 0.0;
const Z_AMENITIES: f32 = 0.004;

#[derive(Component)]
struct ShipPlan2dRotateRoot;

#[derive(Resource)]
struct Plan2dContentHolder(Option<Entity>);

#[derive(Resource)]
struct ShipPlanRotateRootEntity(Entity);

#[derive(Resource)]
struct CurrentDeck(usize);

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
        "Version {VERSION_NUMBER} — 2D plan\nDeck {}/{}: {} | hull {:.0} m × {:.0} m\nPgUp/PgDn: decks | Bow → right\nCabins/zones match 3D model; overlays: dining, pools, theatre, casino",
        deck + 1,
        NUM_DECKS,
        DECK_NAMES[deck],
        SHIP_LENGTH_M,
        SHIP_BEAM_M,
    )
}

fn hull_polygon_for_deck(deck: usize) -> Vec<Vec2> {
    if deck >= FIRST_UPPER_DECK_STYLE_INDEX {
        deck_hull_polygon_upper()
    } else {
        deck_hull_polygon()
    }
}

fn triangulated_plan_mesh(poly: &[Vec2]) -> Mesh {
    if poly.len() < 3 {
        return Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        );
    }
    let flat: Vec<f64> = poly.iter().flat_map(|p| [p.x as f64, p.y as f64]).collect();
    let Ok(indices_u32) = earcutr::earcut(&flat, &[], 2) else {
        return Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        );
    };
    let positions: Vec<[f32; 3]> = poly.iter().map(|p| [p.x, p.y, Z_HULL_FILL]).collect();
    let normals: Vec<[f32; 3]> = (0..poly.len()).map(|_| [0.0, 0.0, 1.0]).collect();
    let indices: Vec<u32> = indices_u32.iter().map(|&i| i as u32).collect();
    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_indices(Indices::U32(indices));
    mesh
}

fn rgb_key(c: Color) -> [u16; 3] {
    let l: LinearRgba = c.into();
    fn q(x: f32) -> u16 {
        (x.clamp(0.0, 1.0) * 65535.0).round() as u16
    }
    [q(l.red), q(l.green), q(l.blue)]
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
        .insert_resource(Plan2dContentHolder(None))
        .insert_resource(ClearColor(Color::srgb(0.04, 0.09, 0.16)))
        .add_systems(Startup, setup_2d)
        .add_systems(
            Update,
            (
                deck_switch_2d,
                sync_plan_deck_content_2d,
                update_deck_label_2d,
            ),
        )
        .run();
}

fn setup_2d(mut commands: Commands) {
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

    let rotate_root = commands
        .spawn((
            ShipPlan2dRotateRoot,
            Transform::from_rotation(Quat::from_rotation_z(-FRAC_PI_2)),
            Visibility::Inherited,
            GlobalTransform::default(),
        ))
        .id();
    commands.insert_resource(ShipPlanRotateRootEntity(rotate_root));

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

/// Replaces the rotating root’s sole content child: hull fill, tile buckets, amenity overlays.
fn rebuild_deck_visuals_into_holder(
    deck_index: usize,
    rotate_root: Entity,
    layouts: &DeckLayouts,
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
    holder_res: &mut Plan2dContentHolder,
) {
    if let Some(prev) = holder_res.0.take() {
        commands.entity(prev).despawn();
    }

    let layout = &layouts.0[deck_index];
    let poly = hull_polygon_for_deck(deck_index);

    let edge_deck = Color::srgb(0.38, 0.3, 0.24);
    let window_color = Color::srgb(0.42, 0.62, 0.9);
    let outer_cabin = Color::srgb(0.95, 0.82, 0.35);
    let inner_cabin = Color::srgb(0.92, 0.55, 0.72);
    let public_deck = Color::srgb(0.78, 0.86, 0.92);
    let hue = 0.52 + (deck_index as f32 * 0.012);
    let base_tint = Color::hsla((hue * 360.0) % 360.0, 0.28, 0.42, 1.0);
    let bucket_colour: [Color; DeckTileBucket::COUNT] = [
        edge_deck,
        window_color,
        inner_cabin,
        outer_cabin,
        public_deck,
        base_tint,
    ];

    let half_tile = TILE_CELL_M * TILE_VISUAL_SCALE * 0.5;
    let mut buckets: [Vec<Vec2>; DeckTileBucket::COUNT] = std::array::from_fn(|_| Vec::new());
    let mut amenity_buckets: HashMap<[u16; 3], (Color, Vec<Vec2>)> = HashMap::new();

    for c in &layout.centers {
        let cell = (
            (c.x / TILE_CELL_M).round() as i32,
            (c.y / TILE_CELL_M).round() as i32,
        );
        let b = DeckTileBucket::classify(*c, cell, &layout.occupied);
        if let Some(ov) = amenity_overlay(deck_index, *c) {
            let key = rgb_key(ov);
            amenity_buckets
                .entry(key)
                .or_insert_with(|| (ov, Vec::new()))
                .1
                .push(*c);
        } else {
            buckets[b.idx()].push(*c);
        }
    }

    let hull_mesh = meshes.add(triangulated_plan_mesh(&poly));

    let content_id = commands
        .spawn((
            Transform::default(),
            Visibility::Inherited,
            GlobalTransform::default(),
            ChildOf(rotate_root),
        ))
        .with_children(|p| {
            p.spawn((
                Mesh2d(hull_mesh),
                MeshMaterial2d(materials.add(Color::srgb(0.22, 0.28, 0.42))),
                Transform::IDENTITY,
            ));

            let z_bucket = Z_TILE_BUCKETS;

            for (bi, bucket_centres) in buckets.into_iter().enumerate() {
                if bucket_centres.is_empty() {
                    continue;
                }
                let zm = meshes.add(merged_plan_squares_mesh(
                    &bucket_centres,
                    half_tile,
                    z_bucket + bi as f32 * 1e-4,
                ));
                let col = bucket_colour[bi];
                p.spawn((
                    Mesh2d(zm),
                    MeshMaterial2d(materials.add(col)),
                    Transform::IDENTITY,
                ));
            }

            let z_am = Z_AMENITIES;
            for (ki, (_, (col, centres))) in amenity_buckets.into_iter().enumerate() {
                if centres.is_empty() {
                    continue;
                }
                let zm = meshes.add(merged_plan_squares_mesh(
                    &centres,
                    half_tile * 0.95,
                    z_am + ki as f32 * 2e-4,
                ));
                p.spawn((
                    Mesh2d(zm),
                    MeshMaterial2d(materials.add(col)),
                    Transform::IDENTITY,
                ));
            }
        })
        .id();

    holder_res.0 = Some(content_id);
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

#[allow(clippy::too_many_arguments)]
fn sync_plan_deck_content_2d(
    current: Res<CurrentDeck>,
    layouts: Res<DeckLayouts>,
    root: Res<ShipPlanRotateRootEntity>,
    mut holder: ResMut<Plan2dContentHolder>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut synced: Local<bool>,
) {
    let need = !*synced || current.is_changed();
    if !need {
        return;
    }
    *synced = true;
    rebuild_deck_visuals_into_holder(
        current.0,
        root.0,
        layouts.as_ref(),
        &mut commands,
        &mut meshes,
        &mut materials,
        holder.as_mut(),
    );
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
