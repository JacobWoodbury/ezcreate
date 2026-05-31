use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPrimaryContextPass, egui};

use crate::{
    components::{FacePaintDecal, PlacedBlock},
    content::{
        delete_user_module, rename_user_module, LibraryCatalog, LibraryItemRef,
        register_grouped_module,
    },
    resources::{
        set_game_mode, AppScreen, GameMode, GameModeChanged, GamePreferences, GridConfig,
        KeyBindings, PaintState, PlacementState, RecentPicks, SelectionState, StampPainter,
        UndoStack,
    },
    systems::{
        paint::delete_face_decal_with_undo,
        thumbnails::{library_item_cache_key, ThumbnailCache},
    },
    ui::{
        ftue::draw_ftue,
        input_capture::UiInputCapture,
        launch_menu::draw_launch_menu,
        settings::{draw_settings_window, SettingsUiState},
    },
};

#[derive(SystemParam)]
struct HudWorldActions<'w, 's> {
    commands: Commands<'w, 's>,
    undo: ResMut<'w, UndoStack>,
    blocks: Query<'w, 's, (Entity, &'static PlacedBlock, &'static GlobalTransform)>,
    decals: Query<'w, 's, (Entity, &'static FacePaintDecal)>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum LibraryRowAction {
    Select,
    Delete,
    StartRename,
}

#[derive(Clone)]
struct RenameTarget {
    mod_id: String,
    item_id: String,
    buffer: String,
}

#[derive(Resource, Default)]
struct UiState {
    show_settings: bool,
    settings: SettingsUiState,
    stamp_save_name: String,
    stamp_name_error: Option<String>,
    rename: Option<RenameTarget>,
    rename_error: Option<String>,
    request_tutorial: bool,
}

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<UiState>()
            .add_systems(EguiPrimaryContextPass, draw_launch_menu_system)
            .add_systems(EguiPrimaryContextPass, draw_ftue_system)
            .add_systems(EguiPrimaryContextPass, draw_hud.run_if(on_playing))
            .add_systems(
                EguiPrimaryContextPass,
                apply_ui_navigation.after(draw_hud),
            )
            .add_systems(
                EguiPrimaryContextPass,
                sync_ui_input_capture.after(apply_ui_navigation),
            );
    }
}

fn on_playing(screen: Res<AppScreen>) -> bool {
    matches!(*screen, AppScreen::Playing)
}

fn draw_launch_menu_system(
    mut contexts: EguiContexts,
    mut screen: ResMut<AppScreen>,
    exit: MessageWriter<AppExit>,
) {
    if !matches!(*screen, AppScreen::LaunchMenu) {
        return;
    }
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };
    draw_launch_menu(ctx, screen.as_mut(), exit);
}

fn draw_ftue_system(
    mut contexts: EguiContexts,
    mut screen: ResMut<AppScreen>,
    bindings: Res<KeyBindings>,
) {
    if !matches!(*screen, AppScreen::Ftue { .. }) {
        return;
    }
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };
    draw_ftue(ctx, screen.as_mut(), &bindings);
}

fn apply_ui_navigation(mut screen: ResMut<AppScreen>, mut ui_state: ResMut<UiState>) {
    if ui_state.request_tutorial {
        ui_state.request_tutorial = false;
        AppScreen::start_ftue(screen.as_mut());
    }
}

fn sync_ui_input_capture(
    mut contexts: EguiContexts,
    mut capture: ResMut<UiInputCapture>,
    screen: Res<AppScreen>,
    ui_state: Res<UiState>,
) {
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };
    UiInputCapture::sync(ctx, &screen, ui_state.show_settings, &mut capture);
}

