//! Resolution picker (16 × 16 through 512 × 512) and aspect-ratio handling.
//!
//! Operates on a stack-allocated `[bool; 8]` snapshot so nested `Ui` closures
//! never hold mutable borrows of `IcoConverterApp` or allocate on the heap.

use crate::app::IcoConverterApp;
use crate::converter::{self, FitMode};
use crate::icons;
use crate::theme::{self, Palette, Weight};

const COLUMNS: usize = 4;

pub fn show(app: &mut IcoConverterApp, ui: &mut egui::Ui, ctx: &egui::Context, p: &Palette) {
    let mut selected = app.selected_sizes;
    let current_mode = app.fit_mode;
    let mut next_mode = None;

    theme::card(p).show(ui, |ui| {
        ui.spacing_mut().item_spacing = egui::vec2(8.0, 8.0);

        // ---- Helper actions --------------------------------------------
        let (clear_clicked, all_clicked) = ui
            .horizontal(|ui| {
                ui.label(theme::txt(
                    "Choose the resolutions to embed",
                    11.5,
                    Weight::Regular,
                    p.muted,
                ));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.spacing_mut().item_spacing.x = 12.0;
                    let clear = ui.add(theme::text_button("Clear", p)).clicked();
                    let all = ui.add(theme::text_button("Select all", p)).clicked();
                    (clear, all)
                })
                .inner
            })
            .inner;
        if clear_clicked {
            selected = [false; converter::AVAILABLE_SIZES.len()];
        }
        if all_clicked {
            selected = [true; converter::AVAILABLE_SIZES.len()];
        }

        // ---- 2 × 4 Resolution chips (16 ..= 512) -----------------------
        let width = ui.available_width();
        let gap = 8.0;
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
                            egui::vec2(chip_width, 40.0),
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
                        painter.rect_filled(rect, 9.0, fill);
                        painter.rect_stroke(
                            rect,
                            9.0,
                            egui::Stroke::new(if is_selected { 1.5_f32 } else { 1.0_f32 }, border),
                        );

                        let text_color = if is_selected { p.accent } else { p.text };
                        let center = if is_selected {
                            rect.center() - egui::vec2(7.0, 0.0)
                        } else {
                            rect.center()
                        };
                        painter.text(
                            center,
                            egui::Align2::CENTER_CENTER,
                            format!("{size} × {size}"),
                            theme::font(12.0, Weight::SemiBold),
                            text_color,
                        );

                        if is_selected {
                            let dot = egui::pos2(rect.right() - 13.0, rect.center().y);
                            painter.circle_filled(dot, 7.0, p.accent);
                            icons::check(
                                painter,
                                egui::Rect::from_center_size(dot, egui::vec2(9.0, 9.0)),
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

        // ---- Aspect ratio / framing ------------------------------------
        ui.add_space(4.0);
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 8.0;
            let (icon_rect, _) =
                ui.allocate_exact_size(egui::vec2(18.0, 18.0), egui::Sense::hover());
            icons::layers(ui.painter(), icon_rect, p.muted);
            ui.label(theme::txt("Framing", 12.0, Weight::SemiBold, p.text));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(theme::txt(
                    current_mode.hint(),
                    11.0,
                    Weight::Regular,
                    p.muted,
                ));
            });
        });

        next_mode = fit_control(ui, p, current_mode);
    });

    app.selected_sizes = selected;
    if let Some(mode) = next_mode {
        app.set_fit_mode(ctx, mode);
    }
}

fn fit_control(ui: &mut egui::Ui, p: &Palette, current: FitMode) -> Option<FitMode> {
    let width = ui.available_width();
    let (outer, _) = ui.allocate_exact_size(egui::vec2(width, 38.0), egui::Sense::hover());
    let painter = ui.painter();
    painter.rect_filled(outer, 10.0, p.surface_alt);
    painter.rect_stroke(outer, 10.0, egui::Stroke::new(1.0_f32, p.border));

    let count = FitMode::ALL.len() as f32;
    let seg_width = (outer.width() - 8.0) / count;

    let mut chosen = None;
    for (index, &mode) in FitMode::ALL.iter().enumerate() {
        let rect = egui::Rect::from_min_size(
            outer.min + egui::vec2(4.0 + index as f32 * seg_width, 4.0),
            egui::vec2(seg_width, 30.0),
        );
        let resp = ui.interact(
            rect,
            ui.id().with(("fit_mode", index)),
            egui::Sense::click(),
        );
        let active = current == mode;

        if active {
            painter.rect_filled(rect, 7.0, p.surface);
            painter.rect_stroke(rect, 7.0, egui::Stroke::new(1.0_f32, p.border_strong));
        } else if resp.hovered() {
            painter.rect_filled(rect, 7.0, p.surface_hover);
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
            theme::font(12.0, weight),
            color,
        );

        if resp.clicked() {
            chosen = Some(mode);
        }
    }
    chosen
}
