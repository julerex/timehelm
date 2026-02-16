//! Ship Game - A 3D Bevy cruise ship game
//!
//! Entry point for the ship game accessible at /ship.
//! A cruise ship game where you can move between different decks in 3D.

use bevy::prelude::*;

const DECK_NAMES: [&str; 6] = [
    "Sun Deck",
    "Pool Deck",
    "Promenade Deck",
    "Main Deck",
    "Lower Deck",
    "Engine Deck",
];

const MOVE_SPEED: f32 = 8.0;
const DECK_HEIGHT: f32 = 4.0;

#[derive(Component)]
struct Player;

#[derive(Component)]
struct DeckLabel;

#[derive(Component)]
struct DeckEntity(usize);

#[derive(Resource)]
struct CurrentDeck(usize);

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
        .insert_resource(CurrentDeck(0))
        .add_systems(Startup, setup)
        .add_systems(Update, (player_movement, deck_switch, update_deck_label))
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Camera
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0.0, 5.0, 15.0)
            .looking_at(Vec3::new(0.0, 2.0, 0.0), Vec3::Y),
        ..default()
    });

    // Directional light
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: 10000.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(5.0, 15.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });

    // Ambient light
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 500.0,
            ..default()
        },
        transform: Transform::from_xyz(0.0, 10.0, 0.0),
        ..default()
    });

    // Ocean/sky color material
    let _deck_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.2, 0.35, 0.5),
        ..default()
    });

    let railing_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.6, 0.5, 0.4),
        ..default()
    });

    // Create decks (6 levels)
    for i in 0..6 {
        let y = i as f32 * DECK_HEIGHT;
        let hue = 0.55 + (i as f32 * 0.02);
        let deck_color = Color::hsl(hue * 360.0 % 360.0, 0.3, 0.35);

        let visible = i == 0;
        // Deck floor (long ship shape)
        commands.spawn((
            PbrBundle {
                mesh: meshes.add(Cuboid::new(15.0, 0.15, 4.0)),
                material: materials.add(StandardMaterial {
                    base_color: deck_color,
                    ..default()
                }),
                transform: Transform::from_xyz(0.0, y, 0.0),
                visibility: if visible {
                    Visibility::Visible
                } else {
                    Visibility::Hidden
                },
                ..default()
            },
            DeckEntity(i),
        ));

        // Railings on sides
        for z_side in [-4.5, 4.5] {
            commands.spawn((
                PbrBundle {
                    mesh: meshes.add(Cuboid::new(14.0, 0.5, 0.15)),
                    material: railing_material.clone(),
                    transform: Transform::from_xyz(0.0, y + 0.8, z_side),
                    visibility: if visible {
                        Visibility::Visible
                    } else {
                        Visibility::Hidden
                    },
                    ..default()
                },
                DeckEntity(i),
            ));
        }

        // Windows along the deck
        for j in -5..=5 {
            let x = j as f32 * 2.5;
            commands.spawn((
                PbrBundle {
                    mesh: meshes.add(Cuboid::new(0.75, 0.6, 0.05)),
                    material: materials.add(StandardMaterial {
                        base_color: Color::srgb(0.5, 0.7, 0.95),
                        emissive: Color::srgb(0.2, 0.3, 0.5).into(),
                        ..default()
                    }),
                    transform: Transform::from_xyz(x, y + 2.0, 4.0),
                    visibility: if visible {
                        Visibility::Visible
                    } else {
                        Visibility::Hidden
                    },
                    ..default()
                },
                DeckEntity(i),
            ));
        }
    }

    // Player (green capsule-like box)
    let player_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.2, 0.8, 0.3),
        ..default()
    });
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Cuboid::new(0.3, 0.6, 0.3)),
            material: player_material,
            transform: Transform::from_xyz(0.0, DECK_HEIGHT / 2.0 + 0.3, 0.0),
            ..default()
        },
        Player,
    ));

    // Deck label (empty - updated by system)
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
                    font_size: 24.0,
                    color: Color::WHITE,
                    ..default()
                },
            ),
            ..default()
        },
        DeckLabel,
    ));
}

fn player_movement(
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut query: Query<&mut Transform, With<Player>>,
) {
    let mut dir = Vec3::ZERO;
    if keyboard.pressed(KeyCode::KeyA) || keyboard.pressed(KeyCode::ArrowLeft) {
        dir.x -= 1.0;
    }
    if keyboard.pressed(KeyCode::KeyD) || keyboard.pressed(KeyCode::ArrowRight) {
        dir.x += 1.0;
    }
    if dir != Vec3::ZERO {
        dir = dir.normalize();
        for mut transform in &mut query {
            transform.translation += dir * MOVE_SPEED * time.delta_seconds();
            transform.translation.x = transform.translation.x.clamp(-12.0, 12.0);
            // Face movement direction
            if dir.x != 0.0 {
                transform.rotation = Quat::from_rotation_y(if dir.x > 0.0 { 0.0 } else { std::f32::consts::PI });
            }
        }
    }
}

fn deck_switch(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut current_deck: ResMut<CurrentDeck>,
    mut deck_query: Query<(&mut Visibility, &DeckEntity)>,
    mut player_query: Query<&mut Transform, With<Player>>,
) {
    let mut changed = false;
    if keyboard.just_pressed(KeyCode::PageUp) {
        if current_deck.0 > 0 {
            current_deck.0 -= 1;
            changed = true;
        }
    }
    if keyboard.just_pressed(KeyCode::PageDown) {
        if current_deck.0 < 5 {
            current_deck.0 += 1;
            changed = true;
        }
    }
    if changed {
        for (mut vis, deck_ent) in deck_query.iter_mut() {
            *vis = if deck_ent.0 == current_deck.0 {
                Visibility::Visible
            } else {
                Visibility::Hidden
            };
        }
        let y = current_deck.0 as f32 * DECK_HEIGHT + DECK_HEIGHT / 2.0 + 0.3;
        for mut transform in &mut player_query {
            transform.translation.y = y;
        }
    }
}

fn update_deck_label(
    current_deck: Res<CurrentDeck>,
    mut query: Query<&mut Text, With<DeckLabel>>,
) {
    for mut text in &mut query {
        text.sections[0].value = format!(
            "Deck {}/6: {}\nLeft/Right or A/D - Page Up/Down decks",
            current_deck.0 + 1,
            DECK_NAMES[current_deck.0]
        );
    }
}
