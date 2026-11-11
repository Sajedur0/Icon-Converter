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
                egui::Rect::from_center_size(screen.center(), egui::vec2(344.0, 366.0)),
            );
            theme::paint_shadow(painter, card, 18, theme::MODAL_SHADOW);
            theme::paint_box(
                painter,
                card,
                18,
                p.surface,
                Some(egui::Stroke::new(1.0_f32, p.border)),
            );

            let accent_line = egui::Rect::from_center_size(
                egui::pos2(card.center().x, card.top() + 1.0),
                egui::vec2(52.0, 3.0),
            );
            theme::paint_box(painter, accent_line, 1, p.accent, None);

            let close_clicked = ui
                .scope_builder(egui::UiBuilder::new().max_rect(card.shrink(20.0)), |ui| {
                    ui.spacing_mut().item_spacing = egui::vec2(6.0, 6.0);

                    let x_clicked = ui
                        .with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                            let (rect, resp) = ui
                                .allocate_exact_size(egui::vec2(26.0, 26.0), egui::Sense::click());
                            if resp.hovered() {
                                theme::paint_box(ui.painter(), rect, 7, p.surface_hover, None);
                            }
                            let color = if resp.hovered() { p.text } else { p.subtle };
                            icons::close(ui.painter(), rect.shrink(7.0), color);
                            resp.clicked()
                        })
                        .inner;

                    let button_clicked = ui
                        .vertical_centered(|ui| {
                            let (mark, _) = ui
                                .allocate_exact_size(egui::vec2(54.0, 60.0), egui::Sense::hover());
                            theme::paint_image_contained(ui.painter(), mark, &app.app_icon);

                            ui.add_space(5.0);
                            ui.label(theme::txt("ICO Converter", 17.0, Weight::SemiBold, p.text));
                            ui.add_space(1.0);
                            ui.label(theme::txt(
                                format!(
                                    "v{} · Professional Icon Utility",
                                    env!("CARGO_PKG_VERSION")
                                ),
                                11.5,
                                Weight::Regular,
                                p.muted,
                            ));
                            ui.add_space(10.0);

                            egui::Frame::NONE
                                .fill(p.surface_alt)
                                .corner_radius(egui::CornerRadius::same(10))
                                .stroke(egui::Stroke::new(1.0_f32, p.border))
                                .inner_margin(egui::Margin::symmetric(12, 10))
                                .show(ui, |ui| {
                                    ui.spacing_mut().item_spacing = egui::vec2(8.0, 7.0);
                                    info_row(ui, "Author", "Sajedur Rahman Roni");
                                    info_row(ui, "Output", "16–512 px · 32-bit ICO");
                                    info_row(ui, "Engine", "Linear-Light Lanczos3");
                                    info_row(ui, "License", "Apache-2.0");
                                });

                            ui.add_space(8.0);
                            ui.hyperlink_to(
                                theme::txt("github.com/Sajedur0", 11.5, Weight::SemiBold, p.accent),
                                "https://github.com/Sajedur0",
                            );

                            ui.add_space(10.0);
                            let width = ui.available_width().floor();
                            let (rect, resp) = ui
                                .allocate_exact_size(egui::vec2(width, 34.0), egui::Sense::click());
                            let fill = if resp.hovered() { p.accent_hover } else { p.accent };
                            theme::paint_shadow(painter, rect, 9, theme::CONTROL_SHADOW);
                            theme::paint_box(painter, rect, 9, fill, None);
                            theme::paint_text(
                                painter,
                                rect.center(),
                                egui::Align2::CENTER_CENTER,
                                "Done",
                                theme::font(12.0, Weight::SemiBold),
                                p.on_accent,
                            );
                            resp.clicked()
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
        ui.label(theme::txt(key, 11.5, Weight::Regular, PALETTE.muted));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(theme::txt(value, 11.5, Weight::SemiBold, PALETTE.text));
        });
    });
}
