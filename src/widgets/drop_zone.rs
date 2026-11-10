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
        ui.spacing_mut().item_spacing = egui::vec2(8.0, 8.0);
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
    let width = ui.available_width().floor();
    let (rect, _) = ui.allocate_exact_size(egui::vec2(width, 124.0), egui::Sense::hover());
    let painter = ui.painter();

    theme::paint_box(
        painter,
        rect,
        8,
        p.surface_alt,
        Some(egui::Stroke::new(1.0_f32, p.border)),
    );

    theme::paint_text(
        painter,
        egui::pos2(rect.center().x, rect.center().y - 10.0),
        egui::Align2::CENTER_CENTER,
        format!("Processing {}…", utils::truncate_middle(name, 32)),
        theme::font(12.5, Weight::SemiBold),
        p.text,
    );
    theme::paint_text(
        painter,
        egui::pos2(rect.center().x, rect.center().y + 12.0),
        egui::Align2::CENTER_CENTER,
        "Rendering linear-light frames",
        theme::font(11.0, Weight::Regular),
        p.muted,
    );
}

fn empty(app: &mut IcoConverterApp, ui: &mut egui::Ui, ctx: &egui::Context, is_dragging: bool) {
    let p = PALETTE;
    let width = ui.available_width().floor();
    let (rect, resp) = ui.allocate_exact_size(egui::vec2(width, 124.0), egui::Sense::click());
    let painter = ui.painter();

    let fill = if is_dragging {
        p.accent_soft
    } else {
        p.surface_alt
    };
    theme::paint_box(painter, rect, 8, fill, None);

    let border = egui::Stroke::new(
        if is_dragging { 1.5_f32 } else { 1.0_f32 },
        if is_dragging {
            p.accent
        } else {
            p.border_strong
        },
    );
    icons::dashed_round_rect(painter, rect.shrink(1.0), 8.0, border, 6.0, 4.0);

    let badge = egui::Rect::from_center_size(
        egui::pos2(rect.center().x, rect.top() + 26.0),
        egui::vec2(32.0, 32.0),
    );
    theme::paint_box(painter, badge, 8, p.accent_soft, None);
    icons::upload(painter, badge.shrink(8.0), p.accent);

    let headline = if is_dragging {
        "Release to import"
    } else {
        "Drop source image here"
    };
    theme::paint_text(
        painter,
        egui::pos2(rect.center().x, rect.top() + 56.0),
        egui::Align2::CENTER_CENTER,
        headline,
        theme::font(12.5, Weight::SemiBold),
        if is_dragging { p.accent } else { p.text },
    );

    let button = egui::Rect::from_center_size(
        egui::pos2(rect.center().x, rect.top() + 84.0),
        egui::vec2(132.0, 26.0),
    );
    theme::paint_box(
        painter,
        button,
        6,
        p.surface,
        Some(egui::Stroke::new(1.0_f32, p.border_strong)),
    );
    let btn_galley = painter.layout_no_wrap(
        "Browse · Ctrl+O".to_owned(),
        theme::font(11.0, Weight::SemiBold),
        p.text,
    );
    let icon_side = 13.0;
    let icon_gap = 6.0;
    let total_w = icon_side + icon_gap + btn_galley.size().x;
    let start_x = (button.center().x - total_w * 0.5).round();
    icons::folder(
        painter,
        egui::Rect::from_center_size(
            egui::pos2(start_x + icon_side * 0.5, button.center().y),
            egui::vec2(icon_side, icon_side),
        ),
        p.text,
    );
    let btn_text_pos = theme::snap_pos(
        painter,
        egui::pos2(
            start_x + icon_side + icon_gap,
            button.center().y - btn_galley.size().y * 0.5,
        ),
    );
    painter.galley(btn_text_pos, btn_galley, p.text);

    theme::paint_text(
        painter,
        egui::pos2(rect.center().x, rect.top() + 110.0),
        egui::Align2::CENTER_CENTER,
        "PNG · JPG · WEBP · BMP · GIF · TIFF · ICO",
        theme::font(11.0, Weight::Regular),
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
                utils::truncate_middle(&s.name, 26),
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

        let (box_rect, _) = ui.allocate_exact_size(egui::vec2(88.0, 88.0), egui::Sense::hover());
        let painter = ui.painter();
        theme::paint_box(
            painter,
            box_rect,
            8,
            p.surface_alt,
            Some(egui::Stroke::new(1.0_f32, p.border)),
        );

        if let Some(tex_id) = main_tex {
            let img_rect = theme::snap_rect(painter, box_rect.shrink(6.0));
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
            painter.rect_stroke(
                img_rect,
                egui::CornerRadius::same(2),
                egui::Stroke::new(1.0_f32, p.border_strong),
                egui::StrokeKind::Inside,
            );
        }

        ui.vertical(|ui| {
            ui.spacing_mut().item_spacing.y = 6.0;

            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.spacing_mut().item_spacing.y = 1.0;
                    ui.label(theme::txt(name, 12.0, Weight::SemiBold, p.text));
                    ui.label(theme::txt(meta_line, 11.0, Weight::Regular, p.muted));
                });

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                    ui.spacing_mut().item_spacing.x = 6.0;
                    let clear =
                        egui::Button::new(theme::txt("Clear", 11.0, Weight::SemiBold, p.danger))
                            .fill(p.danger_soft)
                            .stroke(egui::Stroke::NONE)
                            .corner_radius(egui::CornerRadius::same(6))
                            .min_size(egui::vec2(0.0, 26.0));
                    remove_clicked = ui.add(clear).clicked();
                    replace_clicked = ui.add(theme::secondary_button("Change")).clicked();
                });
            });

            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 6.0;
                for (idx, &size) in converter::RULER_SIZES.iter().enumerate() {
                    let active = app.is_size_selected(size);
                    let (tile, resp) =
                        ui.allocate_exact_size(egui::vec2(72.0, 38.0), egui::Sense::click());
                    let painter = ui.painter();
                    let border_col = if resp.hovered() {
                        p.accent
                    } else if active {
                        p.border_strong
                    } else {
                        p.border
                    };
                    theme::paint_box(
                        painter,
                        tile,
                        6,
                        p.surface_alt,
                        Some(egui::Stroke::new(1.0_f32, border_col)),
                    );

                    let side = (size as f32).min(24.0);
                    let icon_rect = theme::snap_rect(
                        painter,
                        egui::Rect::from_center_size(
                            egui::pos2(tile.left() + 18.0, tile.center().y),
                            egui::vec2(side, side),
                        ),
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
                    theme::paint_text(
                        painter,
                        egui::pos2(tile.right() - 7.0, tile.center().y),
                        egui::Align2::RIGHT_CENTER,
                        format!("{size}px"),
                        theme::font(
                            11.0,
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
