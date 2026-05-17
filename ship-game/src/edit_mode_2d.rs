//! Plan-view edit mode: select cells and edit floor / wall materials from the HUD.

use crate::app_2d::{
    CurrentDeck, DeckContentEntities, DeckWalkGrids, GamePlanCamera2d, ShipPlan2dRotateRoot,
};
use crate::cell::Material;
use crate::cell_box::{CellIndex, PlanKey};
use crate::deck_layout::{DeckLayouts, CELL_SIZE_M};
use crate::plan_mesh::{rebuild_plan_deck_meshes, DeckPlanMeshes};
use crate::shared::cursor_in_game_viewport;
use bevy::prelude::*;

const PANEL_WIDTH_PX: f32 = 300.0;
const FIELD_ROW_GAP: f32 = 6.0;
const PICKER_ROW_GAP: f32 = 2.0;
const Z_HIGHLIGHT: f32 = 0.002;

#[derive(Resource, Default)]
pub struct PlanEditMode {
    pub active: bool,
}

#[derive(Resource, Default)]
pub struct SelectedPlanCell(pub Option<PlanKey>);

#[derive(Resource, Default)]
pub struct OpenMaterialPicker(pub Option<CellMaterialField>);

#[derive(Resource, Default)]
pub struct PlanDeckMeshDirty(pub Option<usize>);

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CellMaterialField {
    Floor,
    Wall1,
    Wall2,
    Wall3,
    Wall4,
}

impl CellMaterialField {
    const ALL: [Self; 5] = [
        Self::Floor,
        Self::Wall1,
        Self::Wall2,
        Self::Wall3,
        Self::Wall4,
    ];

    fn label(self) -> &'static str {
        match self {
            Self::Floor => "Floor",
            Self::Wall1 => "Wall 1 (+X / east)",
            Self::Wall2 => "Wall 2 (+Y / north)",
            Self::Wall3 => "Wall 3 (−X / west)",
            Self::Wall4 => "Wall 4 (−Y / south)",
        }
    }

    fn read(self, cell: &crate::cell::Cell) -> Material {
        match self {
            Self::Floor => cell.floor,
            Self::Wall1 => cell.wall1,
            Self::Wall2 => cell.wall2,
            Self::Wall3 => cell.wall3,
            Self::Wall4 => cell.wall4,
        }
    }

    fn write(self, cell: &mut crate::cell::Cell, material: Material) {
        match self {
            Self::Floor => cell.floor = material,
            Self::Wall1 => cell.wall1 = material,
            Self::Wall2 => cell.wall2 = material,
            Self::Wall3 => cell.wall3 = material,
            Self::Wall4 => cell.wall4 = material,
        }
    }
}

#[derive(Component)]
struct EditModePanelRoot;

#[derive(Component)]
struct EditModeBannerText;

#[derive(Component)]
struct EditModeCellSummaryText;

#[derive(Component)]
struct MaterialFieldRow {
    field: CellMaterialField,
}

#[derive(Component)]
struct MaterialDropdownButton {
    field: CellMaterialField,
}

#[derive(Component)]
struct MaterialDropdownLabel {
    field: CellMaterialField,
}

#[derive(Component)]
struct MaterialPickerPanel {
    field: CellMaterialField,
}

#[derive(Component)]
struct MaterialPickerOption {
    field: CellMaterialField,
    material: Material,
}

#[derive(Component)]
struct SelectedCellHighlight;

pub struct PlanEditPlugin;

impl Plugin for PlanEditPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PlanEditMode>()
            .init_resource::<SelectedPlanCell>()
            .init_resource::<OpenMaterialPicker>()
            .init_resource::<PlanDeckMeshDirty>()
            .add_systems(
                Update,
                (
                    toggle_plan_edit_mode,
                    clear_selection_on_deck_change,
                    select_cell_in_edit_mode,
                    sync_edit_mode_panel,
                    handle_material_dropdown_buttons,
                    handle_material_picker_options,
                    rebuild_dirty_plan_deck_meshes,
                    sync_selected_cell_highlight,
                ),
            );
    }
}