fn draw_hud(
    mut contexts: EguiContexts,
    mut mode: ResMut<GameMode>,
    mut mode_events: MessageWriter<GameModeChanged>,
    mut grid: ResMut<GridConfig>,
    mut placement: ResMut<PlacementState>,
    mut paint: ResMut<PaintState>,
    mut stamp_painter: ResMut<StampPainter>,
    mut recent: ResMut<RecentPicks>,
    mut prefs: ResMut<GamePreferences>,
    mut bindings: ResMut<KeyBindings>,
    keys: Res<ButtonInput<KeyCode>>,
    selection: Res<SelectionState>,
    mut catalog: ResMut<LibraryCatalog>,
    mut thumbnails: ResMut<ThumbnailCache>,
    mut ui_state: ResMut<UiState>,
    mut world: HudWorldActions,
) {
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };

    // ── Top bar ────────────────────────────────────────────────────────────────
    egui::TopBottomPanel::top("top_bar").show(ctx, |ui| {
        ui.horizontal(|ui| {
            ui.label("ezcreate");
            ui.separator();
            if ui.selectable_label(*mode == GameMode::Place, "Place").clicked() {
                set_game_mode(&mut mode, &mut mode_events, GameMode::Place);
            }
            if ui.selectable_label(*mode == GameMode::Select, "Select").clicked() {
                set_game_mode(&mut mode, &mut mode_events, GameMode::Select);
            }
            if ui.selectable_label(*mode == GameMode::Paint, "Paint").clicked() {
                set_game_mode(&mut mode, &mut mode_events, GameMode::Paint);
            }
            ui.separator();
            ui.checkbox(&mut grid.prevent_overlapping, "Prevent overlap");
            if *mode == GameMode::Select {
                ui.label(format!("selected: {}", selection.selected.len()));
            }
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("⚙ Settings").clicked() {
                    ui_state.show_settings = !ui_state.show_settings;
                }
                if ui.button("Help").clicked() {
                    ui_state.request_tutorial = true;
                }
            });
        });
    });

    // ── Settings window ────────────────────────────────────────────────────────
    if ui_state.show_settings {
        let mut open = ui_state.show_settings;
        egui::Window::new("Settings")
            .open(&mut open)
            .collapsible(false)
            .resizable(false)
            .default_width(360.0)
            .show(ctx, |ui| {
                draw_settings_window(
                    ui,
                    &mut grid,
                    &mut prefs,
                    &mut bindings,
                    &mut ui_state.settings,
                    &keys,
                );
            });
        ui_state.show_settings = open;
    }

    let process_item = |ui: &mut egui::Ui,
                            ui_state: &mut UiState,
                            item: &LibraryItemRef,
                            placement: &mut PlacementState,
                            recent: &mut RecentPicks,
                            catalog: &mut LibraryCatalog,
                            thumbnails: &mut ThumbnailCache|
     -> Option<LibraryRowAction> {
        if ui_state
            .rename
            .as_ref()
            .is_some_and(|r| r.mod_id == item.mod_id && r.item_id == item.item_id)
        {
            match draw_library_item_renaming(ui, item, ui_state, thumbnails) {
                RenameRowAction::Commit => {
                    if let Some(rename) = ui_state.rename.take() {
                        if let Err(e) = rename_user_module(item, &rename.buffer, catalog) {
                            ui_state.rename_error = Some(e);
                            ui_state.rename = Some(rename);
                        } else {
                            ui_state.rename_error = None;
                            let name = rename.buffer.trim().to_string();
                            for r in recent.items.iter_mut() {
                                if r.mod_id == item.mod_id && r.item_id == item.item_id {
                                    r.display_name = name.clone();
                                }
                            }
                            if placement.selected_item.as_ref().is_some_and(|s| {
                                s.mod_id == item.mod_id && s.item_id == item.item_id
                            }) {
                                if let Some(sel) = placement.selected_item.as_mut() {
                                    sel.display_name = name;
                                }
                            }
                        }
                    }
                }
                RenameRowAction::Cancel => {
                    ui_state.rename = None;
                    ui_state.rename_error = None;
                }
                RenameRowAction::None => {}
            }
            return None;
        }

        let selected = placement
            .selected_item
            .as_ref()
            .is_some_and(|s| s.item_id == item.item_id && s.mod_id == item.mod_id);

        match draw_library_item(ui, item, selected, item.is_user_deletable(), thumbnails) {
            Some(LibraryRowAction::Select) => Some(LibraryRowAction::Select),
            Some(LibraryRowAction::Delete) => Some(LibraryRowAction::Delete),
            Some(LibraryRowAction::StartRename) => {
                ui_state.rename = Some(RenameTarget {
                    mod_id: item.mod_id.clone(),
                    item_id: item.item_id.clone(),
                    buffer: item.display_name.clone(),
                });
                ui_state.rename_error = None;
                None
            }
            None => None,
        }
    };

    // ── Left sidebar (library) ────────────────────────────────────────────────
    egui::SidePanel::left("library").default_width(260.0).show(ctx, |ui| {
        egui::ScrollArea::vertical().show(ui, |ui| {
            let mut module_to_delete: Option<LibraryItemRef> = None;

            if !recent.items.is_empty() {
                ui.heading("Recent");
                let recent_list = recent.items.clone();
                for item in recent_list {
                    match process_item(
                        ui,
                        &mut ui_state,
                        &item,
                        &mut placement,
                        &mut recent,
                        &mut catalog,
                        &mut thumbnails,
                    ) {
                        Some(LibraryRowAction::Select) => {
                            placement.selected_item = Some(item.clone());
                            placement.active_section = None;
                            placement.snap_placement_euler();
                            recent.push(item.clone());
                            set_game_mode(&mut mode, &mut mode_events, GameMode::Place);
                        }
                        Some(LibraryRowAction::Delete) => module_to_delete = Some(item),
                        Some(LibraryRowAction::StartRename) | None => {}
                    }
                }
                ui.separator();
            }

            ui.heading("Library");
            if catalog.items.is_empty() {
                ui.label("No mod.json items found under assets/mods");
            } else {
                for item in catalog.items.clone() {
                    match process_item(
                        ui,
                        &mut ui_state,
                        &item,
                        &mut placement,
                        &mut recent,
                        &mut catalog,
                        &mut thumbnails,
                    ) {
                        Some(LibraryRowAction::Select) => {
                            placement.selected_item = Some(item.clone());
                            placement.active_section = None;
                            placement.snap_placement_euler();
                            recent.push(item.clone());
                            set_game_mode(&mut mode, &mut mode_events, GameMode::Place);
                        }
                        Some(LibraryRowAction::Delete) => module_to_delete = Some(item),
                        Some(LibraryRowAction::StartRename) | None => {}
                    }
                }
            }

            if let Some(item) = module_to_delete {
                if let Err(err) = delete_user_module(&item, &mut catalog) {
                    warn!("Delete module failed: {err}");
                } else {
                    recent.items.retain(|i| i.item_id != item.item_id || i.mod_id != item.mod_id);
                    let cache_key = library_item_cache_key(&item);
                    thumbnails.images.remove(&cache_key);
                    thumbnails.colors.remove(&cache_key);
                    thumbnails.texture_handles.remove(&cache_key);
                    if placement
                        .selected_item
                        .as_ref()
                        .is_some_and(|s| s.item_id == item.item_id && s.mod_id == item.mod_id)
                    {
                        placement.selected_item = None;
                        placement.active_section = None;
                    }
                }
            }

            if *mode == GameMode::Select {
                ui.separator();
                ui.heading("Blueprint");
                ui.label("Save selection as reusable module (section JSON).");
                if ui.button("Save selection as module").clicked() {
                    let name = format!("Module {}", selection.selected.len());
                    match register_grouped_module(
                        &selection,
                        &world.blocks,
                        &world.decals,
                        &grid,
                        &name,
                        &mut catalog,
                    ) {
                        Ok(item) => {
                            recent.push(item);
                        }
                        Err(err) => {
                            warn!("Save module failed: {err}");
                        }
                    }
                }
            }

            if *mode == GameMode::Paint {
                ui.separator();
                draw_stamp_editor(ui, &mut stamp_painter, &mut paint, &mut ui_state);
                draw_painted_faces_panel(
                    ui,
                    &selection,
                    &world.decals,
                    &mut world.commands,
                    &mut world.undo,
                    &grid,
                );
            }
        });
    });

    // ── Bottom bar ─────────────────────────────────────────────────────────────
    egui::TopBottomPanel::bottom("bottom_bar").show(ctx, |ui| {
        ui.horizontal(|ui| {
            ui.label(format!("Mode: {}", mode.label()));
            match *mode {
                GameMode::Place => {
                    ui.separator();
                    if ui.button("rY +90°").clicked() {
                        placement.rotate_yaw_forward();
                    }
                    if ui.button("rY -90°").clicked() {
                        placement.rotate_yaw_reverse();
                    }
                    ui.label(format!(
                        "yaw: {:.0}°",
                        placement.placement_euler.y.to_degrees()
                    ));
                    if placement.selected_item.as_ref().and_then(|i| i.section_spec_path.as_deref()).is_some() {
                        ui.separator();
                        ui.label("📦 section");
                    }
                }
                GameMode::Select => {
                    ui.separator();
                    ui.label("LMB click/drag select · Del delete · Q/E rotate");
                }
                GameMode::Paint => {
                    ui.separator();
                    if stamp_painter.apply_mode {
                        ui.label("LMB applies stamp grid · Clear = solid brush color");
                        if ui.small_button("Edit stamp").clicked() {
                            stamp_painter.apply_mode = false;
                        }
                    } else {
                        ui.label("Edit stamp grid in sidebar");
                        if ui.small_button("Apply to blocks").clicked() {
                            stamp_painter.apply_mode = true;
                        }
                    }
                }
            }
            ui.separator();
            ui.label("Alt+RMB delete · Ctrl+Z/Y undo");
        });
    });

    draw_marquee_overlay(ctx, &mode, &selection);
}

