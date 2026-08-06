use crate::app::IcoConverterApp;
use crate::theme;

pub(crate) fn show(
    app: &mut IcoConverterApp,
    ui: &mut egui::Ui,
    ctx: &egui::Context,
    is_dragging: bool,
) {
    let full = ui.available_width();
    let (rect, resp) = ui.allocate_exact_size(egui::vec2(full, 190.0), egui::Sense::click());

    let hovered = resp.hovered() || is_dragging;
    let fill = if hovered { theme::CARD_HOVER } else { theme::CARD };
    let stroke = egui::Stroke::new(
        if hovered { 2.0_f32 } else { 1.0_f32 },
        if hovered { theme::ACCENT } else { theme::BORDER },
    );

    let painter = ui.painter();
    painter.rect_filled(rect, egui::Rounding::same(14.0), fill);
    painter.rect_stroke(rect, egui::Rounding::same(14.0), stroke);

    if resp.clicked() {
        app.handle_file_pick(ctx);
        return;
    }

    match &app.preview {
        Some(tex) => {
            let img = tex.size_vec2();
            let inner = rect.shrink2(egui::vec2(16.0, 30.0));
            let scale = (inner.width() / img.x).min(inner.height() / img.y).min(3.0);
            let draw = img * scale;
            let img_rect = egui::Rect::from_center_size(inner.center(), draw);
            let uv = egui::Rect::from_min_max(
                egui::Pos2::ZERO,
                egui::pos2(img_rect.width() / 16.0, img_rect.height() / 16.0),
            );
            painter.image(app.checker.id(), img_rect, uv, egui::Color32::WHITE);
            painter.image(
                tex.id(),
                img_rect,
                egui::Rect::from_min_max(egui::Pos2::ZERO, egui::pos2(1.0, 1.0)),
                egui::Color32::WHITE,
            );

            let name = app
                .source_path
                .as_ref()
                .and_then(|p| p.file_name().map(|s| s.to_string_lossy().into_owned()))
                .unwrap_or_default();
            let dims = tex.size_vec2();
            let info = format!("{name}   ·   {} × {}", dims.x as u32, dims.y as u32);
            let info_pos = egui::pos2(rect.center().x, rect.bottom() - 14.0);
            painter.text(
                info_pos,
                egui::Align2::CENTER_CENTER,
                info,
                egui::FontId::proportional(12.0),
                theme::MUTED,
            );
        }
        None => {
            let center = rect.center();
            painter.text(
                egui::pos2(center.x, center.y - 10.0),
                egui::Align2::CENTER_CENTER,
                "Drop your image here",
                egui::FontId::proportional(16.0),
                if is_dragging { theme::ACCENT_HOVER } else { theme::TEXT },
            );
            painter.text(
                egui::pos2(center.x, center.y + 14.0),
                egui::Align2::CENTER_CENTER,
                "or click to browse",
                egui::FontId::proportional(13.0),
                theme::MUTED,
            );
        }
    }
}