pub fn spawn_edit_mode_panel(commands: &mut Commands, ui_camera: Entity) {
    commands
        .spawn((
            Node {
                width: Val::Px(PANEL_WIDTH_PX),
                height: Val::Percent(100.0),
                position_type: PositionType::Absolute,
                top: Val::Px(0.0),
                right: Val::Px(0.0),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(12.0)),
                row_gap: Val::Px(8.0),
                overflow: Overflow::scroll_y(),
                ..default()
            },
            BackgroundColor(Color::srgba(0.06, 0.1, 0.14, 0.94)),
            Visibility::Hidden,
            EditModePanelRoot,
            UiTargetCamera(ui_camera),
        ))
        .with_children(|panel| {
            panel.spawn((
                Text::new("Edit mode: off"),
                TextFont {
                    font_size: 18.0,
                    ..default()
                },
                TextColor(Color::srgb(0.9, 0.85, 0.5)),
                EditModeBannerText,
            ));
            panel.spawn((
                Text::new("Press E to enter edit mode.\nClick a cell on the plan to select it."),
                TextFont {
                    font_size: 15.0,
                    ..default()
                },
                TextColor(Color::srgb(0.78, 0.84, 0.9)),
                EditModeCellSummaryText,
            ));
            for field in CellMaterialField::ALL {
                spawn_material_field_row(panel, field);
            }
        });
}

fn spawn_material_field_row(parent: &mut ChildSpawnerCommands, field: CellMaterialField) {
    parent
        .spawn((
            Node {
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(FIELD_ROW_GAP),
                display: Display::None,
                ..default()
            },
            MaterialFieldRow { field },
        ))
        .with_children(|row| {
            row.spawn((
                Text::new(field.label()),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::srgb(0.72, 0.78, 0.86)),
            ));
            row.spawn((
                Button,
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(28.0),
                    padding: UiRect::horizontal(Val::Px(8.0)),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::SpaceBetween,
                    border: UiRect::all(Val::Px(1.0)),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.14, 0.2, 0.28)),
                BorderColor::all(Color::srgb(0.35, 0.45, 0.55)),
                MaterialDropdownButton { field },
            ))
            .with_children(|btn| {
                btn.spawn((
                    Text::new("—"),
                    TextFont {
                        font_size: 14.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                    MaterialDropdownLabel { field },
                ));
                btn.spawn((
                    Text::new("▼"),
                    TextFont {
                        font_size: 12.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.65, 0.72, 0.8)),
                ));
            });
            row.spawn((
                Node {
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(PICKER_ROW_GAP),
                    display: Display::None,
                    padding: UiRect::left(Val::Px(4.0)),
                    ..default()
                },
                MaterialPickerPanel { field },
            ))
            .with_children(|picker| {
                for material in Material::ALL {
                    picker
                        .spawn((
                            Button,
                            Node {
                                width: Val::Percent(100.0),
                                height: Val::Px(24.0),
                                padding: UiRect::horizontal(Val::Px(8.0)),
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            BackgroundColor(material_picker_bg(material)),
                            MaterialPickerOption { field, material },
                        ))
                        .with_children(|opt| {
                            opt.spawn((
                                Text::new(material.label()),
                                TextFont {
                                    font_size: 13.0,
                                    ..default()
                                },
                                TextColor(Color::WHITE),
                            ));
                        });
                }
            });
        });
}

fn material_picker_bg(material: Material) -> Color {
    let c: LinearRgba = material.color().into();
    Color::linear_rgba(
        c.red * 0.35 + 0.08,
        c.green * 0.35 + 0.08,
        c.blue * 0.35 + 0.08,
        0.95,
    )
}

fn toggle_plan_edit_mode(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut edit_mode: ResMut<PlanEditMode>,
    mut picker: ResMut<OpenMaterialPicker>,
) {
    if keyboard.just_pressed(KeyCode::KeyE) {
        edit_mode.active = !edit_mode.active;
        if !edit_mode.active {
            picker.0 = None;
        }
    }
}

fn clear_selection_on_deck_change(
    current_deck: Res<CurrentDeck>,
    mut selected: ResMut<SelectedPlanCell>,
    mut picker: ResMut<OpenMaterialPicker>,
    mut last_deck: Local<Option<usize>>,
) {
    let deck = current_deck.0;
    if let Some(prev) = *last_deck {
        if prev != deck {
            selected.0 = None;
            picker.0 = None;
        }
    }
    *last_deck = Some(deck);
}

fn hull_xy_under_cursor(
    window: &Window,
    cameras: &Query<(&Camera, &GlobalTransform), With<GamePlanCamera2d>>,
    rotate_roots: &Query<&GlobalTransform, With<ShipPlan2dRotateRoot>>,
) -> Option<Vec2> {
    let Ok((camera, cam_tf)) = cameras.single() else {
        return None;
    };
    let cursor = cursor_in_game_viewport(window, camera)?;
    let world_xy = camera.viewport_to_world_2d(cam_tf, cursor).ok()?;
    let plan_root_tf = rotate_roots.single().ok()?;
    Some(
        plan_root_tf
            .affine()
            .inverse()
            .transform_point3(world_xy.extend(0.0))
            .truncate(),
    )
}

