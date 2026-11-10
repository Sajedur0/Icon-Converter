//! Compact light-mode "About" dialog.

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

            let card = egui::Rect::from_center_size(screen.center(), egui::vec2(340.0, 320.0));
            painter.rect_filled(card, 14.0, p.surface);
            painter.rect_stroke(card, 14.0, egui::Stroke::new(1.0_f32, p.border));

            let close_clicked = ui
                .allocate_ui_at_rect(card.shrink(18.0), |ui| {
                    ui.spacing_mut().item_spacing = egui::vec2(6.0, 6.0);

                    let x_clicked = ui
                        .with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                            let (rect, resp) = ui
                                .allocate_exact_size(egui::vec2(24.0, 24.0), egui::Sense::click());
                            icons::close(ui.painter(), rect.shrink(5.5), p.muted);
                            resp.clicked()
                        })
                        .inner;

                    let button_clicked = ui
                        .vertical_centered(|ui| {
                            let (mark, _) = ui
                                .allocate_exact_size(egui::vec2(44.0, 44.0), egui::Sense::hover());
                            let painter = ui.painter();
                            painter.rect_filled(mark, 12.0, p.accent);
                            icons::image(painter, mark.shrink(10.0), p.on_accent);

                            ui.add_space(4.0);
                            ui.label(theme::txt("ICO Converter", 17.0, Weight::SemiBold, p.text));
                            ui.label(theme::txt(
                                format!("v{}  ·  Minimal Light Edition", env!("CARGO_PKG_VERSION")),
                                11.0,
                                Weight::Regular,
                                p.muted,
                            ));
                            ui.add_space(8.0);

                            egui::Frame::none()
                                .fill(p.surface_alt)
                                .rounding(8.0)
                                .inner_margin(egui::Margin::symmetric(10.0, 8.0))
                                .show(ui, |ui| {
                                    ui.spacing_mut().item_spacing = egui::vec2(6.0, 5.0);
                                    info_row(ui, "Developer", "Sajedur Rahman Roni");
                                    info_row(ui, "Resolutions", "16 – 512 px (8 sizes)");
                                    info_row(
                                        ui,
                                        "Formats",
                                        "PNG · JPG · WEBP · BMP · GIF · TIFF · ICO",
                                    );
                                    info_row(ui, "License", "MIT");
                                });

                            ui.add_space(6.0);
                            ui.hyperlink_to(
                                theme::txt("github.com/Sajedur0", 11.5, Weight::SemiBold, p.accent),
                                "https://github.com/Sajedur0",
                            );

                            ui.add_space(8.0);
                            let close = egui::Button::new(theme::txt(
                                "Close",
                                12.5,
                                Weight::SemiBold,
                                p.on_accent,
                            ))
                            .fill(p.accent)
                            .stroke(egui::Stroke::NONE)
                            .rounding(8.0)
                            .min_size(egui::vec2(120.0, 32.0));
                            ui.add(close).clicked()
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
        ui.label(theme::txt(key, 11.0, Weight::Regular, PALETTE.muted));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(theme::txt(value, 11.0, Weight::SemiBold, PALETTE.text));
        });
    });
}