/// Stamp editor panel — shown in the sidebar when in Paint mode.
fn draw_stamp_editor(
    ui: &mut egui::Ui,
    stamp_painter: &mut StampPainter,
    paint: &mut PaintState,
    ui_state: &mut UiState,
) {
    ui.heading("Stamp editor");

    // Mode toggle.
    ui.horizontal(|ui| {
        if ui.selectable_label(!stamp_painter.apply_mode, "✏ Edit").clicked() {
            stamp_painter.apply_mode = false;
        }
        if ui.selectable_label(stamp_painter.apply_mode, "🖌 Apply").clicked() {
            stamp_painter.apply_mode = true;
        }
    });

    ui.add_space(4.0);

    // Brush color picker.
    ui.horizontal(|ui| {
        ui.label("Brush color:");
        let [r, g, b, a] = stamp_painter.brush_color;
        let mut egui_color = egui::Color32::from_rgba_unmultiplied(r, g, b, a);
        if ui.color_edit_button_srgba(&mut egui_color).changed() {
            stamp_painter.brush_color =
                [egui_color.r(), egui_color.g(), egui_color.b(), egui_color.a()];
            paint.brush_color = stamp_painter.brush_color_bevy();
        }
    });

    // Stamp size control.
    ui.horizontal(|ui| {
        ui.label("Size:");
        let mut n = stamp_painter.stamp.width as i32;
        if ui.add(egui::DragValue::new(&mut n).range(1..=16).speed(0.1)).changed() {
            let n = n as usize;
            stamp_painter.stamp.resize(n, n);
        }
        ui.label(format!("{}×{}", stamp_painter.stamp.width, stamp_painter.stamp.height));
    });

    ui.add_space(4.0);
    ui.label("Click cells to paint the stamp. Apply mode stamps this grid onto block faces.");
    ui.small("Clear all pixels to use brush color as a solid fill instead.");

    // The pixel grid.
    let cols = stamp_painter.stamp.width;
    let rows = stamp_painter.stamp.height;
    let cell = (220.0 / cols as f32).min(32.0).max(8.0);

    egui::Grid::new("stamp_grid")
        .spacing([1.0, 1.0])
        .show(ui, |ui| {
            for row in 0..rows {
                for col in 0..cols {
                    let [r, g, b, a] = stamp_painter.stamp.get(col, row);
                    let color = egui::Color32::from_rgba_unmultiplied(r, g, b, a);
                    let (rect, response) = ui.allocate_exact_size(
                        egui::vec2(cell, cell),
                        egui::Sense::click(),
                    );
                    if ui.is_rect_visible(rect) {
                        ui.painter().rect_filled(rect, 2.0, color);
                        ui.painter().rect_stroke(
                            rect,
                            2.0,
                            egui::Stroke::new(0.5, egui::Color32::from_gray(60)),
                            egui::StrokeKind::Inside,
                        );
                    }
                    if response.clicked() {
                        stamp_painter.stamp.set(col, row, stamp_painter.brush_color);
                    }
                }
                ui.end_row();
            }
        });

    ui.add_space(4.0);

    // Fill / clear controls.
    ui.horizontal(|ui| {
        if ui.button("Fill all").clicked() {
            let color = stamp_painter.brush_color;
            for px in &mut stamp_painter.stamp.pixels {
                *px = color;
            }
        }
        if ui.button("Clear").clicked() {
            for px in &mut stamp_painter.stamp.pixels {
                *px = [0, 0, 0, 0];
            }
        }
    });

    ui.separator();

    // Save stamp.
    ui.horizontal(|ui| {
        ui.text_edit_singleline(&mut ui_state.stamp_save_name);
        if ui.button("Save").clicked() {
            if ui_state.stamp_save_name.trim().is_empty() {
                ui_state.stamp_name_error = Some("Enter a name.".into());
            } else {
                match stamp_painter.save_stamp(ui_state.stamp_save_name.trim()) {
                    Ok(()) => {
                        ui_state.stamp_name_error = None;
                        ui_state.stamp_save_name.clear();
                    }
                    Err(e) => {
                        ui_state.stamp_name_error = Some(e);
                    }
                }
            }
        }
        if ui.button("Reload saved").clicked() {
            stamp_painter.reload_stamps();
        }
    });
    if let Some(ref err) = ui_state.stamp_name_error {
        ui.colored_label(egui::Color32::RED, err);
    }

    // Saved stamp list.
    if !stamp_painter.saved_stamps.is_empty() {
        ui.label("Saved stamps:");
        let saved = stamp_painter.saved_stamps.clone();
        let mut stamp_to_delete: Option<String> = None;
        for (name, saved_stamp) in saved {
            ui.horizontal(|ui| {
                if ui.selectable_label(false, &name).clicked() {
                    stamp_painter.stamp = saved_stamp;
                }
                if trash_icon_button(ui, "Delete stamp").clicked() {
                    stamp_to_delete = Some(name);
                }
            });
        }
        if let Some(name) = stamp_to_delete {
            if let Err(e) = stamp_painter.delete_saved_stamp(&name) {
                ui_state.stamp_name_error = Some(e);
            }
        }
    }
}

