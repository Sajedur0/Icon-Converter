//! Centred "About" dialog.

use crate::app::IcoConverterApp;
use crate::icons;
use crate::theme::{self, Palette, Weight};

pub fn show(app: &mut IcoConverterApp, ctx: &egui::Context, p: &Palette) {
    egui::Area::new(egui::Id::new("about_modal"))
        .order(egui::Order::Foreground)
        .fixed_pos(ctx.screen_rect().min)
        .show(ctx, |ui| {
            let screen = ctx.screen_rect();
            let painter = ui.painter();
            painter.rect_filled(screen, 0.0, p.overlay);

            let backdrop = ui.interact(
                screen,
                ui.id().with("about_backdrop"),
                egui::Sense::click(),
            );

            let card = egui::Rect::from_center_size(
                screen.center(),
                egui::vec2(380.0, 372.0),
            );
            painter.rect_filled(card, 18.0, p.surface);
            painter.rect_stroke(card, 18.0, egui::Stroke::new(1.0, p.border));

            let close_clicked = ui
                .allocate_ui_at_rect(card.shrink(22.0), |ui| {
                    ui.spacing_mut().item_spacing = egui::vec2(6.0, 8.0);

                    let x_clicked = ui
                        .with_layout(
                            egui::Layout::right_to_left(egui::Align::Min),
                            |ui| {
                                let (rect, resp) = ui.allocate_exact_size(
                                    egui::vec2(26.0, 26.0),
                                    egui::Sense::click(),
                                );
                                icons::close(ui.painter(), rect.shrink(6.0), p.muted);
                                resp.clicked()
                            },
                        )
                        .inner;

                    let button_clicked = ui
                        .vertical_centered(|ui| {
                            let (mark, _) = ui.allocate_exact_size(
                                egui::vec2(54.0, 54.0),
                                egui::Sense::hover(),
                            );
                            let painter = ui.painter();
                            painter.rect_filled(mark, 15.0, p.accent);
                            icons::image(painter, mark.shrink(12.0), p.on_accent);

                            ui.add_space(4.0);
                            ui.label(theme::txt(
                                "ICO Converter",
                                19.0,
                                Weight::Bold,
                                p.text,
                            ));
                            ui.label(theme::txt(
                                format!(
                                    "Version {}  ·  Rust + egui",
                                    env!("CARGO_PKG_VERSION")
                                ),
                                11.5,
                                Weight::Regular,
                                p.muted,
                            ));
                            ui.add_space(4.0);
                            ui.label(theme::txt(
                                "Build crisp, multi-resolution Windows icons from any image — fast, offline and free.",
                                12.0,
                                Weight::Regular,
                                p.muted,
                            ));
                            ui.add_space(8.0);

                            egui::Frame::none()
                                .fill(p.surface_alt)
                                .rounding(9.0)
                                .inner_margin(egui::Margin::symmetric(12.0, 9.0))
                                .show(ui, |ui| {
                                    ui.spacing_mut().item_spacing =
                                        egui::vec2(6.0, 6.0);
                                    info_row(ui, p, "Developer", "Sajedur Rahman Roni");
                                    info_row(
                                        ui,
                                        p,
                                        "Formats",
                                        "PNG · JPG · WEBP · BMP · GIF · TIFF",
                                    );
                                    info_row(ui, p, "License", "MIT");
                                });

                            ui.add_space(8.0);
                            ui.hyperlink_to(
                                theme::txt(
                                    "github.com/Sajedur0",
                                    12.0,
                                    Weight::SemiBold,
                                    p.accent,
                                ),
                                "https://github.com/Sajedur0",
                            );

                            ui.add_space(10.0);
                            let close = egui::Button::new(theme::txt(
                                "Close",
                                13.0,
                                Weight::SemiBold,
                                p.on_accent,
                            ))
                            .fill(p.accent)
                            .stroke(egui::Stroke::NONE)
                            .rounding(9.0)
                            .min_size(egui::vec2(150.0, 36.0));
                            ui.add(close).clicked()
                        })
                        .inner;

                    x_clicked || button_clicked
                })
                .inner;

            let outside_clicked = backdrop.clicked()
                && backdrop
                    .interact_pointer_pos()
                    .map_or(false, |pos| !card.contains(pos));

            if close_clicked || outside_clicked {
                app.show_about = false;
            }
        });
}

fn info_row(ui: &mut egui::Ui, p: &Palette, key: &str, value: &str) {
    ui.horizontal(|ui| {
        ui.label(theme::txt(key, 11.5, Weight::Regular, p.muted));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(theme::txt(value, 11.5, Weight::SemiBold, p.text));
        });
    });
}
