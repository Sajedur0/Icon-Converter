//! Source-image card: empty drop target, live framed square preview, file details
//! and a pixel-accurate ruler of the rendered icon at its common shell sizes.

use crate::app::IcoConverterApp;
use crate::converter;
use crate::icons;
use crate::theme::{self, Palette, Weight};
use crate::utils;

pub fn show(
    app: &mut IcoConverterApp,
    ui: &mut egui::Ui,
    ctx: &egui::Context,
    is_dragging: bool,
    p: &Palette,
) {
    let has_source = app.source.is_some();
    let loading = app.loading_label.clone();
    theme::card(p).show(ui, |ui| {
        ui.spacing_mut().item_spacing = egui::vec2(8.0, 10.0);
        if has_source {
            loaded(app, ui, ctx, p);
        } else if let Some(name) = loading {
            preparing(ui, &name, p);
        } else {
            empty(app, ui, ctx, is_dragging, p);
        }
    });
}

fn preparing(ui: &mut egui::Ui, name: &str, p: &Palette) {
    let width = ui.available_width();
    let (rect, _) = ui.allocate_exact_size(egui::vec2(width, 196.0), egui::Sense::hover());
    let painter = ui.painter();

    painter.rect_filled(rect, 12.0, p.surface_alt);
    painter.rect_stroke(rect, 12.0, egui::Stroke::new(1.0_f32, p.border));

    let badge = egui::Rect::from_center_size(
        egui::pos2(rect.center().x, rect.top() + 66.0),
        egui::vec2(48.0, 48.0),
    );
    painter.rect_filled(badge, 14.0, p.accent_soft);
    icons::image(painter, badge.shrink(12.0), p.accent);

    painter.text(
        egui::pos2(rect.center().x, rect.top() + 116.0),
        egui::Align2::CENTER_CENTER,
        format!("Preparing {}…", utils::truncate_middle(name, 32)),
        theme::font(14.0, Weight::SemiBold),
        p.text,
    );
    painter.text(
        egui::pos2(rect.center().x, rect.top() + 142.0),
        egui::Align2::CENTER_CENTER,
        "Decoding & rendering linear-light icon frames",
        theme::font(11.5, Weight::Regular),
        p.muted,
    );
}

fn empty(
    app: &mut IcoConverterApp,
    ui: &mut egui::Ui,
    ctx: &egui::Context,
    is_dragging: bool,
    p: &Palette,
) {
    let width = ui.available_width();
    let (rect, resp) = ui.allocate_exact_size(egui::vec2(width, 196.0), egui::Sense::click());
    let painter = ui.painter();

    let fill = if is_dragging {
        p.accent_soft
    } else {
        p.surface_alt
    };
    painter.rect_filled(rect, 12.0, fill);

    let border = egui::Stroke::new(
        if is_dragging { 2.0_f32 } else { 1.4_f32 },
        if is_dragging {
            p.accent
        } else {
            p.border_strong
        },
    );
    icons::dashed_round_rect(painter, rect.shrink(1.0), 12.0, border, 8.0, 6.0);

    let badge = egui::Rect::from_center_size(
        egui::pos2(rect.center().x, rect.top() + 48.0),
        egui::vec2(48.0, 48.0),
    );
    painter.rect_filled(badge, 14.0, p.accent_soft);
    icons::upload(painter, badge.shrink(13.0), p.accent);

    let headline = if is_dragging {
        "Release to add your image"
    } else {
        "Drag & drop an image here"
    };
    painter.text(
        egui::pos2(rect.center().x, rect.top() + 92.0),
        egui::Align2::CENTER_CENTER,
        headline,
        theme::font(14.5, Weight::SemiBold),
        if is_dragging { p.accent } else { p.text },
    );

    let button = egui::Rect::from_center_size(
        egui::pos2(rect.center().x, rect.top() + 130.0),
        egui::vec2(158.0, 34.0),
    );
    painter.rect_filled(button, 8.0, p.surface);
    painter.rect_stroke(button, 8.0, egui::Stroke::new(1.0_f32, p.border_strong));
    icons::folder(
        painter,
        egui::Rect::from_center_size(
            egui::pos2(button.left() + 24.0, button.center().y),
            egui::vec2(16.0, 16.0),
        ),
        p.text,
    );
    painter.text(
        egui::pos2(button.left() + 39.0, button.center().y),
        egui::Align2::LEFT_CENTER,
        "Browse files  ·  Ctrl+O",
        theme::font(12.0, Weight::SemiBold),
        p.text,
    );

    painter.text(
        egui::pos2(rect.center().x, rect.top() + 170.0),
        egui::Align2::CENTER_CENTER,
        "PNG  ·  JPG  ·  WEBP  ·  BMP  ·  GIF  ·  TIFF  ·  ICO",
        theme::font(11.0, Weight::Regular),
        p.subtle,
    );

    if resp.clicked() {
        app.handle_file_pick(ctx);
    }
}

