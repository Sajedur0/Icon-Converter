//! Compact professional "About" dialog.
//!
//! The card has a fixed size. The close button sits in the top-right corner and
//! the Done button is pinned to the bottom of the card, so the content above
//! them can never push either control outside the frame.

use crate::app::IcoConverterApp;
use crate::icons;
use crate::theme::{self, Weight, PALETTE};

const CARD_WIDTH: f32 = 344.0;
const CARD_HEIGHT: f32 = 392.0;
const CARD_PADDING: f32 = 20.0;
const DONE_HEIGHT: f32 = 34.0;
const DONE_GAP: f32 = 14.0;
const CLOSE_SIZE: f32 = 26.0;
const CLOSE_INSET: f32 = 12.0;

pub fn show(app: &mut IcoConverterApp, ctx: &egui::Context) {
    let p = PALETTE;

    egui::Area::new(egui::Id::new("about_modal"))
        .order(egui::Order::Foreground)
        .fade_in(false)
        .fixed_pos(ctx.screen_rect().min)
        .show(ctx, |ui| {
            let screen = ctx.screen_rect();
            let painter = ui.painter().clone();
            painter.rect_filled(screen, egui::CornerRadius::ZERO, p.overlay);

            let backdrop =
                ui.interact(screen, ui.id().with("about_backdrop"), egui::Sense::click());

            let card = theme::snap_rect(
                &painter,
                egui::Rect::from_center_size(
                    screen.center(),
                    egui::vec2(CARD_WIDTH, CARD_HEIGHT),
                ),
            );
            theme::paint_shadow(&painter, card, 18, theme::MODAL_SHADOW);
            theme::paint_box(
                &painter,
                card,
                18,
                p.surface,
                Some(egui::Stroke::new(1.0_f32, p.border)),
            );

            let accent_line = egui::Rect::from_center_size(
                egui::pos2(card.center().x, card.top() + 1.0),
                egui::vec2(52.0, 3.0),
            );
            theme::paint_box(&painter, accent_line, 1, p.accent, None);

            // Close button: top-right corner, outside the content flow.
            let close_rect = theme::snap_rect(
                &painter,
                egui::Rect::from_min_size(
                    egui::pos2(
                        card.right() - CLOSE_INSET - CLOSE_SIZE,
                        card.top() + CLOSE_INSET,
                    ),
                    egui::vec2(CLOSE_SIZE, CLOSE_SIZE),
                ),
            );
            let close_resp =
                ui.interact(close_rect, ui.id().with("about_close"), egui::Sense::click());
            if close_resp.hovered() {
                theme::paint_box(&painter, close_rect, 7, p.surface_hover, None);
            }
            let close_color = if close_resp.hovered() { p.text } else { p.subtle };
            icons::close(&painter, close_rect.shrink(7.0), close_color);

            // Done button: pinned to the bottom of the card's inner area.
            let inner = card.shrink(CARD_PADDING);
            let done_rect = theme::snap_rect(
                &painter,
                egui::Rect::from_min_size(
                    egui::pos2(inner.left(), inner.bottom() - DONE_HEIGHT),
                    egui::vec2(inner.width(), DONE_HEIGHT),
                ),
            );
            let content_rect = egui::Rect::from_min_max(
                inner.min,
                egui::pos2(inner.right(), done_rect.top() - DONE_GAP),
            );

            ui.scope_builder(egui::UiBuilder::new().max_rect(content_rect), |ui| {
                ui.spacing_mut().item_spacing = egui::vec2(6.0, 6.0);
                ui.vertical_centered(|ui| {
                    ui.add_space(4.0);
                    let (mark, _) =
                        ui.allocate_exact_size(egui::vec2(54.0, 60.0), egui::Sense::hover());
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
                            ui.set_width(inner.width() - 24.0);
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
                });
            });

            let done_resp =
                ui.interact(done_rect, ui.id().with("about_done"), egui::Sense::click());
            let done_fill = if done_resp.hovered() {
                p.accent_hover
            } else {
                p.accent
            };
            theme::paint_shadow(&painter, done_rect, 9, theme::CONTROL_SHADOW);
            theme::paint_box(&painter, done_rect, 9, done_fill, None);
            theme::paint_text(
                &painter,
                done_rect.center(),
                egui::Align2::CENTER_CENTER,
                "Done",
                theme::font(12.0, Weight::SemiBold),
                p.on_accent,
            );

            let close_clicked = close_resp.clicked() || done_resp.clicked();
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