/// Painted faces on the current selection (Paint mode).
fn draw_painted_faces_panel(
    ui: &mut egui::Ui,
    selection: &SelectionState,
    decals: &Query<(Entity, &FacePaintDecal)>,
    commands: &mut Commands,
    undo: &mut UndoStack,
    grid: &GridConfig,
) {
    if selection.selected.is_empty() {
        return;
    }

    let mut painted: Vec<(Entity, FacePaintDecal)> = Vec::new();
    for (entity, decal) in decals.iter() {
        if selection.selected.contains(&decal.parent_block) {
            painted.push((entity, decal.clone()));
        }
    }

    if painted.is_empty() {
        return;
    }

    ui.separator();
    ui.heading("Painted faces");
    ui.small("On selected blocks");

    let mut decal_to_delete: Option<Entity> = None;
    painted.sort_by(|a, b| {
        a.1.parent_block
            .index()
            .cmp(&b.1.parent_block.index())
            .then(face_normal_sort_key(a.1.face_normal).cmp(&face_normal_sort_key(b.1.face_normal)))
    });

    for (entity, decal) in painted {
        let kind_label = match &decal.kind {
            crate::resources::FacePaintKind::Solid => "solid",
            crate::resources::FacePaintKind::Stamp { .. } => "stamp",
        };
        let label = format!("{} · {}", face_normal_label(decal.face_normal), kind_label);
        ui.horizontal(|ui| {
            let [r, g, b, a] = {
                let s = decal.color.to_srgba();
                [
                    (s.red * 255.0) as u8,
                    (s.green * 255.0) as u8,
                    (s.blue * 255.0) as u8,
                    (s.alpha * 255.0) as u8,
                ]
            };
            ui.colored_label(
                egui::Color32::from_rgba_unmultiplied(r, g, b, a),
                "■",
            );
            ui.label(&label);
            if trash_icon_button(ui, "Remove face paint").clicked() {
                decal_to_delete = Some(entity);
            }
        });
    }

    if let Some(entity) = decal_to_delete {
        if let Ok((_, decal)) = decals.get(entity) {
            delete_face_decal_with_undo(commands, undo, grid, entity, decal);
        }
    }
}