fn loaded(app: &mut IcoConverterApp, ui: &mut egui::Ui, ctx: &egui::Context, p: &Palette) {
    let (name, meta_line) = match app.source.as_ref() {
        Some(s) => {
            let w = s.prepared.width();
            let h = s.prepared.height();
            let dims = match (app.fit_mode, s.prepared.alpha_bbox()) {
                (converter::FitMode::Trim, Some((_, _, bw, bh))) if bw != w || bh != h => {
                    format!("{w} × {h} → {bw} × {bh}")
                }
                _ => format!("{w} × {h}"),
            };
            let size = s
                .size_bytes
                .map(utils::human_size)
                .unwrap_or_else(|| "In memory".to_owned());
            (
                utils::truncate_middle(&s.name, 34),
                format!("{}  ·  {dims}  ·  {size}", s.format),
            )
        }
        None => return,
    };

    let main_tex = app.previews.as_ref().map(|t| t.main.id());
    let ruler_tex: Option<[egui::TextureId; converter::RULER_SIZES.len()]> = app
        .previews
        .as_ref()
        .map(|t| std::array::from_fn(|i| t.ruler[i].id()));
    let checker = app.checker.id();

    // ---- Large WYSIWYG square icon preview -----------------------------
    let width = ui.available_width();
    let (preview_rect, _) = ui.allocate_exact_size(egui::vec2(width, 174.0), egui::Sense::hover());
    let painter = ui.painter();
    painter.rect_filled(preview_rect, 12.0, p.surface_alt);
    painter.rect_stroke(preview_rect, 12.0, egui::Stroke::new(1.0_f32, p.border));

    if let Some(tex_id) = main_tex {
        let inner = preview_rect.shrink(14.0);
        let side = inner.width().min(inner.height());
        let img_rect = egui::Rect::from_center_size(inner.center(), egui::vec2(side, side));
        let tiled_uv =
            egui::Rect::from_min_max(egui::Pos2::ZERO, egui::pos2(side / 16.0, side / 16.0));
        painter.image(checker, img_rect, tiled_uv, egui::Color32::WHITE);
        painter.image(
            tex_id,
            img_rect,
            egui::Rect::from_min_max(egui::Pos2::ZERO, egui::pos2(1.0, 1.0)),
            egui::Color32::WHITE,
        );
        painter.rect_stroke(img_rect, 3.0, egui::Stroke::new(1.0_f32, p.border_strong));
    }

    // ---- File details --------------------------------------------------
    let (remove_clicked, replace_clicked) = ui
        .horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 10.0;
            let (tile, _) = ui.allocate_exact_size(egui::vec2(38.0, 38.0), egui::Sense::hover());
            let painter = ui.painter();
            painter.rect_filled(tile, 9.0, p.accent_soft);
            icons::image(painter, tile.shrink(9.0), p.accent);

            ui.vertical(|ui| {
                ui.spacing_mut().item_spacing.y = 1.0;
                ui.label(theme::txt(name, 13.0, Weight::SemiBold, p.text));
                ui.label(theme::txt(meta_line, 11.5, Weight::Regular, p.muted));
            });

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.spacing_mut().item_spacing.x = 8.0;
                let remove =
                    egui::Button::new(theme::txt("Remove", 11.5, Weight::SemiBold, p.danger))
                        .fill(p.danger_soft)
                        .stroke(egui::Stroke::NONE)
                        .rounding(8.0)
                        .min_size(egui::vec2(0.0, 30.0));
                let remove_clicked = ui.add(remove).clicked();
                let replace_clicked = ui.add(theme::secondary_button("Replace", p)).clicked();
                (remove_clicked, replace_clicked)
            })
            .inner
        })
        .inner;

    if remove_clicked {
        app.clear_source();
    }
    if replace_clicked {
        app.handle_file_pick(ctx);
    }
    if app.source.is_none() {
        return;
    }

    // ---- Pixel-accurate multi-size ruler -------------------------------
    ui.add_space(4.0);
    ui.horizontal(|ui| {
        ui.label(theme::txt("Icon preview", 11.5, Weight::SemiBold, p.muted));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(theme::txt(
                format!("{} · actual resampled pixels", app.fit_mode.label()),
                10.5,
                Weight::Regular,
                p.subtle,
            ));
        });
    });
    ui.add_space(2.0);

    let gap = 8.0;
    let count = converter::RULER_SIZES.len() as f32;
    let tile_width = ((ui.available_width() - gap * (count - 1.0)) / count).max(60.0);

    let mut toggled_size: Option<u32> = None;
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = gap;
        for (idx, &size) in converter::RULER_SIZES.iter().enumerate() {
            let active = app.is_size_selected(size);
            let (tile, resp) =
                ui.allocate_exact_size(egui::vec2(tile_width, 86.0), egui::Sense::click());
            let painter = ui.painter();
            let border_color = if resp.hovered() {
                p.accent
            } else if active {
                p.border_strong
            } else {
                p.border
            };
            painter.rect_filled(tile, 10.0, p.surface_alt);
            painter.rect_stroke(tile, 10.0, egui::Stroke::new(1.0_f32, border_color));

            let side = size as f32;
            let icon_rect = egui::Rect::from_center_size(
                egui::pos2(tile.center().x, tile.top() + 38.0),
                egui::vec2(side, side),
            );
            let tiled_uv =
                egui::Rect::from_min_max(egui::Pos2::ZERO, egui::pos2(side / 16.0, side / 16.0));
            if let Some(ruler_ids) = ruler_tex {
                let tint = if active {
                    egui::Color32::WHITE
                } else {
                    egui::Color32::from_white_alpha(135)
                };
                painter.image(checker, icon_rect, tiled_uv, egui::Color32::WHITE);
                painter.image(
                    ruler_ids[idx],
                    icon_rect,
                    egui::Rect::from_min_max(egui::Pos2::ZERO, egui::pos2(1.0, 1.0)),
                    tint,
                );
            }
            painter.rect_stroke(icon_rect, 2.0, egui::Stroke::new(1.0_f32, p.border_strong));
            painter.text(
                egui::pos2(tile.center().x, tile.bottom() - 11.0),
                egui::Align2::CENTER_CENTER,
                format!("{size} px"),
                theme::font(
                    10.5,
                    if active {
                        Weight::SemiBold
                    } else {
                        Weight::Regular
                    },
                ),
                if active { p.text } else { p.subtle },
            );

            if resp
                .on_hover_text(format!("Click to toggle {size} × {size}"))
                .clicked()
            {
                toggled_size = Some(size);
            }
        }
    });

    if let Some(size) = toggled_size {
        if let Some(pos) = converter::AVAILABLE_SIZES.iter().position(|&s| s == size) {
            app.selected_sizes[pos] = !app.selected_sizes[pos];
        }
    }
}
