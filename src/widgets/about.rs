//! Compact professional "About" dialog.

use crate::app::IcoConverterApp;
use crate::icons;
use crate::theme::{self, Weight, PALETTE};

pub fn show(app: &mut IcoConverterApp, ctx: &egui::Context) {
    let p = PALETTE;

    egui::Area::new(egui::Id::new("about_modal"))
        .order(egui::Order::Foreground)
        .fade_in(false)
        .fixed_pos(ctx.screen_rect().min)
        .show(ctx, |ui| {
            let screen = ctx.screen_rect();
            let painter = ui.painter();
            painter.rect_filled(screen, egui::CornerRadius::ZERO, p.overlay);

            let backdrop =
                ui.interact(screen, ui.id().with("about_backdrop"), egui::Sense::click());

            let card = theme::snap_rect(
                painter,
                egui::Rect::from_center_size(screen.center(), egui::vec2(320.0, 314.0)),
            );
            theme::paint_shadow(painter, card, 14, theme::MODAL_SHADOW);
            theme::paint_box(
                painter,
                card,
                14,
                p.surface,
                Some(egui::Stroke::new(1.0_f32, p.border)),
            );

            let close_clicked = ui
                .scope_builder(egui::UiBuilder::new().max_rect(card.shrink(16.0)), |ui| {
                    ui.spacing_mut().item_spacing = egui::vec2(6.0, 5.0);

                    let x_clicked = ui
                        .with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                            let (rect, resp) = ui
                                .allocate_exact_size(egui::vec2(22.0, 22.0), egui::Sense::click());
                            icons::close(ui.painter(), rect.shrink(5.0), p.muted);
                            resp.clicked()
                        })
                        .inner;

                    let button_clicked = ui
                        .vertical_centered(|ui| {
                            let (mark, _) = ui
                                .allocate_exact_size(egui::vec2(38.0, 38.0), egui::Sense::hover());
                            let painter = ui.painter();
                            theme::paint_shadow(painter, mark, 10, theme::CONTROL_SHADOW);
                            theme::paint_box(painter, mark, 10, p.accent, None);
                            icons::image(painter, mark.shrink(8.5), p.on_accent);

                            ui.add_space(3.0);
                            ui.label(theme::txt("ICO Converter", 14.0, Weight::SemiBold, p.text));
                            ui.label(theme::txt(
                                format!(
                                    "v{} · Professional Icon Utility",
                                    env!("CARGO_PKG_VERSION")
                                ),
                                11.0,
                                Weight::Regular,
                                p.muted,
                            ));
                            ui.add_space(6.0);

                            egui::Frame::NONE
                                .fill(p.surface_alt)
                                .corner_radius(egui::CornerRadius::same(7))
                                .inner_margin(egui::Margin::symmetric(10, 8))
                                .show(ui, |ui| {
                                    ui.spacing_mut().item_spacing = egui::vec2(6.0, 4.0);
                                    info_row(ui, "Author", "Sajedur Rahman Roni");
                                    info_row(ui, "Output", "16–512 px · 32-bit ICO");
                                    info_row(ui, "Engine", "Linear-Light Lanczos3");
                                    info_row(ui, "License", "Apache-2.0");
                                });

                            ui.add_space(6.0);
                            ui.hyperlink_to(
                                theme::txt("github.com/Sajedur0", 11.0, Weight::SemiBold, p.accent),
                                "https://github.com/Sajedur0",
                            );

                            ui.add_space(6.0);
                            let done = egui::Button::new(theme::txt(
                                "Done",
                                11.5,
                                Weight::SemiBold,
                                p.on_accent,
                            ))
                            .fill(p.accent)
                            .stroke(egui::Stroke::NONE)
                            .corner_radius(egui::CornerRadius::same(7))
                            .min_size(egui::vec2(104.0, 30.0));
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
        ui.label(theme::txt(key, 11.0, Weight::Regular, PALETTE.muted));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(theme::txt(value, 11.0, Weight::SemiBold, PALETTE.text));
        });
    });
}
