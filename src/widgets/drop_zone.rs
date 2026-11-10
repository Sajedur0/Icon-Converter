//! Source card: compact drop zone, live square preview, metadata and pixel-exact mini tiles.

use crate::app::IcoConverterApp;
use crate::converter;
use crate::icons;
use crate::theme::{self, Weight, PALETTE};
use crate::utils;

pub fn show(app: &mut IcoConverterApp, ui: &mut egui::Ui, ctx: &egui::Context, is_dragging: bool) {
    let has_source = app.source.is_some();
    let loading = app.loading_label.clone();

    theme::card().show(ui, |ui| {
        ui.spacing_mut().item_spacing = egui::vec2(7.0, 7.0);
        if has_source {
            loaded(app, ui, ctx);
        } else if let Some(name) = loading {
            preparing(ui, &name);
        } else {
            empty(app, ui, ctx, is_dragging);
        }
    });
}

fn preparing(ui: &mut egui::Ui, name: &str) {
    let p = PALETTE;
    let width = ui.available_width();
    let (rect, _) = ui.allocate_exact_size(egui::vec2(width, 118.0), egui::Sense::hover());
    let painter = ui.painter();

    painter.rect_filled(rect, 8.0, p.surface_alt);
    painter.rect_stroke(rect, 8.0, egui::Stroke::new(1.0_f32, p.border));

    painter.text(
        egui::pos2(rect.center().x, rect.center().y - 8.0),
        egui::Align2::CENTER_CENTER,
        format!("Processing {}…", utils::truncate_middle(name, 34)),
        theme::font(11.5, Weight::SemiBold),
        p.text,
    );
    painter.text(
        egui::pos2(rect.center().x, rect.center().y + 10.0),
        egui::Align2::CENTER_CENTER,
        "Rendering linear-light frames",
        theme::font(9.5, Weight::Regular),
        p.muted,
    );
}

fn empty(app: &mut IcoConverterApp, ui: &mut egui::Ui, ctx: &egui::Context, is_dragging: bool) {
    let p = PALETTE;
    let width = ui.available_width();
    let (rect, resp) = ui.allocate_exact_size(egui::vec2(width, 122.0), egui::Sense::click());
    let painter = ui.painter();

    let fill = if is_dragging {
        p.accent_soft
    } else {
        p.surface_alt
    };
    painter.rect_filled(rect, 8.0, fill);

    let border = egui::Stroke::new(
        if is_dragging { 1.6_f32 } else { 1.1_f32 },
        if is_dragging {
            p.accent
        } else {
            p.border_strong
        },
    );
    icons::dashed_round_rect(painter, rect.shrink(1.0), 8.0, border, 6.0, 4.5);

    let badge = egui::Rect::from_center_size(
        egui::pos2(rect.center().x, rect.top() + 28.0),
        egui::vec2(32.0, 32.0),
    );
    painter.rect_filled(badge, 8.0, p.accent_soft);
    icons::upload(painter, badge.shrink(8.5), p.accent);

    let headline = if is_dragging {
        "Release to import"
    } else {
        "Drop source image here"
    };
    painter.text(
        egui::pos2(rect.center().x, rect.top() + 57.0),
        egui::Align2::CENTER_CENTER,
        headline,
        theme::font(11.5, Weight::SemiBold),
        if is_dragging { p.accent } else { p.text },
    );

    let button = egui::Rect::from_center_size(
        egui::pos2(rect.center().x, rect.top() + 84.0),
        egui::vec2(126.0, 24.0),
    );
    painter.rect_filled(button, 6.0, p.surface);
    painter.rect_stroke(button, 6.0, egui::Stroke::new(1.0_f32, p.border_strong));
    icons::folder(
        painter,
        egui::Rect::from_center_size(
            egui::pos2(button.left() + 17.0, button.center().y),
            egui::vec2(12.5, 12.5),
        ),
        p.text,
    );
    painter.text(
        egui::pos2(button.left() + 28.0, button.center().y),
        egui::Align2::LEFT_CENTER,
        "Browse · Ctrl+O",
        theme::font(10.0, Weight::SemiBold),
        p.text,
    );

    painter.text(
        egui::pos2(rect.center().x, rect.top() + 109.0),
        egui::Align2::CENTER_CENTER,
        "PNG · JPG · WEBP · BMP · GIF · TIFF · ICO",
        theme::font(9.5, Weight::Regular),
        p.subtle,
    );

    if resp.clicked() {
        app.handle_file_pick(ctx);
    }
}

