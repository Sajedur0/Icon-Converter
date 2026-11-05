//! Resolution picker and aspect-ratio handling.
//!
//! The widget works on a local snapshot of the selection and returns the new
//! state at the end of the frame. That keeps every nested `Ui` closure free of
//! mutable borrows of the application struct.

use crate::app::IcoConverterApp;
use crate::converter::{self, FitMode};
use crate::icons;
use crate::theme::{self, Palette, Weight};
use std::collections::HashSet;

const COLUMNS: usize = 4;

pub fn show(app: &mut IcoConverterApp, ui: &mut egui::Ui, p: &Palette) {
    let mut selected: HashSet<u32> = app.selected_sizes.clone();
    let mut fit_mode = app.fit_mode;

    theme::card(p).show(ui, |ui| {
        ui.spacing_mut().item_spacing = egui::vec2(8.0, 10.0);

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
            selected.clear();
        }
        if all_clicked {
            selected = converter::AVAILABLE_SIZES.iter().copied().collect();
        }

        // ---- Chips -------------------------------------------------------
        let width = ui.available_width();
        let gap = 8.0;
        let chip_width = (width - gap * (COLUMNS - 1) as f32) / COLUMNS as f32;

        for row in converter::AVAILABLE_SIZES.chunks(COLUMNS) {
            let toggled = ui
                .horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = gap;
                    let mut clicked_sizes: Vec<u32> = Vec::new();
                    for &size in row {
                        let (rect, resp) = ui.allocate_exact_size(
                            egui::vec2(chip_width, 40.0),
                            egui::Sense::click(),
                        );
                        let is_selected = selected.contains(&size);
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
                            egui::Stroke::new(if is_selected { 1.5 } else { 1.0 }, border),
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
                            clicked_sizes.push(size);
                        }
                    }
                    clicked_sizes
                })
                .inner;
            for size in toggled {
                if !selected.remove(&size) {
                    selected.insert(size);
                }
            }
            ui.add_space(gap);
        }

        // ---- Aspect ratio ------------------------------------------------
        ui.add_space(2.0);
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 8.0;
            let (icon_rect, _) =
                ui.allocate_exact_size(egui::vec2(18.0, 18.0), egui::Sense::hover());
            icons::layers(ui.painter(), icon_rect, p.muted);
            ui.label(theme::txt("Aspect ratio", 12.0, Weight::SemiBold, p.text));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(theme::txt(fit_mode.hint(), 11.0, Weight::Regular, p.muted));
            });
        });

        if let Some(mode) = fit_control(ui, p, fit_mode) {
            fit_mode = mode;
        }
    });

    app.selected_sizes = selected;
    app.fit_mode = fit_mode;
}

fn fit_control(ui: &mut egui::Ui, p: &Palette, current: FitMode) -> Option<FitMode> {
    let width = ui.available_width();
    let (outer, _) = ui.allocate_exact_size(egui::vec2(width, 38.0), egui::Sense::hover());
    let painter = ui.painter();
    painter.rect_filled(outer, 10.0, p.surface_alt);
    painter.rect_stroke(outer, 10.0, egui::Stroke::new(1.0, p.border));

    let half = (outer.width() - 8.0) / 2.0;
    let options = [
        (FitMode::Cover, FitMode::Cover.label()),
        (FitMode::Contain, FitMode::Contain.label()),
    ];

    let mut chosen = None;
    for (index, (mode, label)) in options.iter().enumerate() {
        let rect = egui::Rect::from_min_size(
            outer.min + egui::vec2(4.0 + index as f32 * half, 4.0),
            egui::vec2(half, 30.0),
        );
        let resp = ui.interact(
            rect,
            ui.id().with(("fit_mode", index)),
            egui::Sense::click(),
        );
        let active = current == *mode;

        if active {
            painter.rect_filled(rect, 7.0, p.surface);
            painter.rect_stroke(rect, 7.0, egui::Stroke::new(1.0, p.border_strong));
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
            *label,
            theme::font(12.0, weight),
            color,
        );

        if resp.clicked() {
            chosen = Some(*mode);
        }
    }
    chosen
}
