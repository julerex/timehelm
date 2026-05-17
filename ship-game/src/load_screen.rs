//! Save-game picker shown before the ship layout is loaded (replaces procedural startup).

#![allow(clippy::type_complexity)]

use crate::deck_layout::DeckLayouts;
#[cfg(target_arch = "wasm32")]
use crate::ship_save::{decode_save, saved_ship_manifest_url, saved_ship_url, SavedShipManifest};
use crate::ship_save::{list_saved_ship_files, load_save_by_filename};
use bevy::prelude::*;
use bevy::tasks::{AsyncComputeTaskPool, Task};
use std::collections::HashMap;

/// Startup shows the load menu; gameplay systems run only after a save is chosen.
#[derive(States, Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GamePhase {
    #[default]
    LoadMenu,
    InGame,
}

#[derive(Resource, Default)]
struct AvailableSaves {
    discovered: bool,
    files: Vec<String>,
}

#[derive(Resource, Default)]
struct LoadMenuStatus(String);

#[derive(Component)]
struct LoadMenuRoot;

#[derive(Component)]
struct LoadMenuStatusText;

#[derive(Component)]
struct LoadMenuList;

#[derive(Component)]
struct SaveSlotButton(String);

#[cfg(target_arch = "wasm32")]
#[derive(Resource, Default)]
struct ManifestFetchTask(Option<Task<Result<Vec<String>, String>>>);

#[derive(Resource, Default)]
struct PendingSaveLoad(Option<Task<Result<DeckLayouts, String>>>);

pub struct LoadScreenPlugin;

impl Plugin for LoadScreenPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GamePhase>()
            .init_resource::<AvailableSaves>()
            .init_resource::<LoadMenuStatus>()
            .init_resource::<PendingSaveLoad>();
        #[cfg(target_arch = "wasm32")]
        app.init_resource::<ManifestFetchTask>();
        app.add_systems(
            Update,
            (
                discover_saves,
                rebuild_save_list_ui,
                save_slot_clicks,
                poll_pending_save_load,
            )
                .run_if(in_state(GamePhase::LoadMenu)),
        );
        #[cfg(target_arch = "wasm32")]
        app.add_systems(
            Update,
            poll_manifest_fetch.run_if(in_state(GamePhase::LoadMenu)),
        );
        app.add_systems(OnEnter(GamePhase::InGame), hide_load_menu);
    }
}

/// Full-screen load UI parented to the UI camera.
pub fn spawn_load_menu(commands: &mut Commands, ui_camera: Entity) {
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                position_type: PositionType::Absolute,
                top: Val::Px(0.0),
                left: Val::Px(0.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                padding: UiRect::all(Val::Px(24.0)),
                row_gap: Val::Px(16.0),
                ..default()
            },
            BackgroundColor(Color::srgb(0.04, 0.09, 0.16)),
            UiTargetCamera(ui_camera),
            LoadMenuRoot,
        ))
        .with_children(|menu| {
            menu.spawn((
                Text::new("Load ship"),
                TextFont {
                    font_size: 32.0,
                    ..default()
                },
                TextColor(Color::srgb(0.9, 0.94, 1.0)),
            ));
            menu.spawn((
                Text::new("Choose a saved layout"),
                TextFont {
                    font_size: 18.0,
                    ..default()
                },
                TextColor(Color::srgb(0.65, 0.75, 0.88)),
            ));
            menu.spawn((
                Node {
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(10.0),
                    max_height: Val::Px(420.0),
                    overflow: Overflow::scroll_y(),
                    padding: UiRect::axes(Val::Px(8.0), Val::Px(4.0)),
                    ..default()
                },
                LoadMenuList,
            ));
            menu.spawn((
                Text::new(""),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::srgb(0.75, 0.82, 0.9)),
                LoadMenuStatusText,
            ));
        });
}

fn discover_saves(
    mut available: ResMut<AvailableSaves>,
    mut status: ResMut<LoadMenuStatus>,
    #[cfg(target_arch = "wasm32")] mut manifest_task: ResMut<ManifestFetchTask>,
) {
    if available.discovered {
        return;
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        available.files = list_saved_ship_files();
        available.discovered = true;
        if available.files.is_empty() {
            status.0 = "No saves in saved_ships/ — run make write-ship-default".into();
        }
    }

    #[cfg(target_arch = "wasm32")]
    {
        if manifest_task.0.is_some() {
            return;
        }
        status.0 = "Loading save list…".into();
        manifest_task.0 = Some(AsyncComputeTaskPool::get().spawn(fetch_save_manifest()));
    }
}

#[cfg(target_arch = "wasm32")]
async fn fetch_save_manifest() -> Result<Vec<String>, String> {
    use wasm_bindgen::JsCast;
    use wasm_bindgen_futures::JsFuture;

    let window = web_sys::window().ok_or("no window")?;
    let resp_val = JsFuture::from(window.fetch_with_str(saved_ship_manifest_url()))
        .await
        .map_err(|e| format!("fetch failed: {e:?}"))?;
    let resp: web_sys::Response = resp_val
        .dyn_into()
        .map_err(|_| "response is not Response")?;
    if !resp.ok() {
        return Err(format!("manifest HTTP {}", resp.status()));
    }
    let text = JsFuture::from(resp.text().map_err(|_| "response.text() unavailable")?)
        .await
        .map_err(|e| format!("read body: {e:?}"))?
        .as_string()
        .ok_or("manifest body is not text")?;
    let manifest: SavedShipManifest =
        serde_json::from_str(&text).map_err(|e| format!("manifest JSON: {e}"))?;
    Ok(manifest.saves)
}