fn loaded(app: &mut IcoConverterApp, ui: &mut egui::Ui, ctx: &egui::Context) {
    let p = PALETTE;
    let (name, meta_line) = match app.source.as_ref() {
        Some(s) => {
            let w = s.prepared.width();
            let h = s.prepared.height();
            let dims = match (app.fit_mode, s.prepared.alpha_bbox()) {
                (converter::FitMode::Trim, Some((_, _, bw, bh))) if bw != w || bh != h => {
                    format!("{w}×{h} → {bw}×{bh}")
                }
                _ => format!("{w}×{h}"),
            };
            let size = s
                .size_bytes
                .map(utils::human_size)
                .unwrap_or_else(|| "Buffer".to_owned());
            let prep = utils::format_duration(s.prep_duration);
            (
                utils::truncate_middle(&s.name, 28),
                format!("{} · {dims} · {size} · {prep}", s.format),
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

    let mut remove_clicked = false;
    let mut replace_clicked = false;
    let mut toggled_size: Option<u32> = None;

    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 10.0;

        let (box_rect, _) = ui.allocate_exact_size(egui::vec2(86.0, 86.0), egui::Sense::hover());
        let painter = ui.painter();
        painter.rect_filled(box_rect, 8.0, p.surface_alt);
        painter.rect_stroke(box_rect, 8.0, egui::Stroke::new(1.0_f32, p.border));

        if let Some(tex_id) = main_tex {
            let img_rect = box_rect.shrink(5.0);
            let side = img_rect.width();
            let tiled_uv =
                egui::Rect::from_min_max(egui::Pos2::ZERO, egui::pos2(side / 12.0, side / 12.0));
            painter.image(checker, img_rect, tiled_uv, egui::Color32::WHITE);
            painter.image(
                tex_id,
                img_rect,
                egui::Rect::from_min_max(egui::Pos2::ZERO, egui::pos2(1.0, 1.0)),
                egui::Color32::WHITE,
            );
            painter.rect_stroke(img_rect, 2.0, egui::Stroke::new(1.0_f32, p.border_strong));
        }

        ui.vertical(|ui| {
            ui.spacing_mut().item_spacing.y = 6.0;

            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.spacing_mut().item_spacing.y = 1.0;
                    ui.label(theme::txt(name, 11.5, Weight::SemiBold, p.text));
                    ui.label(theme::txt(meta_line, 9.5, Weight::Regular, p.muted));
                });

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                    ui.spacing_mut().item_spacing.x = 5.0;
                    let clear =
                        egui::Button::new(theme::txt("Clear", 10.0, Weight::SemiBold, p.danger))
                            .fill(p.danger_soft)
                            .stroke(egui::Stroke::NONE)
                            .rounding(6.0)
                            .min_size(egui::vec2(0.0, 24.0));
                    remove_clicked = ui.add(clear).clicked();
                    replace_clicked = ui.add(theme::secondary_button("Change")).clicked();
                });
            });

            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 5.0;
                for (idx, &size) in converter::RULER_SIZES.iter().enumerate() {
                    let active = app.is_size_selected(size);
                    let (tile, resp) =
                        ui.allocate_exact_size(egui::vec2(60.0, 36.0), egui::Sense::click());
                    let painter = ui.painter();
                    let border_col = if resp.hovered() {
                        p.accent
                    } else if active {
                        p.border_strong
                    } else {
                        p.border
                    };
                    painter.rect_filled(tile, 6.0, p.surface_alt);
                    painter.rect_stroke(tile, 6.0, egui::Stroke::new(1.0_f32, border_col));

                    let side = (size as f32).min(24.0);
                    let icon_rect = egui::Rect::from_center_size(
                        egui::pos2(tile.left() + 17.0, tile.center().y),
                        egui::vec2(side, side),
                    );
                    if let Some(ruler_ids) = ruler_tex {
                        let tint = if active {
                            egui::Color32::WHITE
                        } else {
                            egui::Color32::from_white_alpha(135)
                        };
                        let uv = egui::Rect::from_min_max(
                            egui::Pos2::ZERO,
                            egui::pos2(icon_rect.width() / 10.0, icon_rect.height() / 10.0),
                        );
                        painter.image(checker, icon_rect, uv, egui::Color32::WHITE);
                        painter.image(
                            ruler_ids[idx],
                            icon_rect,
                            egui::Rect::from_min_max(egui::Pos2::ZERO, egui::pos2(1.0, 1.0)),
                            tint,
                        );
                    }
                    painter.text(
                        egui::pos2(tile.right() - 7.0, tile.center().y),
                        egui::Align2::RIGHT_CENTER,
                        format!("{size}px"),
                        theme::font(
                            9.5,
                            if active {
                                Weight::SemiBold
                            } else {
                                Weight::Regular
                            },
                        ),
                        if active { p.text } else { p.subtle },
                    );

                    if resp.on_hover_text(format!("{size}×{size} px")).clicked() {
                        toggled_size = Some(size);
                    }
                }
            });
        });
    });

    if remove_clicked {
        app.clear_source();
    } else if replace_clicked {
        app.handle_file_pick(ctx);
    } else if let Some(size) = toggled_size {
        if let Some(pos) = converter::AVAILABLE_SIZES.iter().position(|&s| s == size) {
            app.selected_sizes[pos] = !app.selected_sizes[pos];
        }
    }
}
