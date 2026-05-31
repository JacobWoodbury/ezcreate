use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPrimaryContextPass, egui};

use crate::{
    components::{FacePaintDecal, PlacedBlock},
    content::{LibraryCatalog, LibraryItemRef, register_grouped_module},
    resources::{
        set_game_mode, GameMode, GameModeChanged, GamePreferences, GridConfig, KeyBindings,
        PaintState, PlacementState, RecentPicks, SelectionState, StampPainter,
    },
    systems::thumbnails::{library_item_cache_key, ThumbnailCache},
    ui::settings::{draw_settings_window, SettingsUiState},
};

#[derive(SystemParam)]
struct ModuleSaveQueries<'w, 's> {
    blocks: Query<'w, 's, (Entity, &'static PlacedBlock, &'static GlobalTransform)>,
    decals: Query<'w, 's, &'static FacePaintDecal>,
}

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<UiState>()
            .add_systems(EguiPrimaryContextPass, draw_hud);
    }
}

#[derive(Resource, Default)]
struct UiState {
    show_settings: bool,
    settings: SettingsUiState,
    stamp_save_name: String,
    stamp_name_error: Option<String>,
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
    module_save: ModuleSaveQueries,
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

    // ── Left sidebar (library) ────────────────────────────────────────────────
    egui::SidePanel::left("library").default_width(260.0).show(ctx, |ui| {
        egui::ScrollArea::vertical().show(ui, |ui| {
            if !recent.items.is_empty() {
                ui.heading("Recent");
                let recent_list = recent.items.clone();
                for item in recent_list {
                    let selected = placement
                        .selected_item
                        .as_ref()
                        .is_some_and(|s| s.item_id == item.item_id);
                    if draw_library_item(ui, &item, selected, &mut thumbnails) {
                        placement.selected_item = Some(item.clone());
                        placement.active_section = None;
                        placement.snap_placement_euler();
                        recent.push(item.clone());
                        set_game_mode(&mut mode, &mut mode_events, GameMode::Place);
                    }
                }
                ui.separator();
            }

            ui.heading("Library");
            if catalog.items.is_empty() {
                ui.label("No mod.json items found under assets/mods");
            } else {
                for item in catalog.items.clone() {
                    let selected = placement
                        .selected_item
                        .as_ref()
                        .is_some_and(|s| s.item_id == item.item_id);
                    if draw_library_item(ui, &item, selected, &mut thumbnails) {
                        placement.selected_item = Some(item.clone());
                        placement.active_section = None;
                        placement.snap_placement_euler();
                        recent.push(item.clone());
                        set_game_mode(&mut mode, &mut mode_events, GameMode::Place);
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
                        &module_save.blocks,
                        &module_save.decals,
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
        for (name, saved_stamp) in saved {
            if ui.small_button(&name).clicked() {
                stamp_painter.stamp = saved_stamp;
            }
        }
    }
}

/// Renders a library item row with a color swatch / thumbnail + label.
/// Returns true if the row was clicked.
fn draw_library_item(
    ui: &mut egui::Ui,
    item: &LibraryItemRef,
    selected: bool,
    thumbnails: &mut ThumbnailCache,
) -> bool {
    const THUMB: f32 = 40.0;
    let bg = if selected {
        egui::Color32::from_rgba_unmultiplied(90, 160, 255, 60)
    } else {
        egui::Color32::TRANSPARENT
    };

    let (rect, response) = ui.allocate_exact_size(
        egui::vec2(ui.available_width(), THUMB + 4.0),
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
            let handle = thumbnails.texture_handles.entry(cache_key.clone()).or_insert_with(|| {
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
                egui::Color32::from_rgba_unmultiplied(swatch[0], swatch[1], swatch[2], swatch[3]),
            );
        }

        // Name + section badge.
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

    response
        .on_hover_text(format!("Mod: {}", item.mod_id))
        .clicked()
}

/// Drawn after panels so it sits on top of the 3D view; converts window pixels to egui points.
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

    let pp = ctx.pixels_per_point();
    let min = egui::pos2(rect.min.x / pp, rect.min.y / pp);
    let max = egui::pos2(rect.max.x / pp, rect.max.y / pp);
    let egui_rect = egui::Rect::from_min_max(min, max);

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
