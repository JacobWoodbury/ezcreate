use bevy_egui::egui;

/// Semi-transparent full-viewport dim layer behind menu cards.
pub fn dim_fullscreen_overlay(ctx: &egui::Context, id: &str) {
    let rect = ctx.content_rect();
    egui::Area::new(egui::Id::new(id))
        .order(egui::Order::Background)
        .fixed_pos(rect.min)
        .show(ctx, |ui| {
            ui.allocate_rect(rect, egui::Sense::click());
            ui.painter()
                .rect_filled(rect, 0.0, egui::Color32::from_rgba_unmultiplied(0, 0, 0, 160));
        });
}
