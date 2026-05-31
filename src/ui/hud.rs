use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPrimaryContextPass, egui};

use crate::{
    content::{LibraryCatalog, register_grouped_module},
    resources::{GameMode, GridConfig, PaintState, PlacementState, RecentPicks, SelectionState},
};

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(EguiPrimaryContextPass, draw_hud);
    }
}

fn draw_hud(
    mut contexts: EguiContexts,
    mut mode: ResMut<GameMode>,
    mut grid: ResMut<GridConfig>,
    mut placement: ResMut<PlacementState>,
    mut paint: ResMut<PaintState>,
    mut recent: ResMut<RecentPicks>,
    selection: Res<SelectionState>,
    mut catalog: ResMut<LibraryCatalog>,
    blocks: Query<(Entity, &crate::components::PlacedBlock, &GlobalTransform)>,
) {
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };

    egui::TopBottomPanel::top("top_bar").show(ctx, |ui| {
        ui.horizontal(|ui| {
            ui.label("ezcreate");
            ui.separator();
            if ui.selectable_label(*mode == GameMode::Place, "Place").clicked() {
                *mode = GameMode::Place;
            }
            if ui.selectable_label(*mode == GameMode::Select, "Select").clicked() {
                *mode = GameMode::Select;
            }
            if ui.selectable_label(*mode == GameMode::Paint, "Paint").clicked() {
                *mode = GameMode::Paint;
            }
            ui.separator();
            ui.checkbox(&mut grid.prevent_overlapping, "Prevent overlap");
            if *mode == GameMode::Select {
                ui.label(format!("selected: {}", selection.selected.len()));
            }
        });
    });

    egui::SidePanel::left("library").default_width(240.0).show(ctx, |ui| {
        if !recent.items.is_empty() {
            ui.heading("Recent");
            for item in recent.items.clone() {
                let selected = placement
                    .selected_item
                    .as_ref()
                    .is_some_and(|s| s.item_id == item.item_id);
                if ui.selectable_label(selected, &item.display_name).clicked() {
                    placement.selected_item = Some(item.clone());
                    placement.snap_placement_euler();
                    recent.push(item.clone());
                    *mode = GameMode::Place;
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
                if ui.selectable_label(selected, &item.display_name).clicked() {
                    placement.selected_item = Some(item.clone());
                    placement.snap_placement_euler();
                    recent.push(item.clone());
                    *mode = GameMode::Place;
                }
                ui.label(format!("id: {}", item.item_id));
                ui.separator();
            }
        }

        if *mode == GameMode::Select {
            ui.separator();
            ui.heading("Blueprint");
            ui.label("Save selection as reusable module (section JSON).");
            if ui.button("Save selection as module").clicked() {
                let name = format!("Module {}", selection.selected.len());
                match register_grouped_module(&selection, &blocks, &grid, &name, &mut catalog) {
                    Ok(item) => {
                        recent.push(item);
                    }
                    Err(err) => {
                        warn!("Save module failed: {err}");
                    }
                }
            }
        }
    });

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
                }
                GameMode::Select => {
                    ui.separator();
                    ui.label("LMB click/drag select · Del delete · Q/E rotate");
                }
                GameMode::Paint => {
                    ui.separator();
                    let srgba = paint.brush_color.to_srgba();
                    let mut egui_color = egui::Color32::from_rgba_unmultiplied(
                        (srgba.red * 255.0) as u8,
                        (srgba.green * 255.0) as u8,
                        (srgba.blue * 255.0) as u8,
                        (srgba.alpha * 255.0) as u8,
                    );
                    if ui.color_edit_button_srgba(&mut egui_color).changed() {
                        paint.brush_color = Color::srgba(
                            egui_color.r() as f32 / 255.0,
                            egui_color.g() as f32 / 255.0,
                            egui_color.b() as f32 / 255.0,
                            egui_color.a() as f32 / 255.0,
                        );
                    }
                    ui.label("LMB paint face");
                }
            }
            ui.separator();
            ui.label("Alt+RMB delete · Ctrl+Z/Y undo");
        });
    });

    draw_marquee_overlay(ctx, &mode, &selection);
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
