use crate::app::IcoConverterApp;
use crate::theme;

pub(crate) fn show(app: &mut IcoConverterApp, ui: &mut egui::Ui) {
    ui.add_space(2.0);
    let about_rect = egui::Rect::from_min_size(
        egui::pos2(ui.max_rect().right() - 38.0, ui.max_rect().top() + 4.0),
        egui::vec2(32.0, 28.0),
    );
    ui.allocate_ui_at_rect(about_rect, |ui| {
        let btn = egui::Button::new(egui::RichText::new("ℹ").size(20.0).color(theme::ACCENT_HOVER))
            .frame(false);
        if ui.add(btn).on_hover_text("About").clicked() {
            app.show_about = !app.show_about;
        }
    });
    ui.vertical_centered(|ui| {
        ui.label(egui::RichText::new("ICO Converter").size(30.0).strong().color(theme::TEXT));
        ui.add_space(2.0);
        ui.label(
            egui::RichText::new("PNG · JPG · WEBP  →  .ICO")
                .size(13.0)
                .color(theme::MUTED),
        );
    });
    ui.add_space(8.0);
}
