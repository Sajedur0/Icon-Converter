//! Output size grid (`16×16` through `512×512`) and canvas fit selector.

use crate::app::IcoConverterApp;
use crate::converter::{self, FitMode};
use crate::icons;
use crate::theme::{self, Weight, PALETTE};

const COLUMNS: usize = 4;

pub fn show(app: &mut IcoConverterApp, ui: &mut egui::Ui, ctx: &egui::Context) {
    let p = PALETTE;
    let mut selected = app.selected_sizes;
    let current_mode = app.fit_mode;
    let mut next_mode = None;

    theme::card().show(ui, |ui| {
        ui.spacing_mut().item_spacing = egui::vec2(6.0, 6.0);

        let selected_count = selected.iter().filter(|&&on| on).count();
        let (none_clicked, all_clicked) = ui
            .horizontal(|ui| {
                ui.label(theme::txt("Output Sizes", 12.0, Weight::SemiBold, p.text));
                ui.label(theme::txt(
                    format!("{selected_count}/{}", converter::AVAILABLE_SIZES.len()),
                    11.0,
                    Weight::Regular,
                    p.muted,
                ));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.spacing_mut().item_spacing.x = 8.0;
                    let none = ui.add(theme::text_button("None")).clicked();
                    let all = ui.add(theme::text_button("All")).clicked();
                    (none, all)
                })
                .inner
            })
            .inner;

        if none_clicked {
            selected = [false; converter::AVAILABLE_SIZES.len()];
        }
        if all_clicked {
            selected = [true; converter::AVAILABLE_SIZES.len()];
        }

        let gap = 6.0;
        let width = ui.available_width().floor();
        let chip_width = ((width - gap * (COLUMNS - 1) as f32) / COLUMNS as f32).floor();

        for (row_idx, row) in converter::AVAILABLE_SIZES.chunks(COLUMNS).enumerate() {
            let base_idx = row_idx * COLUMNS;
            let toggled = ui
                .horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = gap;
                    let mut clicked_indices: Vec<usize> = Vec::new();
                    for (col_idx, &size) in row.iter().enumerate() {
                        let slot_idx = base_idx + col_idx;
                        let w = if col_idx + 1 == COLUMNS {
                            (width - (chip_width + gap) * (COLUMNS - 1) as f32).max(chip_width)
                        } else {
                            chip_width
                        };
                        let (rect, resp) =
                            ui.allocate_exact_size(egui::vec2(w, 32.0), egui::Sense::click());
                        let is_selected = selected[slot_idx];
                        let hovered = resp.hovered();
                        let painter = ui.painter();

                        let fill = if is_selected {
                            p.accent_soft
                        } else if hovered {
                            p.surface_hover
                        } else {
                            p.surface_alt
                        };
                        let border = if is_selected {
                            p.accent
                        } else if hovered {
                            p.border_strong
                        } else {
                            p.border
                        };
                        if is_selected {
                            theme::paint_shadow(painter, rect, 8, theme::CONTROL_SHADOW);
                        }
                        theme::paint_box(
                            painter,
                            rect,
                            8,
                            fill,
                            Some(egui::Stroke::new(1.0_f32, border)),
                        );

                        let text_color = if is_selected { p.accent } else { p.text };
                        let center = if is_selected {
                            rect.center() - egui::vec2(6.0, 0.0)
                        } else {
                            rect.center()
                        };
                        theme::paint_text(
                            painter,
                            center,
                            egui::Align2::CENTER_CENTER,
                            format!("{size}×{size}"),
                            theme::font(11.5, Weight::SemiBold),
                            text_color,
                        );

                        if is_selected {
                            let dot = theme::snap_pos(
                                painter,
                                egui::pos2(rect.right() - 11.0, rect.center().y),
                            );
                            painter.circle_filled(dot, 6.0, p.accent);
                            icons::check(
                                painter,
                                egui::Rect::from_center_size(dot, egui::vec2(8.0, 8.0)),
                                p.on_accent,
                            );
                        }

                        if resp.clicked() {
                            clicked_indices.push(slot_idx);
                        }
                    }
                    clicked_indices
                })
                .inner;
            for idx in toggled {
                selected[idx] = !selected[idx];
            }
        }

        ui.add_space(2.0);
        ui.horizontal(|ui| {
            ui.label(theme::txt("Canvas Fit", 12.0, Weight::SemiBold, p.text));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(theme::txt(
                    current_mode.hint(),
                    11.0,
                    Weight::Regular,
                    p.muted,
                ));
            });
        });

        next_mode = fit_control(ui, current_mode);
    });

    app.selected_sizes = selected;
    if let Some(mode) = next_mode {
        app.set_fit_mode(ctx, mode);
    }
}

fn fit_control(ui: &mut egui::Ui, current: FitMode) -> Option<FitMode> {
    let p = PALETTE;
    let width = ui.available_width().floor();
    let (outer, _) = ui.allocate_exact_size(egui::vec2(width, 32.0), egui::Sense::hover());
    let painter = ui.painter();
    theme::paint_box(
        painter,
        outer,
        7,
        p.surface_alt,
        Some(egui::Stroke::new(1.0_f32, p.border)),
    );

    let count = FitMode::ALL.len();
    let inner_w = (outer.width() - 6.0).floor();
    let base_seg_w = (inner_w / count as f32).floor();

    let mut chosen = None;
    for (index, &mode) in FitMode::ALL.iter().enumerate() {
        let x = outer.min.x + 3.0 + index as f32 * base_seg_w;
        let seg_w = if index + 1 == count {
            (outer.max.x - 3.0 - x).max(base_seg_w)
        } else {
            base_seg_w
        };
        let rect =
            egui::Rect::from_min_size(egui::pos2(x, outer.min.y + 3.0), egui::vec2(seg_w, 26.0));
        let resp = ui.interact(
            rect,
            ui.id().with(("fit_mode", index)),
            egui::Sense::click(),
        );
        let active = current == mode;

        if active {
            theme::paint_shadow(painter, rect, 6, theme::CONTROL_SHADOW);
            theme::paint_box(
                painter,
                rect,
                6,
                p.surface,
                Some(egui::Stroke::new(1.0_f32, p.border)),
            );
        } else if resp.hovered() {
            theme::paint_box(painter, rect, 5, p.surface_hover, None);
        }

        let weight = if active {
            Weight::SemiBold
        } else {
            Weight::Regular
        };
        let color = if active { p.text } else { p.muted };
        theme::paint_text(
            painter,
            rect.center(),
            egui::Align2::CENTER_CENTER,
            mode.label(),
            theme::font(11.5, weight),
            color,
        );

        if resp.clicked() {
            chosen = Some(mode);
        }
    }
    chosen
}