#[cfg(target_arch = "wasm32")]
async fn fetch_save_bytes(url: &str) -> Result<Vec<u8>, String> {
    use wasm_bindgen::JsCast;
    use wasm_bindgen_futures::JsFuture;

    let window = web_sys::window().ok_or("no window")?;
    let resp_val = JsFuture::from(window.fetch_with_str(url))
        .await
        .map_err(|e| format!("fetch failed: {e:?}"))?;
    let resp: web_sys::Response = resp_val
        .dyn_into()
        .map_err(|_| "response is not Response")?;
    if !resp.ok() {
        return Err(format!("save HTTP {}", resp.status()));
    }
    let buffer = JsFuture::from(
        resp.array_buffer()
            .map_err(|_| "array_buffer unavailable")?,
    )
    .await
    .map_err(|e| format!("read body: {e:?}"))?;
    let array = js_sys::Uint8Array::new(&buffer);
    Ok(array.to_vec())
}

#[cfg(target_arch = "wasm32")]
fn poll_manifest_fetch(
    mut available: ResMut<AvailableSaves>,
    mut status: ResMut<LoadMenuStatus>,
    mut manifest_task: ResMut<ManifestFetchTask>,
) {
    let Some(mut task) = manifest_task.0.take() else {
        return;
    };
    if let Some(result) = block_on_task(&mut task) {
        match result {
            Ok(files) => {
                available.files = files;
                if available.files.is_empty() {
                    status.0 =
                        "No saves in /saved_ships/ — add .ship.zst files and manifest.json".into();
                } else {
                    status.0.clear();
                }
            }
            Err(e) => {
                status.0 = format!("Could not load save list: {e}");
            }
        }
        available.discovered = true;
    } else {
        manifest_task.0 = Some(task);
    }
}

fn block_on_task<T>(task: &mut Task<T>) -> Option<T> {
    futures_lite::future::block_on(futures_lite::future::poll_once(task))
}

fn rebuild_save_list_ui(
    available: Res<AvailableSaves>,
    status: Res<LoadMenuStatus>,
    mut commands: Commands,
    list_roots: Query<Entity, With<LoadMenuList>>,
    slot_buttons: Query<Entity, With<SaveSlotButton>>,
    mut status_texts: Query<&mut Text, With<LoadMenuStatusText>>,
    mut built: Local<HashMap<String, ()>>,
) {
    if !available.discovered {
        return;
    }

    for mut text in &mut status_texts {
        text.0 = status.0.clone();
    }

    let current: HashMap<String, ()> = available.files.iter().map(|f| (f.clone(), ())).collect();
    if *built == current {
        return;
    }
    *built = current.clone();

    for entity in &slot_buttons {
        commands.entity(entity).despawn();
    }

    let Ok(list_entity) = list_roots.single() else {
        return;
    };

    if available.files.is_empty() {
        return;
    }

    commands.entity(list_entity).with_children(|list| {
        for filename in &available.files {
            let label = save_display_name(filename);
            list.spawn((
                Button,
                Node {
                    min_width: Val::Px(280.0),
                    padding: UiRect::axes(Val::Px(16.0), Val::Px(10.0)),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.12, 0.2, 0.32)),
                SaveSlotButton(filename.clone()),
            ))
            .with_children(|btn| {
                btn.spawn((
                    Text::new(label),
                    TextFont {
                        font_size: 20.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                ));
            });
        }
    });
}

fn save_display_name(filename: &str) -> String {
    filename
        .strip_suffix(".ship.zst")
        .unwrap_or(filename)
        .replace('_', " ")
}

fn save_slot_clicks(
    mut interaction_query: Query<
        (&Interaction, &SaveSlotButton),
        (Changed<Interaction>, With<Button>),
    >,
    mut status: ResMut<LoadMenuStatus>,
    mut pending: ResMut<PendingSaveLoad>,
) {
    if pending.0.is_some() {
        return;
    }

    for (interaction, slot) in &mut interaction_query {
        if *interaction != Interaction::Pressed {
            continue;
        }
        status.0 = format!("Loading {}…", save_display_name(&slot.0));

        #[cfg(not(target_arch = "wasm32"))]
        {
            match load_save_by_filename(&slot.0) {
                Ok(layouts) => {
                    pending.0 = Some(AsyncComputeTaskPool::get().spawn(async { Ok(layouts) }));
                }
                Err(e) => status.0 = format!("Load failed: {e}"),
            }
        }

        #[cfg(target_arch = "wasm32")]
        {
            let filename = slot.0.clone();
            pending.0 = Some(AsyncComputeTaskPool::get().spawn(async move {
                let url = saved_ship_url(&filename);
                let bytes = fetch_save_bytes(&url).await?;
                decode_save(&bytes).map_err(|e| e.to_string())
            }));
        }
    }
}

fn poll_pending_save_load(
    mut pending: ResMut<PendingSaveLoad>,
    mut layouts: ResMut<DeckLayouts>,
    mut status: ResMut<LoadMenuStatus>,
    mut next_phase: ResMut<NextState<GamePhase>>,
) {
    let Some(mut task) = pending.0.take() else {
        return;
    };
    let Some(result) = block_on_task(&mut task) else {
        pending.0 = Some(task);
        return;
    };
    match result {
        Ok(loaded) => {
            *layouts = loaded;
            status.0.clear();
            next_phase.set(GamePhase::InGame);
        }
        Err(e) => status.0 = format!("Load failed: {e}"),
    }
}

fn hide_load_menu(mut commands: Commands, roots: Query<Entity, With<LoadMenuRoot>>) {
    for entity in &roots {
        commands.entity(entity).despawn();
    }
}
