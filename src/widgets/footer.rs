//! Bottom action bar: live export spec and primary CTA.

use crate::app::IcoConverterApp;
use crate::icons;
use crate::theme::{self, Weight, PALETTE};
use crate::utils;

pub fn show(app: &mut IcoConverterApp, ui: &mut egui::Ui) {
    let p = PALETTE;
    let count = app.selected_count();
    let is_loading = app.loading_label.is_some();
    let has_source = app.source.is_some();
    let can_convert = has_source && !is_loading && count > 0;
    let noun = if count == 1 { "size" } else { "sizes" };

    let summary = match app.estimated_ico_bytes() {
        Some(bytes) if bytes > 0 => format!(
            "{count} {noun} · {} · {}",
            app.fit_mode.label(),
            utils::human_size(bytes),
        ),
        _ => format!("{count} {noun} · {}", app.fit_mode.label()),
    };

    ui.horizontal(|ui| {
        ui.label(theme::txt(summary, 11.0, Weight::Regular, p.muted));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(theme::txt("Ctrl+Enter", 11.0, Weight::Regular, p.subtle));
        });
    });

    ui.add_space(6.0);

    let width = ui.available_width().floor();
    let (rect, resp) = ui.allocate_exact_size(egui::vec2(width, 38.0), egui::Sense::click());
    let hovered = resp.hovered() && can_convert;
    let fill = if !can_convert {
        p.border_strong
    } else if hovered {
        p.accent_hover
    } else {
        p.accent
    };
    let foreground = if can_convert { p.on_accent } else { p.text };

    let label = if is_loading {
        "Processing…"
    } else if !has_source {
        "Select Source Image"
    } else if count == 0 {
        "Choose Output Size"
    } else {
        "Export .ICO"
    };

    let painter = ui.painter();
    theme::paint_box(painter, rect, 8, fill, None);

    let galley = painter.layout_no_wrap(
        label.to_owned(),
        theme::font(12.5, Weight::SemiBold),
        foreground,
    );
    let icon_side = 15.0;
    let gap = 7.0;
    let group_width = icon_side + gap + galley.size().x;
    let start_x = (rect.center().x - group_width * 0.5).round();

    icons::download(
        painter,
        egui::Rect::from_center_size(
            egui::pos2(start_x + icon_side * 0.5, rect.center().y),
            egui::vec2(icon_side, icon_side),
        ),
        foreground,
    );
    let text_pos = theme::snap_pos(
        painter,
        egui::pos2(
            start_x + icon_side + gap,
            rect.center().y - galley.size().y * 0.5,
        ),
    );
    painter.galley(text_pos, galley, foreground);

    if resp.clicked() && can_convert {
        app.convert_and_save();
    }
}
