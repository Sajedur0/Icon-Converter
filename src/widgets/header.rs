//! Compact top bar: brand mark, concise identity and About trigger.

use crate::app::IcoConverterApp;
use crate::icons;
use crate::theme::{self, Weight, PALETTE};

pub fn show(app: &mut IcoConverterApp, ui: &mut egui::Ui) {
    let p = PALETTE;

    let info_clicked = ui
        .horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 10.0;

            let (mark, _) = ui.allocate_exact_size(egui::vec2(38.0, 38.0), egui::Sense::hover());
            theme::paint_image_contained(ui.painter(), mark, &app.app_icon);

            ui.label(theme::txt("ICO Converter", 15.0, Weight::SemiBold, p.text));

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