fn face_normal_label(n: Vec3) -> &'static str {
    if n.y > 0.9 {
        "+Y"
    } else if n.y < -0.9 {
        "-Y"
    } else if n.x > 0.9 {
        "+X"
    } else if n.x < -0.9 {
        "-X"
    } else if n.z > 0.9 {
        "+Z"
    } else {
        "-Z"
    }
}

fn face_normal_sort_key(n: Vec3) -> (i32, i32, i32) {
    (
        (n.x * 10.0).round() as i32,
        (n.y * 10.0).round() as i32,
        (n.z * 10.0).round() as i32,
    )
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum RenameRowAction {
    None,
    Commit,
    Cancel,
}

fn trash_icon_button(ui: &mut egui::Ui, tip: &str) -> egui::Response {
    ui.add(
        egui::Button::new(egui::RichText::new("🗑").size(14.0))
            .frame(false)
            .min_size(egui::vec2(20.0, 20.0)),
    )
    .on_hover_text(tip)
}

fn rename_icon_button(ui: &mut egui::Ui, tip: &str) -> egui::Response {
    ui.add(
        egui::Button::new(egui::RichText::new("✎").size(14.0))
            .frame(false)
            .min_size(egui::vec2(20.0, 20.0)),
    )
    .on_hover_text(tip)
}

fn draw_library_item_renaming(
    ui: &mut egui::Ui,
    item: &LibraryItemRef,
    ui_state: &mut UiState,
    thumbnails: &mut ThumbnailCache,
) -> RenameRowAction {
    const THUMB: f32 = 40.0;
    let Some(rename) = ui_state.rename.as_mut() else {
        return RenameRowAction::None;
    };

    let action = ui
        .horizontal(|ui| {
            let thumb_rect =
                ui.allocate_exact_size(egui::vec2(THUMB, THUMB), egui::Sense::hover()).1;
            if ui.is_rect_visible(thumb_rect.rect) {
                let cache_key = library_item_cache_key(item);
                let swatch = thumbnails
                    .colors
                    .get(&cache_key)
                    .copied()
                    .unwrap_or([80, 80, 80, 255]);
                ui.painter().rect_filled(
                    thumb_rect.rect,
                    4.0,
                    egui::Color32::from_rgba_unmultiplied(swatch[0], swatch[1], swatch[2], swatch[3]),
                );
            }
            let response = ui.text_edit_singleline(&mut rename.buffer);
            if ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                return RenameRowAction::Commit;
            }
            if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                return RenameRowAction::Cancel;
            }
            if response.lost_focus() {
                return RenameRowAction::Commit;
            }
            RenameRowAction::None
        })
        .inner;

    if let Some(err) = ui_state.rename_error.as_ref() {
        ui.colored_label(egui::Color32::RED, err);
    }

    action
}

