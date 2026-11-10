//! Compact inline status notification above the action bar.

use crate::app::{IcoConverterApp, StatusKind};
use crate::icons;
use crate::theme::{self, Weight, PALETTE};

pub fn show(app: &mut IcoConverterApp, ui: &mut egui::Ui) {
    let p = PALETTE;
    let status = match app.status.clone() {
        Some(status) => status,
        None => return,
    };

    let (accent, background) = match status.kind {
        StatusKind::Success => (p.success, p.success_soft),
        StatusKind::Error => (p.danger, p.danger_soft),
    };

    let dismissed = egui::Frame::none()
        .fill(background)
        .rounding(7.0)
        .stroke(egui::Stroke::new(1.0_f32, accent))
        .inner_margin(egui::Margin::symmetric(9.0, 6.0))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 7.0;
                let (badge, _) =
                    ui.allocate_exact_size(egui::vec2(16.0, 16.0), egui::Sense::hover());
                let painter = ui.painter();
                painter.circle_filled(badge.center(), 8.0, accent);
                let inner = egui::Rect::from_center_size(badge.center(), egui::vec2(9.0, 9.0));
                match status.kind {
                    StatusKind::Success => icons::check(painter, inner, p.on_accent),
                    StatusKind::Error => icons::close(painter, inner, p.on_accent),
                }

                ui.vertical(|ui| {
                    ui.spacing_mut().item_spacing.y = 0.0;
                    ui.label(theme::txt(
                        status.message.clone(),
                        10.5,
                        Weight::SemiBold,
                        accent,
                    ));
                    if let Some(detail) = &status.detail {
                        ui.label(theme::txt(detail.clone(), 9.5, Weight::Regular, p.muted));
                    }
                });

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let (rect, resp) =
                        ui.allocate_exact_size(egui::vec2(20.0, 20.0), egui::Sense::click());
                    icons::close(ui.painter(), rect.shrink(5.0), p.muted);
                    resp.clicked()
                })
                .inner
            })
            .inner
        })
        .inner;

    if dismissed {
        app.status = None;
    }
    ui.add_space(6.0);
}
