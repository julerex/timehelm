//! Plan-view edit mode: select cells and edit floor / wall materials from the HUD.

#![allow(clippy::too_many_arguments, clippy::type_complexity)]

use crate::app_2d::{
    plan_world_render_layers, CurrentDeck, DeckContentEntities, DeckWalkGrids, GamePlanCamera2d,
};
use crate::cell::{Fixture, FloorMaterial, SideMaterial};
use crate::cell_box;
use crate::cell_box::{CellIndex, PlanKey};
use crate::deck_layout::DeckLayouts;
use crate::load_screen::GamePhase;
use crate::plan_mesh::{rebuild_plan_deck_meshes, DeckPlanMeshes};
use crate::shared::cursor_in_game_viewport;
use bevy::prelude::*;
use std::collections::HashSet;

const PANEL_WIDTH_PX: f32 = 300.0;
const FIELD_ROW_GAP: f32 = 6.0;
const PICKER_ROW_GAP: f32 = 2.0;
const Z_HIGHLIGHT: f32 = 0.002;
const Z_BOX_SELECT: f32 = 0.001;
const DRAG_CLICK_THRESHOLD_PX: f32 = 4.0;

#[derive(Resource, Default)]
pub struct PlanEditMode {
    pub active: bool,
}

#[derive(Resource, Default)]
pub struct SelectedPlanCells(pub HashSet<PlanKey>);

#[derive(Resource, Default)]
pub struct OpenMaterialPicker(pub Option<CellMaterialField>);

#[derive(Resource, Default)]
pub struct PlanDeckMeshDirty(pub Option<usize>);

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CellMaterialField {
    Floor,
    Side1,
    Side2,
    Side3,
    Side4,
}

impl CellMaterialField {
    const ALL: [Self; 5] = [
        Self::Floor,
        Self::Side1,
        Self::Side2,
        Self::Side3,
        Self::Side4,
    ];

    fn is_floor(self) -> bool {
        matches!(self, Self::Floor)
    }

    fn label(self) -> &'static str {
        match self {
            Self::Floor => "Floor",
            Self::Side1 => "Side 1 (+X / east)",
            Self::Side2 => "Side 2 (+Y / north)",
            Self::Side3 => "Side 3 (−X / west)",
            Self::Side4 => "Side 4 (−Y / south)",
        }
    }

    fn read_label(self, cell: &crate::cell::Cell) -> String {
        if self.is_floor() {
            cell.floor.label().to_string()
        } else {
            self.read_side(cell).label().to_string()
        }
    }

    fn read_side(self, cell: &crate::cell::Cell) -> SideMaterial {
        match self {
            Self::Side1 => cell.side1,
            Self::Side2 => cell.side2,
            Self::Side3 => cell.side3,
            Self::Side4 => cell.side4,
            Self::Floor => SideMaterial::Open,
        }
    }

    fn write_floor(self, cell: &mut crate::cell::Cell, floor: FloorMaterial) {
        if self.is_floor() {
            cell.floor = floor;
        }
    }

    fn write_side(self, cell: &mut crate::cell::Cell, side: SideMaterial) {
        match self {
            Self::Side1 => cell.side1 = side,
            Self::Side2 => cell.side2 = side,
            Self::Side3 => cell.side3 = side,
            Self::Side4 => cell.side4 = side,
            Self::Floor => {}
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum PickerChoice {
    Floor(FloorMaterial),
    Side(SideMaterial),
}

impl PickerChoice {
    fn label(self) -> &'static str {
        match self {
            Self::Floor(m) => m.label(),
            Self::Side(m) => m.label(),
        }
    }

    fn bg_color(self) -> Color {
        match self {
            Self::Floor(m) => picker_bg_from_color(m.color()),
            Self::Side(m) => picker_bg_from_color(m.picker_color()),
        }
    }

    fn apply(self, field: CellMaterialField, cell: &mut crate::cell::Cell) {
        match self {
            Self::Floor(floor) => field.write_floor(cell, floor),
            Self::Side(side) => field.write_side(cell, side),
        }
    }
}

#[derive(Clone)]
struct CellSnapshot {
    side1: SideMaterial,
    side2: SideMaterial,
    side3: SideMaterial,
    side4: SideMaterial,
    floor: FloorMaterial,
    fixtures: Vec<Fixture>,
}

impl CellSnapshot {
    fn from_cell(cell: &crate::cell::Cell) -> Self {
        Self {
            side1: cell.side1,
            side2: cell.side2,
            side3: cell.side3,
            side4: cell.side4,
            floor: cell.floor,
            fixtures: cell.fixtures.clone(),
        }
    }

    fn apply(&self, cell: &mut crate::cell::Cell) {
        cell.side1 = self.side1;
        cell.side2 = self.side2;
        cell.side3 = self.side3;
        cell.side4 = self.side4;
        cell.floor = self.floor;
        cell.fixtures = self.fixtures.clone();
    }
}

fn format_fixtures_summary(fixtures: &[Fixture]) -> String {
    if fixtures.is_empty() {
        return "none".to_string();
    }
    let mut counts = [0u32; 3];
    for fixture in fixtures {
        match fixture {
            Fixture::Bed(_) => counts[0] += 1,
            Fixture::Shower(_) => counts[1] += 1,
            Fixture::Toilet(_) => counts[2] += 1,
        }
    }
    let mut parts = Vec::new();
    if counts[0] > 0 {
        parts.push(format!("Bed×{}", counts[0]));
    }
    if counts[1] > 0 {
        parts.push(format!("Shower×{}", counts[1]));
    }
    if counts[2] > 0 {
        parts.push(format!("Toilet×{}", counts[2]));
    }
    parts.join(", ")
}

#[derive(Resource, Default)]
struct PlanCellClipboard {
    entries: Vec<(i32, i32, CellSnapshot)>,
}

#[derive(Resource, Default)]
struct BoxSelectDrag {
    screen_start: Option<Vec2>,
    hull_start: Option<Vec2>,
}

#[derive(Component)]
struct EditModePanelRoot;

#[derive(Component)]
struct EditModeBannerText;

#[derive(Component)]
struct EditModeCellSummaryText;

/// Marks a material-field row in the edit panel (field identity lives on child widgets).
#[derive(Component)]
struct MaterialFieldRow;

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
    choice: PickerChoice,
}

