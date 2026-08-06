use crate::app::IcoConverterApp;
use crate::theme;

pub(crate) fn modal(app: &mut IcoConverterApp, ctx: &egui::Context) {
    egui::Area::new(egui::Id::new("about_modal"))
        .order(egui::Order::Foreground)
        .show(ctx, |ui| {
            let screen = ctx.screen_rect();
            let painter = ui.painter();
            painter.rect_filled(screen, 0.0, egui::Color32::from_black_alpha(150));

            let overlay = ui.interact(
                screen,
                egui::Id::new("about_modal_block"),
                egui::Sense::click(),
            );

            let panel_size = egui::vec2(340.0, 230.0);
            let panel_rect = egui::Rect::from_center_size(screen.center(), panel_size);
            painter.rect_filled(panel_rect, egui::Rounding::same(14.0), theme::CARD);
            painter.rect_stroke(
                panel_rect,
                egui::Rounding::same(14.0),
                egui::Stroke::new(1.0_f32, theme::BORDER),
            );

            if overlay.clicked() {
                let pos = overlay.interact_pointer_pos();
                if pos.map_or(true, |p| !panel_rect.contains(p)) {
                    app.show_about = false;
                }
            }
            if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
                app.show_about = false;
            }

            ui.allocate_ui_at_rect(panel_rect, |ui| {
                let x = egui::Button::new(egui::RichText::new("✕").size(15.0).color(theme::MUTED))
                    .frame(false);
                let x_rect = egui::Rect::from_min_size(
                    panel_rect.right_top() + egui::vec2(-30.0, 10.0),
                    egui::vec2(20.0, 20.0),
                );
                if ui.put(x_rect, x).on_hover_text("Close").clicked() {
                    app.show_about = false;
                }

                ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                    ui.add_space(30.0);
                    ui.label(
                        egui::RichText::new("ICO Converter")
                            .size(18.0)
                            .strong()
                            .color(theme::TEXT),
                    );
                    ui.add_space(8.0);
                    ui.label(
                        egui::RichText::new("Developed: Sajedur Rahman Roni")
                            .size(13.0)
                            .color(theme::TEXT),
                    );
                    ui.add_space(4.0);
                    ui.label(
                        egui::RichText::new(format!("Version {}", env!("CARGO_PKG_VERSION")))
                            .size(12.0)
                            .color(theme::MUTED),
                    );
                    ui.add_space(10.0);
                    ui.hyperlink_to(
                        egui::RichText::new("GitHub: github.com/sajedur0")
                            .size(13.0)
                            .color(theme::ACCENT),
                        "https://github.com/sajedur0",
                    );
                });

                ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
                    ui.add_space(16.0);
                    let close = egui::Button::new(
                        egui::RichText::new("Close")
                            .size(14.0)
                            .strong()
                            .color(egui::Color32::WHITE),
                    )
                    .fill(theme::ACCENT)
                    .stroke(egui::Stroke::NONE)
                    .min_size(egui::vec2(120.0, 36.0));
                    if ui.add(close).clicked() {
                        app.show_about = false;
                    }
                });
            });
        });
}
