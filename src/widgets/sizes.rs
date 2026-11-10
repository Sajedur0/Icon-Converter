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
                ui.label(theme::txt("Output Sizes", 11.0, Weight::SemiBold, p.text));
                ui.label(theme::txt(
                    format!("{selected_count}/{}", converter::AVAILABLE_SIZES.len()),
                    9.5,
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

        let width = ui.available_width();
        let gap = 6.0;
        let chip_width = (width - gap * (COLUMNS - 1) as f32) / COLUMNS as f32;

        for (row_idx, row) in converter::AVAILABLE_SIZES.chunks(COLUMNS).enumerate() {
            let base_idx = row_idx * COLUMNS;
            let toggled = ui
                .horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = gap;
                    let mut clicked_indices: Vec<usize> = Vec::new();
                    for (col_idx, &size) in row.iter().enumerate() {
                        let slot_idx = base_idx + col_idx;
                        let (rect, resp) = ui.allocate_exact_size(
                            egui::vec2(chip_width, 31.0),
                            egui::Sense::click(),
                        );
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
                        painter.rect_filled(rect, 7.0, fill);
                        painter.rect_stroke(
                            rect,
                            7.0,
                            egui::Stroke::new(if is_selected { 1.3_f32 } else { 1.0_f32 }, border),
                        );

                        let text_color = if is_selected { p.accent } else { p.text };
                        let center = if is_selected {
                            rect.center() - egui::vec2(5.5, 0.0)
                        } else {
                            rect.center()
                        };
                        painter.text(
                            center,
                            egui::Align2::CENTER_CENTER,
                            format!("{size}×{size}"),
                            theme::font(10.5, Weight::SemiBold),
                            text_color,
                        );

                        if is_selected {
                            let dot = egui::pos2(rect.right() - 10.5, rect.center().y);
                            painter.circle_filled(dot, 5.5, p.accent);
                            icons::check(
                                painter,
                                egui::Rect::from_center_size(dot, egui::vec2(7.5, 7.5)),
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
            ui.label(theme::txt("Canvas Fit", 11.0, Weight::SemiBold, p.text));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(theme::txt(
                    current_mode.hint(),
                    9.5,
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
    let width = ui.available_width();
    let (outer, _) = ui.allocate_exact_size(egui::vec2(width, 30.0), egui::Sense::hover());
    let painter = ui.painter();
    painter.rect_filled(outer, 7.0, p.surface_alt);
    painter.rect_stroke(outer, 7.0, egui::Stroke::new(1.0_f32, p.border));

    let count = FitMode::ALL.len() as f32;
    let seg_width = (outer.width() - 6.0) / count;

    let mut chosen = None;
    for (index, &mode) in FitMode::ALL.iter().enumerate() {
        let rect = egui::Rect::from_min_size(
            outer.min + egui::vec2(3.0 + index as f32 * seg_width, 3.0),
            egui::vec2(seg_width, 24.0),
        );
        let resp = ui.interact(
            rect,
            ui.id().with(("fit_mode", index)),
            egui::Sense::click(),
        );
        let active = current == mode;

        if active {
            painter.rect_filled(rect, 5.0, p.surface);
            painter.rect_stroke(rect, 5.0, egui::Stroke::new(1.0_f32, p.border_strong));
        } else if resp.hovered() {
            painter.rect_filled(rect, 5.0, p.surface_hover);
        }

        let weight = if active {
            Weight::SemiBold
        } else {
            Weight::Regular
        };
        let color = if active { p.text } else { p.muted };
        painter.text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            mode.label(),
            theme::font(10.5, weight),
            color,
        );

        if resp.clicked() {
            chosen = Some(mode);
        }
    }
    chosen
}