#[derive(Component)]
struct SelectedCellHighlight;

#[derive(Component)]
struct BoxSelectRect;

pub struct PlanEditPlugin;

impl Plugin for PlanEditPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PlanEditMode>()
            .init_resource::<SelectedPlanCells>()
            .init_resource::<OpenMaterialPicker>()
            .init_resource::<PlanDeckMeshDirty>()
            .init_resource::<PlanCellClipboard>()
            .init_resource::<BoxSelectDrag>()
            .add_systems(
                Update,
                (
                    toggle_plan_edit_mode,
                    clear_selection_on_deck_change,
                    handle_box_select_input,
                    update_box_select_rect_visual,
                    clipboard_hotkeys,
                    sync_edit_mode_panel,
                    handle_material_dropdown_buttons,
                    handle_material_picker_options,
                    rebuild_dirty_plan_deck_meshes,
                    sync_selected_cell_highlights,
                )
                    .run_if(in_state(GamePhase::InGame)),
            );
    }
}

/// Occupied plan keys on `deck` inside the inclusive rectangle `[min_plan, max_plan]`.
#[must_use]
pub fn plan_keys_in_rect(
    layouts: &DeckLayouts,
    deck: usize,
    min_plan: PlanKey,
    max_plan: PlanKey,
) -> HashSet<PlanKey> {
    let min_x = min_plan.0.min(max_plan.0);
    let max_x = min_plan.0.max(max_plan.0);
    let min_y = min_plan.1.min(max_plan.1);
    let max_y = min_plan.1.max(max_plan.1);
    layouts
        .cells
        .iter_deck(deck as u8)
        .filter_map(|(idx, _)| {
            let plan = idx.plan();
            (plan.0 >= min_x && plan.0 <= max_x && plan.1 >= min_y && plan.1 <= max_y)
                .then_some(plan)
        })
        .collect()
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
                Text::new(
                    "Press E to enter edit mode.\nClick or drag on the plan to select cells.",
                ),
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
            MaterialFieldRow,
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
                let choices: Vec<PickerChoice> = if field.is_floor() {
                    FloorMaterial::ALL
                        .into_iter()
                        .map(PickerChoice::Floor)
                        .collect()
                } else {
                    SideMaterial::ALL
                        .into_iter()
                        .map(PickerChoice::Side)
                        .collect()
                };
                for choice in choices {
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
                            BackgroundColor(choice.bg_color()),
                            MaterialPickerOption { field, choice },
                        ))
                        .with_children(|opt| {
                            opt.spawn((
                                Text::new(choice.label()),
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

fn picker_bg_from_color(color: Color) -> Color {
    let c: LinearRgba = color.into();
    Color::linear_rgba(
        c.red * 0.35 + 0.08,
        c.green * 0.35 + 0.08,
        c.blue * 0.35 + 0.08,
        0.95,
    )
}

fn ctrl_pressed(keyboard: &ButtonInput<KeyCode>) -> bool {
    keyboard.pressed(KeyCode::ControlLeft) || keyboard.pressed(KeyCode::ControlRight)
}

fn pointer_over_edit_panel(window: &Window, edit_mode: &PlanEditMode) -> bool {
    if !edit_mode.active {
        return false;
    }
    let Some(pos) = window.cursor_position() else {
        return false;
    };
    pos.x > window.width() - PANEL_WIDTH_PX
}

fn hull_xy_under_cursor(
    window: &Window,
    cameras: &Query<(&Camera, &GlobalTransform), With<GamePlanCamera2d>>,
) -> Option<Vec2> {
    let Ok((camera, cam_tf)) = cameras.single() else {
        return None;
    };
    let cursor = cursor_in_game_viewport(window, camera)?;
    camera.viewport_to_world_2d(cam_tf, cursor).ok()
}

fn plan_rect_from_hull_corners(hull_a: Vec2, hull_b: Vec2, deck: u8) -> Option<(PlanKey, PlanKey)> {
    let idx_a = CellIndex::from_world_xy_deck(hull_a, deck)?;
    let idx_b = CellIndex::from_world_xy_deck(hull_b, deck)?;
    let min_x = idx_a.x.min(idx_b.x);
    let max_x = idx_a.x.max(idx_b.x);
    let min_y = idx_a.y.min(idx_b.y);
    let max_y = idx_a.y.max(idx_b.y);
    Some(((min_x, min_y), (max_x, max_y)))
}

fn select_single_cell(
    hull_xy: Vec2,
    deck: usize,
    layouts: &DeckLayouts,
    selected: &mut SelectedPlanCells,
    picker: &mut OpenMaterialPicker,
) {
    selected.0.clear();
    picker.0 = None;
    let Some(idx) = CellIndex::from_world_xy_deck(hull_xy, deck as u8) else {
        return;
    };
    let plan = idx.plan();
    if layouts.deck(deck).get(plan).is_some() {
        selected.0.insert(plan);
    }
}

fn toggle_plan_edit_mode(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut edit_mode: ResMut<PlanEditMode>,
    mut picker: ResMut<OpenMaterialPicker>,
    mut selected: ResMut<SelectedPlanCells>,
) {
    if keyboard.just_pressed(KeyCode::KeyE) {
        edit_mode.active = !edit_mode.active;
        if !edit_mode.active {
            picker.0 = None;
            selected.0.clear();
        }
    }
}

fn clear_selection_on_deck_change(
    current_deck: Res<CurrentDeck>,
    mut selected: ResMut<SelectedPlanCells>,
    mut picker: ResMut<OpenMaterialPicker>,
    mut last_deck: Local<Option<usize>>,
) {
    let deck = current_deck.0;
    if let Some(prev) = *last_deck {
        if prev != deck {
            selected.0.clear();
            picker.0 = None;
        }
    }
    *last_deck = Some(deck);
}

fn handle_box_select_input(
    edit_mode: Res<PlanEditMode>,
    mouse: Res<ButtonInput<MouseButton>>,
    window: Single<&Window>,
    current_deck: Res<CurrentDeck>,
    layouts: Res<DeckLayouts>,
    mut selected: ResMut<SelectedPlanCells>,
    mut picker: ResMut<OpenMaterialPicker>,
    cameras: Query<(&Camera, &GlobalTransform), With<GamePlanCamera2d>>,
    mut drag: ResMut<BoxSelectDrag>,
) {
    if !edit_mode.active {
        drag.screen_start = None;
        drag.hull_start = None;
        return;
    }

    if pointer_over_edit_panel(&window, &edit_mode) {
        return;
    }

    let Ok((camera, _)) = cameras.single() else {
        return;
    };
    if cursor_in_game_viewport(&window, camera).is_none() {
        return;
    }

    if mouse.just_pressed(MouseButton::Left) {
        let cursor = window.cursor_position().unwrap_or(Vec2::ZERO);
        drag.screen_start = Some(cursor);
        drag.hull_start = hull_xy_under_cursor(&window, &cameras);
        return;
    }

    if !mouse.just_released(MouseButton::Left) {
        return;
    }

    let Some(screen_start) = drag.screen_start.take() else {
        drag.hull_start = None;
        return;
    };
    let hull_start = drag.hull_start.take();
    let cursor = window.cursor_position().unwrap_or(screen_start);
    let moved = cursor.distance(screen_start);

    let Some(hull_end) = hull_xy_under_cursor(&window, &cameras) else {
        selected.0.clear();
        picker.0 = None;
        return;
    };

    let deck = current_deck.0;
    if moved < DRAG_CLICK_THRESHOLD_PX {
        if let Some(hull_start) = hull_start {
            select_single_cell(hull_start, deck, &layouts, &mut selected, &mut picker);
        } else {
            selected.0.clear();
            picker.0 = None;
        }
        return;
    }

    let Some(hull_start) = hull_start else {
        return;
    };

    let Some((min_plan, max_plan)) = plan_rect_from_hull_corners(hull_start, hull_end, deck as u8)
    else {
        selected.0.clear();
        picker.0 = None;
        return;
    };

    selected.0 = plan_keys_in_rect(&layouts, deck, min_plan, max_plan);
    picker.0 = None;
}

fn update_box_select_rect_visual(
    mut commands: Commands,
    edit_mode: Res<PlanEditMode>,
    mouse: Res<ButtonInput<MouseButton>>,
    window: Single<&Window>,
    assets: Option<Res<crate::app_2d::Plan2dVisualAssets>>,
    cameras: Query<(&Camera, &GlobalTransform), With<GamePlanCamera2d>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    drag: Res<BoxSelectDrag>,
    mut rect_entity: Local<Option<Entity>>,
    mut rect_mesh: Local<Option<Handle<Mesh>>>,
    mut rect_material: Local<Option<Handle<ColorMaterial>>>,
    mut transforms: Query<&mut Transform, With<BoxSelectRect>>,
    mut visibilities: Query<&mut Visibility, With<BoxSelectRect>>,
) {
    let mut hide = || {
        if let Some(entity) = rect_entity.take() {
            commands.entity(entity).despawn();
        }
    };

    if !edit_mode.active || !mouse.pressed(MouseButton::Left) {
        hide();
        return;
    }

    let Some(screen_start) = drag.screen_start else {
        hide();
        return;
    };

    let Some(hull_start) = drag.hull_start else {
        hide();
        return;
    };

    let cursor = window.cursor_position().unwrap_or(screen_start);
    if cursor.distance(screen_start) < DRAG_CLICK_THRESHOLD_PX {
        hide();
        return;
    }

    let Some(hull_end) = hull_xy_under_cursor(&window, &cameras) else {
        hide();
        return;
    };

    let min = hull_start.min(hull_end);
    let max = hull_start.max(hull_end);
    let center = (min + max) * 0.5;
    let size = (max - min).max(Vec2::splat(0.05));

    let Some(assets) = assets else {
        return;
    };

    if rect_entity.is_none() {
        let mesh =
            rect_mesh.get_or_insert_with(|| meshes.add(Mesh::from(Rectangle::new(1.0, 1.0))));
        let material = rect_material.get_or_insert_with(|| {
            materials.add(ColorMaterial::from(Color::srgba(0.4, 0.75, 1.0, 0.25)))
        });
        let entity = commands
            .spawn((
                Mesh2d(mesh.clone()),
                MeshMaterial2d(material.clone()),
                Transform::from_xyz(center.x, center.y, Z_BOX_SELECT)
                    .with_scale(Vec3::new(size.x, size.y, 1.0)),
                plan_world_render_layers(),
                BoxSelectRect,
            ))
            .id();
        commands.entity(assets.plan_root).add_child(entity);
        *rect_entity = Some(entity);
    }

    if let Some(entity) = *rect_entity {
        if let Ok(mut tf) = transforms.get_mut(entity) {
            tf.translation = center.extend(Z_BOX_SELECT);
            tf.scale = Vec3::new(size.x, size.y, 1.0);
        }
        if let Ok(mut vis) = visibilities.get_mut(entity) {
            *vis = Visibility::Inherited;
        }
    }
}

fn clipboard_hotkeys(
    edit_mode: Res<PlanEditMode>,
    keyboard: Res<ButtonInput<KeyCode>>,
    current_deck: Res<CurrentDeck>,
    selected: Res<SelectedPlanCells>,
    mut clipboard: ResMut<PlanCellClipboard>,
    mut layouts: ResMut<DeckLayouts>,
    mut walk_grids: ResMut<DeckWalkGrids>,
    mut dirty: ResMut<PlanDeckMeshDirty>,
    window: Single<&Window>,
    cameras: Query<(&Camera, &GlobalTransform), With<GamePlanCamera2d>>,
) {
    if !edit_mode.active || !ctrl_pressed(&keyboard) {
        return;
    }

    if keyboard.just_pressed(KeyCode::KeyC) && !selected.0.is_empty() {
        let mut min_x = u16::MAX;
        let mut min_y = u16::MAX;
        for &(x, y) in &selected.0 {
            min_x = min_x.min(x);
            min_y = min_y.min(y);
        }
        let deck_cells = layouts.deck(current_deck.0);
        clipboard.entries.clear();
        for &plan in &selected.0 {
            let Some(cell) = deck_cells.get(plan) else {
                continue;
            };
            let dx = i32::from(plan.0) - i32::from(min_x);
            let dy = i32::from(plan.1) - i32::from(min_y);
            clipboard
                .entries
                .push((dx, dy, CellSnapshot::from_cell(cell)));
        }
    }

    if keyboard.just_pressed(KeyCode::KeyV) && !clipboard.entries.is_empty() {
        let Some(hull_xy) = hull_xy_under_cursor(&window, &cameras) else {
            return;
        };
        let deck_i = current_deck.0;
        let Some(anchor_idx) = CellIndex::from_world_xy_deck(hull_xy, deck_i as u8) else {
            return;
        };
        let anchor = anchor_idx.plan();
        let mut changed = false;
        for &(dx, dy, ref snapshot) in &clipboard.entries {
            let tx = i32::from(anchor.0) + dx;
            let ty = i32::from(anchor.1) + dy;
            if tx < 0 || ty < 0 {
                continue;
            }
            let target = (tx as u16, ty as u16);
            let Some(index) = CellIndex::with_plan(deck_i as u8, target) else {
                continue;
            };
            let Some(cell) = layouts.cell_mut(index) else {
                continue;
            };
            snapshot.apply(cell);
            changed = true;
        }
        if changed {
            dirty.0 = Some(deck_i);
            walk_grids.0[deck_i] = crate::cell_box::deck_walk_grid(&layouts.cells, deck_i as u8);
        }
    }
}

fn sync_edit_mode_panel(
    edit_mode: Res<PlanEditMode>,
    selected: Res<SelectedPlanCells>,
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

    let selection_active = edit_mode.active && !selected.0.is_empty();
    for (_, mut node) in &mut field_rows {
        node.display = if selection_active {
            Display::Flex
        } else {
            Display::None
        };
    }

    for mut text in &mut summary {
        text.0 = cell_summary_text(edit_mode.active, &selected.0, current_deck.0, &layouts);
    }

    if !selection_active {
        for (_, mut node) in &mut picker_panels {
            node.display = Display::None;
        }
        return;
    }

    let deck_cells = layouts.deck(current_deck.0);
    let primary = selected.0.iter().copied().min_by_key(|&(x, y)| (x, y));
    let Some(plan) = primary else {
        return;
    };
    let Some(cell) = deck_cells.get(plan) else {
        return;
    };

    for (label, mut text) in &mut dropdown_labels {
        text.0 = label.field.read_label(cell);
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
    selected: &HashSet<PlanKey>,
    deck_index: usize,
    layouts: &DeckLayouts,
) -> String {
    if !edit_active {
        return "Press E to enter edit mode.\nClick or drag on the plan to select cells."
            .to_string();
    }
    if selected.is_empty() {
        return "Edit mode active.\nClick or drag on the plan to select cells.".to_string();
    }
    if selected.len() > 1 {
        return format!(
            "{} cells selected.\nCtrl+C to copy, Ctrl+V to paste at cursor.",
            selected.len()
        );
    }
    let plan = *selected.iter().next().expect("non-empty");
    let deck_cells = layouts.deck(deck_index);
    let Some(cell) = deck_cells.get(plan) else {
        return format!(
            "Cell ({}, {}) is outside the hull on this deck.",
            plan.0, plan.1
        );
    };
    let centre = deck_cells.index(plan).to_world_xy();
    let fixtures_line = format_fixtures_summary(&cell.fixtures);
    let entity_count = layouts
        .entities_at((plan.0, plan.1, deck_index as u8))
        .count();
    format!(
        "Selected: cell ({}, {}) · deck {}\nCentre ({:.1}, {:.1}) m\nFloor: {}\nFixtures: {fixtures_line}\nEntities here: {entity_count}",
        plan.0,
        plan.1,
        deck_index + 1,
        centre.x,
        centre.y,
        cell.floor.label(),
    )
}

fn handle_material_dropdown_buttons(
    edit_mode: Res<PlanEditMode>,
    selected: Res<SelectedPlanCells>,
    mut picker: ResMut<OpenMaterialPicker>,
    mut interactions: Query<
        (&Interaction, &MaterialDropdownButton),
        (Changed<Interaction>, With<Button>),
    >,
) {
    if !edit_mode.active || selected.0.is_empty() {
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
    selected: Res<SelectedPlanCells>,
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
    if !edit_mode.active || selected.0.is_empty() {
        return;
    }
    let deck_i = current_deck.0;

    for (interaction, option) in &mut interactions {
        if *interaction != Interaction::Pressed {
            continue;
        }
        if picker.0 != Some(option.field) {
            continue;
        }
        let mut changed = false;
        for &plan in &selected.0 {
            let Some(index) = CellIndex::with_plan(deck_i as u8, plan) else {
                continue;
            };
            let Some(cell) = layouts.cell_mut(index) else {
                continue;
            };
            option.choice.apply(option.field, cell);
            changed = true;
        }
        if changed {
            dirty.0 = Some(deck_i);
            walk_grids.0[deck_i] = crate::cell_box::deck_walk_grid(&layouts.cells, deck_i as u8);
        }
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

fn sync_selected_cell_highlights(
    mut commands: Commands,
    edit_mode: Res<PlanEditMode>,
    selected: Res<SelectedPlanCells>,
    current_deck: Res<CurrentDeck>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    deck_entities: Res<DeckContentEntities>,
    mut highlights: Local<std::collections::HashMap<PlanKey, Entity>>,
    mut mesh_handle: Local<Option<Handle<Mesh>>>,
    mut material_handle: Local<Option<Handle<ColorMaterial>>>,
    parents: Query<&ChildOf>,
) {
    let deck_i = current_deck.0;
    let expected_parent = deck_entities.0[deck_i];

    let target_keys: HashSet<PlanKey> = if edit_mode.active {
        selected.0.clone()
    } else {
        HashSet::new()
    };

    let to_despawn: Vec<(PlanKey, Entity)> = highlights
        .iter()
        .filter(|(key, entity)| {
            !target_keys.contains(key)
                || parents
                    .get(**entity)
                    .is_ok_and(|p| p.parent() != expected_parent)
        })
        .map(|(k, e)| (*k, *e))
        .collect();
    for (key, entity) in to_despawn {
        commands.entity(entity).despawn();
        highlights.remove(&key);
    }

    let mesh = mesh_handle.get_or_insert_with(|| {
        meshes.add(Mesh::from(Rectangle::new(
            cell_box::beam_cell_m(),
            cell_box::length_cell_m(),
        )))
    });
    let material = material_handle.get_or_insert_with(|| {
        materials.add(ColorMaterial::from(Color::srgba(1.0, 0.92, 0.2, 0.45)))
    });

    for &plan in &target_keys {
        if highlights.contains_key(&plan) {
            continue;
        }
        let Some(pos) = CellIndex::with_plan(deck_i as u8, plan).map(CellIndex::to_world_xy) else {
            continue;
        };
        let entity = commands
            .spawn((
                Mesh2d(mesh.clone()),
                MeshMaterial2d(material.clone()),
                Transform::from_xyz(pos.x, pos.y, Z_HIGHLIGHT),
                plan_world_render_layers(),
                SelectedCellHighlight,
            ))
            .id();
        commands.entity(expected_parent).add_child(entity);
        highlights.insert(plan, entity);
    }
}
