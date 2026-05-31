use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPrimaryContextPass, egui};

use crate::{
    content::LibraryCatalog,
    resources::{GameMode, GridConfig, PlacementState},
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
    catalog: Res<LibraryCatalog>,
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
        });
    });

    egui::SidePanel::left("library").default_width(220.0).show(ctx, |ui| {
        ui.heading("Library");
        ui.separator();
        if catalog.items.is_empty() {
            ui.label("No mod.json items found under assets/mods");
            return;
        }
        for item in &catalog.items {
            let selected = placement
                .selected_item
                .as_ref()
                .is_some_and(|s| s.item_id == item.item_id);
            if ui.selectable_label(selected, &item.display_name).clicked() {
                placement.selected_item = Some(item.clone());
                placement.snap_placement_euler();
            }
            ui.label(format!("id: {}", item.item_id));
            ui.separator();
        }
    });

    egui::TopBottomPanel::bottom("bottom_bar").show(ctx, |ui| {
        ui.horizontal(|ui| {
            ui.label(format!("Mode: {}", mode.label()));
            if *mode == GameMode::Place {
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
            ui.separator();
            ui.label("LMB place · Alt+RMB delete · Q/E rotate · Ctrl+Z/Y undo");
        });
    });
}
