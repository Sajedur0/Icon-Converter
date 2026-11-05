//! Numbered section heading used to structure the workflow.

use crate::theme::{self, Palette, Weight};

pub fn show(ui: &mut egui::Ui, p: &Palette, step: &str, title: &str, trailing: Option<String>) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 8.0;

        let (badge, _) = ui.allocate_exact_size(egui::vec2(26.0, 22.0), egui::Sense::hover());
        let painter = ui.painter();
        painter.rect_filled(badge, 6.0, p.accent_soft);
        painter.text(
            badge.center(),
            egui::Align2::CENTER_CENTER,
            step,
            theme::font(10.0, Weight::Bold),
            p.accent,
        );

        ui.label(theme::txt(title, 13.5, Weight::SemiBold, p.text));

        if let Some(text) = trailing {
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(theme::txt(text, 11.5, Weight::Regular, p.muted));
            });
        }
    });
    ui.add_space(1.0);
}
