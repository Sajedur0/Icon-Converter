//! Minimal source-image card: clean drop zone when empty, compact side-by-side
//! preview + metadata + pixel-exact mini icons when loaded.

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
    let width = ui.available_width();
    let (rect, _) = ui.allocate_exact_size(egui::vec2(width, 138.0), egui::Sense::hover());
    let painter = ui.painter();

    painter.rect_filled(rect, 10.0, p.surface_alt);
    painter.rect_stroke(rect, 10.0, egui::Stroke::new(1.0_f32, p.border));

    painter.text(
        egui::pos2(rect.center().x, rect.center().y - 10.0),
        egui::Align2::CENTER_CENTER,
        format!("Preparing {}…", utils::truncate_middle(name, 34)),
        theme::font(13.5, Weight::SemiBold),
        p.text,
    );
    painter.text(
        egui::pos2(rect.center().x, rect.center().y + 12.0),
        egui::Align2::CENTER_CENTER,
        "Decoding & rendering icon frames",
        theme::font(11.0, Weight::Regular),
        p.muted,
    );
}

fn empty(app: &mut IcoConverterApp, ui: &mut egui::Ui, ctx: &egui::Context, is_dragging: bool) {
    let p = PALETTE;
    let width = ui.available_width();
    let (rect, resp) = ui.allocate_exact_size(egui::vec2(width, 144.0), egui::Sense::click());
    let painter = ui.painter();

    let fill = if is_dragging {
        p.accent_soft
    } else {
        p.surface_alt
    };
    painter.rect_filled(rect, 10.0, fill);

    let border = egui::Stroke::new(
        if is_dragging { 1.8_f32 } else { 1.2_f32 },
        if is_dragging {
            p.accent
        } else {
            p.border_strong
        },
    );
    icons::dashed_round_rect(painter, rect.shrink(1.0), 10.0, border, 7.0, 5.0);

    let badge = egui::Rect::from_center_size(
        egui::pos2(rect.center().x, rect.top() + 34.0),
        egui::vec2(38.0, 38.0),
    );
    painter.rect_filled(badge, 10.0, p.accent_soft);
    icons::upload(painter, badge.shrink(10.0), p.accent);

    let headline = if is_dragging {
        "Release to load image"
    } else {
        "Drag & drop an image here"
    };
    painter.text(
        egui::pos2(rect.center().x, rect.top() + 69.0),
        egui::Align2::CENTER_CENTER,
        headline,
        theme::font(13.5, Weight::SemiBold),
        if is_dragging { p.accent } else { p.text },
    );

    let button = egui::Rect::from_center_size(
        egui::pos2(rect.center().x, rect.top() + 100.0),
        egui::vec2(146.0, 28.0),
    );
    painter.rect_filled(button, 7.0, p.surface);
    painter.rect_stroke(button, 7.0, egui::Stroke::new(1.0_f32, p.border_strong));
    icons::folder(
        painter,
        egui::Rect::from_center_size(
            egui::pos2(button.left() + 20.0, button.center().y),
            egui::vec2(14.0, 14.0),
        ),
        p.text,
    );
    painter.text(
        egui::pos2(button.left() + 33.0, button.center().y),
        egui::Align2::LEFT_CENTER,
        "Browse  ·  Ctrl+O",
        theme::font(11.5, Weight::SemiBold),
        p.text,
    );

    painter.text(
        egui::pos2(rect.center().x, rect.top() + 129.0),
        egui::Align2::CENTER_CENTER,
        "PNG · JPG · WEBP · BMP · GIF · TIFF · ICO",
        theme::font(10.5, Weight::Regular),
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
                _ => format!("{w} × {h}"),
            };
            let size = s
                .size_bytes
                .map(utils::human_size)
                .unwrap_or_else(|| "Memory".to_owned());
            let prep = utils::format_duration(s.prep_duration);
            (
                utils::truncate_middle(&s.name, 30),
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
        ui.spacing_mut().item_spacing.x = 12.0;

        // Left: crisp 96x96 framed square preview on transparency checkerboard
        let (box_rect, _) = ui.allocate_exact_size(egui::vec2(96.0, 96.0), egui::Sense::hover());
        let painter = ui.painter();
        painter.rect_filled(box_rect, 9.0, p.surface_alt);
        painter.rect_stroke(box_rect, 9.0, egui::Stroke::new(1.0_f32, p.border));

        if let Some(tex_id) = main_tex {
            let img_rect = box_rect.shrink(6.0);
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

        // Right: file metadata, live 16/24/32 px actual-pixel strip and Replace/Remove actions
        ui.vertical(|ui| {
            ui.spacing_mut().item_spacing.y = 6.0;

            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.spacing_mut().item_spacing.y = 1.0;
                    ui.label(theme::txt(name, 13.0, Weight::SemiBold, p.text));
                    ui.label(theme::txt(meta_line, 11.0, Weight::Regular, p.muted));
                });

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                    ui.spacing_mut().item_spacing.x = 6.0;
                    let remove =
                        egui::Button::new(theme::txt("Remove", 11.5, Weight::SemiBold, p.danger))
                            .fill(p.danger_soft)
                            .stroke(egui::Stroke::NONE)
                            .rounding(7.0)
                            .min_size(egui::vec2(0.0, 28.0));
                    remove_clicked = ui.add(remove).clicked();
                    replace_clicked = ui.add(theme::secondary_button("Replace")).clicked();
                });
            });

            // Inline actual-pixel mini icons (16, 24, 32 px)
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 6.0;
                for (idx, &size) in converter::RULER_SIZES.iter().enumerate() {
                    let active = app.is_size_selected(size);
                    let (tile, resp) =
                        ui.allocate_exact_size(egui::vec2(62.0, 42.0), egui::Sense::click());
                    let painter = ui.painter();
                    let border_col = if resp.hovered() {
                        p.accent
                    } else if active {
                        p.border_strong
                    } else {
                        p.border
                    };
                    painter.rect_filled(tile, 7.0, p.surface_alt);
                    painter.rect_stroke(tile, 7.0, egui::Stroke::new(1.0_f32, border_col));

                    let side = size as f32;
                    let icon_rect = egui::Rect::from_center_size(
                        egui::pos2(tile.left() + 20.0, tile.center().y),
                        egui::vec2(side.min(28.0), side.min(28.0)),
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
                        egui::pos2(tile.right() - 8.0, tile.center().y),
                        egui::Align2::RIGHT_CENTER,
                        format!("{size}"),
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
                        .on_hover_text(format!("Toggle {size} × {size}"))
                        .clicked()
                    {
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
