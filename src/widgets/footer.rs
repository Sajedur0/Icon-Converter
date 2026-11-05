//! Sticky bottom bar: export summary and the primary action.

use crate::app::IcoConverterApp;
use crate::icons;
use crate::theme::{self, Palette, Weight};

pub fn show(app: &mut IcoConverterApp, ui: &mut egui::Ui, p: &Palette) {
    let count = app.selected_sizes.len();
    let has_source = app.source.is_some();
    let can_convert = has_source && count > 0;

    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 8.0;
        let (icon_rect, _) = ui.allocate_exact_size(egui::vec2(16.0, 16.0), egui::Sense::hover());
        icons::layers(ui.painter(), icon_rect, p.muted);
        ui.label(theme::txt(
            format!(
                "{count} resolutions  ·  {}  ·  Lanczos3",
                app.fit_mode.label()
            ),
            11.5,
            Weight::Regular,
            p.muted,
        ));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(theme::txt("Ctrl + Enter", 10.5, Weight::Regular, p.subtle));
        });
    });

    ui.add_space(8.0);

    let (rect, resp) =
        ui.allocate_exact_size(egui::vec2(ui.available_width(), 46.0), egui::Sense::click());
    let hovered = resp.hovered() && can_convert;
    let fill = if !can_convert {
        p.border_strong
    } else if hovered {
        p.accent_hover
    } else {
        p.accent
    };
    let foreground = if can_convert { p.on_accent } else { p.text };

    let painter = ui.painter();
    painter.rect_filled(rect, 10.0, fill);
    icons::download(
        painter,
        egui::Rect::from_center_size(
            egui::pos2(rect.center().x - 84.0, rect.center().y),
            egui::vec2(18.0, 18.0),
        ),
        foreground,
    );

    let label = if !has_source {
        "Add an image to continue"
    } else if count == 0 {
        "Select at least one size"
    } else {
        "Convert & Save ICO"
    };
    painter.text(
        egui::pos2(rect.center().x - 66.0, rect.center().y),
        egui::Align2::LEFT_CENTER,
        label,
        theme::font(14.0, Weight::SemiBold),
        foreground,
    );

    if resp.clicked() && can_convert {
        app.convert_and_save();
    }
}
