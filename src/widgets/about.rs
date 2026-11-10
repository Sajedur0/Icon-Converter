//! Compact professional "About" dialog.

use crate::app::IcoConverterApp;
use crate::icons;
use crate::theme::{self, Weight, PALETTE};

pub fn show(app: &mut IcoConverterApp, ctx: &egui::Context) {
    let p = PALETTE;

    egui::Area::new(egui::Id::new("about_modal"))
        .order(egui::Order::Foreground)
        .fixed_pos(ctx.screen_rect().min)
        .show(ctx, |ui| {
            let screen = ctx.screen_rect();
            let painter = ui.painter();
            painter.rect_filled(screen, 0.0, p.overlay);

            let backdrop =
                ui.interact(screen, ui.id().with("about_backdrop"), egui::Sense::click());

            let card = egui::Rect::from_center_size(screen.center(), egui::vec2(300.0, 266.0));
            painter.rect_filled(card, 12.0, p.surface);
            painter.rect_stroke(card, 12.0, egui::Stroke::new(1.0_f32, p.border));

            let close_clicked = ui
                .allocate_ui_at_rect(card.shrink(14.0), |ui| {
                    ui.spacing_mut().item_spacing = egui::vec2(5.0, 5.0);

                    let x_clicked = ui
                        .with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                            let (rect, resp) = ui
                                .allocate_exact_size(egui::vec2(20.0, 20.0), egui::Sense::click());
                            icons::close(ui.painter(), rect.shrink(4.5), p.muted);
                            resp.clicked()
                        })
                        .inner;

                    let button_clicked = ui
                        .vertical_centered(|ui| {
                            let (mark, _) = ui
                                .allocate_exact_size(egui::vec2(36.0, 36.0), egui::Sense::hover());
                            let painter = ui.painter();
                            painter.rect_filled(mark, 9.0, p.accent);
                            icons::image(painter, mark.shrink(8.0), p.on_accent);

                            ui.add_space(2.0);
                            ui.label(theme::txt("ICO Converter", 13.5, Weight::SemiBold, p.text));
                            ui.label(theme::txt(
                                format!(
                                    "v{} · Professional Icon Utility",
                                    env!("CARGO_PKG_VERSION")
                                ),
                                9.5,
                                Weight::Regular,
                                p.muted,
                            ));
                            ui.add_space(6.0);

                            egui::Frame::none()
                                .fill(p.surface_alt)
                                .rounding(7.0)
                                .inner_margin(egui::Margin::symmetric(10.0, 7.0))
                                .show(ui, |ui| {
                                    ui.spacing_mut().item_spacing = egui::vec2(5.0, 4.0);
                                    info_row(ui, "Author", "Sajedur Rahman Roni");
                                    info_row(ui, "Output", "16–512 px · 32-bit ICO");
                                    info_row(ui, "Engine", "Linear-Light Lanczos3");
                                    info_row(ui, "License", "MIT");
                                });

                            ui.add_space(5.0);
                            ui.hyperlink_to(
                                theme::txt("github.com/Sajedur0", 10.0, Weight::SemiBold, p.accent),
                                "https://github.com/Sajedur0",
                            );

                            ui.add_space(6.0);
                            let done = egui::Button::new(theme::txt(
                                "Done",
                                10.5,
                                Weight::SemiBold,
                                p.on_accent,
                            ))
                            .fill(p.accent)
                            .stroke(egui::Stroke::NONE)
                            .rounding(7.0)
                            .min_size(egui::vec2(96.0, 28.0));
                            ui.add(done).clicked()
                        })
                        .inner;

                    x_clicked || button_clicked
                })
                .inner;

            let outside_clicked = backdrop.clicked()
                && backdrop
                    .interact_pointer_pos()
                    .is_some_and(|pos| !card.contains(pos));

            if close_clicked || outside_clicked {
                app.show_about = false;
            }
        });
}

fn info_row(ui: &mut egui::Ui, key: &str, value: &str) {
    ui.horizontal(|ui| {
        ui.label(theme::txt(key, 10.0, Weight::Regular, PALETTE.muted));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(theme::txt(value, 10.0, Weight::SemiBold, PALETTE.text));
        });
    });
}
