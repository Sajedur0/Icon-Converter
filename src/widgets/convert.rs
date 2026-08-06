use crate::app::IcoConverterApp;
use crate::theme;

pub(crate) fn browse_row(app: &mut IcoConverterApp, ui: &mut egui::Ui, ctx: &egui::Context) {
    let btn = egui::Button::new(
        egui::RichText::new("Browse Image")
            .size(15.0)
            .strong()
            .color(egui::Color32::WHITE),
    )
    .fill(theme::ACCENT)
    .stroke(egui::Stroke::NONE);
    if ui.add_sized([ui.available_width(), 44.0], btn).clicked() {
        app.handle_file_pick(ctx);
    }
}

pub(crate) fn convert_button(app: &mut IcoConverterApp, ui: &mut egui::Ui) {
    ui.add_space(8.0);
    let can_convert = app.source_path.is_some() && !app.selected_sizes.is_empty();
    let btn = egui::Button::new(
        egui::RichText::new("Convert & Save ICO")
            .size(16.0)
            .strong()
            .color(egui::Color32::WHITE),
    )
    .fill(theme::ACCENT)
    .stroke(egui::Stroke::NONE)
    .min_size(egui::vec2(ui.available_width(), 48.0));
    let resp = ui.add_enabled(can_convert, btn);
    if resp.clicked() {
        app.convert_and_save();
    }
    if !can_convert {
        ui.add_space(4.0);
        ui.label(
            egui::RichText::new("Select an image and at least one size to continue.")
                .size(11.5)
                .color(theme::MUTED),
        );
    }
}
