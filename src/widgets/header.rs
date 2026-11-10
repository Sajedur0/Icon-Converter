//! Minimal light-mode top bar: brand mark, title and About action.

use crate::app::IcoConverterApp;
use crate::icons;
use crate::theme::{self, Weight, PALETTE};

pub fn show(app: &mut IcoConverterApp, ui: &mut egui::Ui) {
    let p = PALETTE;

    let info_clicked = ui
        .horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 10.0;

            let (mark, _) = ui.allocate_exact_size(egui::vec2(34.0, 34.0), egui::Sense::hover());
            let painter = ui.painter();
            painter.rect_filled(mark, 9.0, p.accent);
            icons::image(painter, mark.shrink(8.0), p.on_accent);

            ui.vertical(|ui| {
                ui.spacing_mut().item_spacing.y = 0.0;
                ui.label(theme::txt("ICO Converter", 15.5, Weight::SemiBold, p.text));
                ui.label(theme::txt(
                    "Minimal Windows icon builder · 16 – 512 px",
                    11.0,
                    Weight::Regular,
                    p.muted,
                ));
            });

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let (rect, resp) =
                    ui.allocate_exact_size(egui::vec2(30.0, 30.0), egui::Sense::click());
                let hovered = resp.hovered();
                if hovered {
                    ui.painter().rect_filled(rect, 8.0, p.surface_hover);
                }
                let color = if hovered { p.accent } else { p.muted };
                icons::info(ui.painter(), rect.shrink(7.0), color);
                resp.on_hover_text("About").clicked()
            })
            .inner
        })
        .inner;

    if info_clicked {
        app.show_about = !app.show_about;
    }
}
