use crate::app::IcoConverterApp;
use crate::theme;

pub(crate) fn show(app: &mut IcoConverterApp, ui: &mut egui::Ui) {
    if app.status_message.is_empty() {
        return;
    }
    ui.add_space(4.0);
    let color = if app.status_is_error { theme::ERROR } else { theme::SUCCESS };
    let bg = egui::Color32::from_rgba_unmultiplied(96, 96, 96, 18);
    egui::Frame::none()
        .fill(bg)
        .stroke(egui::Stroke::new(
            1.0_f32,
            egui::Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), 60),
        ))
        .rounding(egui::Rounding::same(8.0))
        .inner_margin(egui::Margin::same(10.0))
        .show(ui, |ui| {
            ui.label(
                egui::RichText::new(&app.status_message).color(color).size(13.0),
            );
        });
}
