//! Top-down 2D plan view of the ship hull (metres in XY; +Y bow).

use crate::shared::{asset_plugin, primary_window};
use crate::ship_hull::{
    deck_hull_polygon, deck_hull_polygon_upper, FIRST_UPPER_DECK_STYLE_INDEX, SHIP_BEAM_M,
    SHIP_LENGTH_M,
};
use bevy::asset::RenderAssetUsages;
use bevy::camera::{OrthographicProjection, Projection, ScalingMode};
use bevy::mesh::{Indices, Mesh, PrimitiveTopology};
use bevy::prelude::*;

const NUM_DECKS: usize = 20;
const SIM_DECK_INDEX: usize = 4;
const VERSION_NUMBER: i64 = 128;
/// Hull mesh vertices are in **metres**. `FixedVertical` projection so 1 world unit = 1 m (not 1 px).
const VIEW_HEIGHT_M: f32 = SHIP_LENGTH_M * 1.12;

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

#[derive(Resource)]
struct CurrentDeck(usize);

#[derive(Resource, Clone)]
struct Hull2dMeshHandle(Handle<Mesh>);

#[derive(Component)]
struct DeckLabel;

#[derive(Component)]
struct UiCamera;

fn deck_label_text(deck: usize) -> String {
    format!(
        "Version {VERSION_NUMBER} — 2D plan\nDeck {}/{}: {} | hull {:.0} m × {:.0} m\nPgUp/PgDn: deck | +Y = bow",
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
    let positions: Vec<[f32; 3]> = poly.iter().map(|p| [p.x, p.y, 0.0]).collect();
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
        .insert_resource(ClearColor(Color::srgb(0.04, 0.09, 0.16)))
        .add_systems(Startup, setup_2d)
        .add_systems(
            Update,
            (
                deck_switch_2d,
                update_hull_mesh_on_deck_change,
                update_deck_label_2d,
            ),
        )
        .run();
}

fn setup_2d(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let poly = hull_polygon_for_deck(SIM_DECK_INDEX);
    let mesh = triangulated_plan_mesh(&poly);
    let mesh_h = meshes.add(mesh);
    commands.insert_resource(Hull2dMeshHandle(mesh_h.clone()));

    // Bevy 0.18 default 2D ortho uses `WindowSize` (≈1 world unit per pixel). Hull span is ~300 m; without
    // `FixedVertical` the ship is a sub-pixel speck. World units = metres for the plan mesh.
    let mut world_ortho = OrthographicProjection::default_2d();
    world_ortho.scaling_mode = ScalingMode::FixedVertical {
        viewport_height: VIEW_HEIGHT_M,
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

    commands.spawn((
        Mesh2d(mesh_h),
        MeshMaterial2d(materials.add(Color::srgb(0.32, 0.62, 0.78))),
        Transform::IDENTITY,
    ));

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

fn update_hull_mesh_on_deck_change(
    current: Res<CurrentDeck>,
    hull: Res<Hull2dMeshHandle>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    if !current.is_changed() {
        return;
    }
    let poly = hull_polygon_for_deck(current.0);
    let new_mesh = triangulated_plan_mesh(&poly);
    if let Some(m) = meshes.get_mut(&hull.0) {
        *m = new_mesh;
    }
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