/// Renders a library item row with thumbnail, label, and optional delete control.
fn draw_library_item(
    ui: &mut egui::Ui,
    item: &LibraryItemRef,
    selected: bool,
    deletable: bool,
    thumbnails: &mut ThumbnailCache,
) -> Option<LibraryRowAction> {
    const THUMB: f32 = 40.0;
    const TRASH_WIDTH: f32 = 28.0;
    const RENAME_WIDTH: f32 = 28.0;
    let row_height = THUMB + 4.0;

    let mut action = None;
    let chrome_width = if deletable {
        TRASH_WIDTH + RENAME_WIDTH
    } else {
        0.0
    };

    ui.horizontal(|ui| {
        let row_width = (ui.available_width() - chrome_width).max(40.0);
        let bg = if selected {
            egui::Color32::from_rgba_unmultiplied(90, 160, 255, 60)
        } else {
            egui::Color32::TRANSPARENT
        };

        let (rect, response) = ui.allocate_exact_size(
            egui::vec2(row_width, row_height),
            egui::Sense::click(),
        );

        if ui.is_rect_visible(rect) {
            ui.painter().rect_filled(rect, 4.0, bg);

            let thumb_rect = egui::Rect::from_min_size(
                rect.min + egui::vec2(4.0, 2.0),
                egui::vec2(THUMB, THUMB),
            );
            let cache_key = library_item_cache_key(item);
            if let Some(color_image) = thumbnails.images.get(&cache_key).cloned() {
                let handle = thumbnails
                    .texture_handles
                    .entry(cache_key.clone())
                    .or_insert_with(|| {
                        ui.ctx().load_texture(
                            cache_key.clone(),
                            color_image,
                            egui::TextureOptions::LINEAR,
                        )
                    });
                ui.painter().image(
                    handle.id(),
                    thumb_rect,
                    egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                    egui::Color32::WHITE,
                );
            } else {
                let swatch = thumbnails
                    .colors
                    .get(&cache_key)
                    .copied()
                    .unwrap_or([80, 80, 80, 255]);
                ui.painter().rect_filled(
                    thumb_rect,
                    4.0,
                    egui::Color32::from_rgba_unmultiplied(
                        swatch[0], swatch[1], swatch[2], swatch[3],
                    ),
                );
            }

            let label = if item.section_spec_path.is_some() {
                format!("📦 {}", item.display_name)
            } else {
                item.display_name.clone()
            };
            let text_rect = egui::Rect::from_min_size(
                rect.min + egui::vec2(THUMB + 8.0, 2.0),
                egui::vec2(rect.width() - THUMB - 12.0, THUMB),
            );
            ui.painter().text(
                text_rect.left_center(),
                egui::Align2::LEFT_CENTER,
                &label,
                egui::FontId::proportional(13.0),
                ui.visuals().text_color(),
            );
        }

        if response.on_hover_text(format!("Mod: {}", item.mod_id)).clicked() {
            action = Some(LibraryRowAction::Select);
        }

        if deletable {
            if rename_icon_button(ui, "Rename module").clicked() {
                action = Some(LibraryRowAction::StartRename);
            }
            if trash_icon_button(ui, "Delete module").clicked() {
                action = Some(LibraryRowAction::Delete);
            }
        }
    });

    action
}

/// Drawn after panels; marquee coords are window logical pixels (same as Bevy cursor / world_to_viewport).
fn draw_marquee_overlay(ctx: &egui::Context, mode: &GameMode, selection: &SelectionState) {
    if *mode != GameMode::Select || !selection.marquee_dragging {
        return;
    }
    let Some(rect) = selection.marquee_rect() else {
        return;
    };
    if selection.marquee_drag_distance() < 2.0 {
        return;
    }

    let egui_rect = egui::Rect::from_min_max(
        egui::pos2(rect.min.x, rect.min.y),
        egui::pos2(rect.max.x, rect.max.y),
    );

    egui::Area::new(egui::Id::new("marquee_overlay"))
        .order(egui::Order::Tooltip)
        .interactable(false)
        .show(ctx, |ui| {
            let painter = ui.painter_at(egui_rect);
            painter.rect_filled(
                egui_rect,
                0.0,
                egui::Color32::from_rgba_unmultiplied(64, 165, 255, 70),
            );
            painter.rect_stroke(
                egui_rect,
                0.0,
                egui::Stroke::new(2.0, egui::Color32::from_rgb(90, 215, 255)),
                egui::StrokeKind::Outside,
            );
        });
}