fn select_cell_in_edit_mode(
    edit_mode: Res<PlanEditMode>,
    mouse: Res<ButtonInput<MouseButton>>,
    window: Single<&Window>,
    current_deck: Res<CurrentDeck>,
    layouts: Res<DeckLayouts>,
    mut selected: ResMut<SelectedPlanCell>,
    mut picker: ResMut<OpenMaterialPicker>,
    cameras: Query<(&Camera, &GlobalTransform), With<GamePlanCamera2d>>,
    rotate_roots: Query<&GlobalTransform, With<ShipPlan2dRotateRoot>>,
) {
    if !edit_mode.active || !mouse.just_pressed(MouseButton::Left) {
        return;
    }
    let Some(hull_xy) = hull_xy_under_cursor(&window, &cameras, &rotate_roots) else {
        return;
    };
    let deck = current_deck.0 as u8;
    let Some(idx) = CellIndex::from_world_xy_deck(hull_xy, deck) else {
        selected.0 = None;
        picker.0 = None;
        return;
    };
    let plan = idx.plan();
    if layouts.deck(current_deck.0).get(plan).is_some() {
        selected.0 = Some(plan);
        picker.0 = None;
    } else {
        selected.0 = None;
        picker.0 = None;
    }
}

fn sync_edit_mode_panel(
    edit_mode: Res<PlanEditMode>,
    selected: Res<SelectedPlanCell>,
    picker: Res<OpenMaterialPicker>,
    current_deck: Res<CurrentDeck>,
    layouts: Res<DeckLayouts>,
    mut panel_vis: Query<&mut Visibility, With<EditModePanelRoot>>,
    mut banner: Query<
        &mut Text,
        (
            With<EditModeBannerText>,
            Without<EditModeCellSummaryText>,
            Without<MaterialDropdownLabel>,
        ),
    >,
    mut summary: Query<
        &mut Text,
        (
            With<EditModeCellSummaryText>,
            Without<EditModeBannerText>,
            Without<MaterialDropdownLabel>,
        ),
    >,
    mut field_rows: Query<(&MaterialFieldRow, &mut Node), Without<MaterialPickerPanel>>,
    mut dropdown_labels: Query<
        (&MaterialDropdownLabel, &mut Text),
        (
            Without<EditModeBannerText>,
            Without<EditModeCellSummaryText>,
        ),
    >,
    mut picker_panels: Query<(&MaterialPickerPanel, &mut Node), Without<MaterialFieldRow>>,
) {
    for mut vis in &mut panel_vis {
        *vis = if edit_mode.active {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }

    for mut text in &mut banner {
        text.0 = if edit_mode.active {
            "Edit mode: ON (E to exit)".to_string()
        } else {
            "Edit mode: off (E to enter)".to_string()
        };
    }

    let selection_active = edit_mode.active && selected.0.is_some();
    for (_, mut node) in &mut field_rows {
        node.display = if selection_active {
            Display::Flex
        } else {
            Display::None
        };
    }

    for mut text in &mut summary {
        text.0 = cell_summary_text(edit_mode.active, selected.0, current_deck.0, &layouts);
    }

    let Some(plan) = selected.0.filter(|_| edit_mode.active) else {
        for (_, mut node) in &mut picker_panels {
            node.display = Display::None;
        }
        return;
    };

    let deck_cells = layouts.deck(current_deck.0);
    let Some(cell) = deck_cells.get(plan) else {
        return;
    };

    for (label, mut text) in &mut dropdown_labels {
        text.0 = label.field.read(cell).label().to_string();
    }

    for (panel, mut node) in &mut picker_panels {
        node.display = if picker.0 == Some(panel.field) {
            Display::Flex
        } else {
            Display::None
        };
    }
}

fn cell_summary_text(
    edit_active: bool,
    plan: Option<PlanKey>,
    deck_index: usize,
    layouts: &DeckLayouts,
) -> String {
    if !edit_active {
        return "Press E to enter edit mode.\nClick a cell on the plan to select it.".to_string();
    }
    let Some(plan) = plan else {
        return "Edit mode active.\nClick a cell on the plan to select it.".to_string();
    };
    let deck_cells = layouts.deck(deck_index);
    let Some(cell) = deck_cells.get(plan) else {
        return format!("Cell ({}, {}) is outside the hull on this deck.", plan.0, plan.1);
    };
    let centre = deck_cells.index(plan).to_world_xy();
    let room_line = deck_cells
        .rooms
        .get(cell.room)
        .map(|room| format!("{} ({})", room.name, room.category.label()))
        .unwrap_or_else(|| format!("id {}", cell.room.0));
    format!(
        "Selected: cell ({}, {}) · deck {}\nCentre ({:.1}, {:.1}) m\nRoom: {room_line}\nAgents on cell: {}",
        plan.0,
        plan.1,
        deck_index + 1,
        centre.x,
        centre.y,
        cell.contents.agents().len()
    )
}

fn handle_material_dropdown_buttons(
    edit_mode: Res<PlanEditMode>,
    selected: Res<SelectedPlanCell>,
    mut picker: ResMut<OpenMaterialPicker>,
    mut interactions: Query<
        (&Interaction, &MaterialDropdownButton),
        (Changed<Interaction>, With<Button>),
    >,
) {
    if !edit_mode.active || selected.0.is_none() {
        return;
    }
    for (interaction, button) in &mut interactions {
        if *interaction == Interaction::Pressed {
            picker.0 = if picker.0 == Some(button.field) {
                None
            } else {
                Some(button.field)
            };
        }
    }
}

fn handle_material_picker_options(
    edit_mode: Res<PlanEditMode>,
    selected: Res<SelectedPlanCell>,
    current_deck: Res<CurrentDeck>,
    mut layouts: ResMut<DeckLayouts>,
    mut walk_grids: ResMut<DeckWalkGrids>,
    mut dirty: ResMut<PlanDeckMeshDirty>,
    mut picker: ResMut<OpenMaterialPicker>,
    mut interactions: Query<
        (&Interaction, &MaterialPickerOption),
        (Changed<Interaction>, With<Button>),
    >,
) {
    if !edit_mode.active {
        return;
    }
    let Some(plan) = selected.0 else {
        return;
    };
    let deck_i = current_deck.0;
    let index = CellIndex::with_plan(deck_i as u8, plan).expect("selected plan in box");

    for (interaction, option) in &mut interactions {
        if *interaction != Interaction::Pressed {
            continue;
        }
        if picker.0 != Some(option.field) {
            continue;
        }
        let Some(cell) = layouts.cell_mut(index) else {
            continue;
        };
        option.field.write(cell, option.material);
        dirty.0 = Some(deck_i);
        walk_grids.0[deck_i] = crate::cell_box::deck_walk_grid(&layouts.cells, deck_i as u8);
        picker.0 = None;
    }
}

fn rebuild_dirty_plan_deck_meshes(
    mut dirty: ResMut<PlanDeckMeshDirty>,
    layouts: Res<DeckLayouts>,
    mut meshes: ResMut<Assets<Mesh>>,
    deck_meshes: Res<DeckPlanMeshes>,
) {
    let Some(deck_i) = dirty.0.take() else {
        return;
    };
    rebuild_plan_deck_meshes(deck_i, &layouts, &mut meshes, &deck_meshes);
}

fn sync_selected_cell_highlight(
    mut commands: Commands,
    edit_mode: Res<PlanEditMode>,
    selected: Res<SelectedPlanCell>,
    current_deck: Res<CurrentDeck>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    deck_entities: Res<DeckContentEntities>,
    mut highlight: Local<Option<Entity>>,
    mut transforms: Query<&mut Transform, With<SelectedCellHighlight>>,
    mut visibilities: Query<&mut Visibility, With<SelectedCellHighlight>>,
    parents: Query<&ChildOf>,
) {
    let target = edit_mode.active.then_some(selected.0).flatten().and_then(|plan| {
        CellIndex::with_plan(current_deck.0 as u8, plan).map(CellIndex::to_world_xy)
    });

    if let Some(entity) = *highlight {
        if let Ok(parent) = parents.get(entity) {
            let expected_parent = deck_entities.0[current_deck.0];
            if parent.parent() != expected_parent {
                commands.entity(entity).despawn();
                *highlight = None;
            }
        }
    }

    let Some(pos) = target else {
        if let Some(entity) = highlight.take() {
            commands.entity(entity).despawn();
        }
        return;
    };

    if let Some(entity) = *highlight {
        if let Ok(mut tf) = transforms.get_mut(entity) {
            tf.translation = pos.extend(Z_HIGHLIGHT);
        }
        if let Ok(mut vis) = visibilities.get_mut(entity) {
            *vis = Visibility::Inherited;
        }
        return;
    }

    let mesh = meshes.add(Mesh::from(Rectangle::new(CELL_SIZE_M, CELL_SIZE_M)));
    let material = materials.add(ColorMaterial::from(Color::srgba(1.0, 0.92, 0.2, 0.45)));
    let entity = commands
        .spawn((
            Mesh2d(mesh),
            MeshMaterial2d(material),
            Transform::from_xyz(pos.x, pos.y, Z_HIGHLIGHT),
            SelectedCellHighlight,
        ))
        .id();
    commands.entity(deck_entities.0[current_deck.0]).add_child(entity);
    *highlight = Some(entity);
}
