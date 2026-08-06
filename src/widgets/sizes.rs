use crate::app::IcoConverterApp;
use crate::converter;
use crate::theme;

pub(crate) fn show(app: &mut IcoConverterApp, ui: &mut egui::Ui) {
    ui.add_space(6.0);
    let count = app.selected_sizes.len();
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new("Sizes").size(14.0).strong().color(theme::TEXT));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(
                egui::RichText::new(format!("{count} selected"))
                    .size(12.0)
                    .color(theme::MUTED),
            );
            let reset = egui::Button::new(
                egui::RichText::new("Reset").size(12.0).color(theme::ACCENT_HOVER),
            )
            .frame(false);
            if ui.add(reset).clicked() {
                app.selected_sizes = converter::AVAILABLE_SIZES.iter().copied().collect();
            }
        });
    });
    ui.add_space(6.0);

    let row_w = ui.available_width();
    let gap = 8.0;
    let chip_w =
        (row_w - gap * (converter::AVAILABLE_SIZES.len() as f32 - 1.0)) / converter::AVAILABLE_SIZES.len() as f32;
    ui.horizontal(|ui| {
        for &size in converter::AVAILABLE_SIZES.iter() {
            let selected = app.selected_sizes.contains(&size);
            let label = egui::SelectableLabel::new(
                selected,
                egui::RichText::new(format!("{size}"))
                    .size(13.0)
                    .strong()
                    .color(if selected { egui::Color32::WHITE } else { theme::TEXT }),
            );
            let resp = ui.add_sized([chip_w, 36.0], label);
            if resp.clicked() {
                if selected {
                    app.selected_sizes.remove(&size);
                } else {
                    app.selected_sizes.insert(size);
                }
            }
        }
    });
}
