//! Compact top bar: brand mark, concise identity and About trigger.

use crate::app::IcoConverterApp;
use crate::icons;
use crate::theme::{self, Weight, PALETTE};

pub fn show(app: &mut IcoConverterApp, ui: &mut egui::Ui) {
    let p = PALETTE;

    let info_clicked = ui
        .horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 10.0;

            let (mark, _) = ui.allocate_exact_size(egui::vec2(32.0, 32.0), egui::Sense::hover());
            let painter = ui.painter();
            theme::paint_box(painter, mark, 8, p.accent, None);
            icons::image(painter, mark.shrink(7.0), p.on_accent);

            ui.vertical(|ui| {
                ui.spacing_mut().item_spacing.y = 1.0;
                ui.label(theme::txt("ICO Converter", 14.0, Weight::SemiBold, p.text));
                ui.label(theme::txt(
                    "Multi-Resolution Icon Studio · 16–512 px",
                    11.0,
                    Weight::Regular,
                    p.muted,
                ));
            });

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let (rect, resp) =
                    ui.allocate_exact_size(egui::vec2(28.0, 28.0), egui::Sense::click());
                let hovered = resp.hovered();
                if hovered {
                    theme::paint_box(ui.painter(), rect, 6, p.surface_hover, None);
                }
                let color = if hovered { p.accent } else { p.muted };
                icons::info(ui.painter(), rect.shrink(6.0), color);
                resp.on_hover_text("About").clicked()
            })
            .inner
        })
        .inner;

    if info_clicked {
        app.show_about = !app.show_about;
    }
}
