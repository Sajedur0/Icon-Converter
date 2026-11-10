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
        .rounding(8.0)
        .stroke(egui::Stroke::new(1.0_f32, accent))
        .inner_margin(egui::Margin::symmetric(10.0, 8.0))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 8.0;
                let (badge, _) =
                    ui.allocate_exact_size(egui::vec2(18.0, 18.0), egui::Sense::hover());
                let painter = ui.painter();
                painter.circle_filled(badge.center(), 9.0, accent);
                let inner = egui::Rect::from_center_size(badge.center(), egui::vec2(10.0, 10.0));
                match status.kind {
                    StatusKind::Success => icons::check(painter, inner, p.on_accent),
                    StatusKind::Error => icons::close(painter, inner, p.on_accent),
                }

                ui.vertical(|ui| {
                    ui.spacing_mut().item_spacing.y = 0.0;
                    ui.label(theme::txt(
                        status.message.clone(),
                        12.0,
                        Weight::SemiBold,
                        accent,
                    ));
                    if let Some(detail) = &status.detail {
                        ui.label(theme::txt(detail.clone(), 10.5, Weight::Regular, p.muted));
                    }
                });

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let (rect, resp) =
                        ui.allocate_exact_size(egui::vec2(22.0, 22.0), egui::Sense::click());
                    icons::close(ui.painter(), rect.shrink(5.5), p.muted);
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
    ui.add_space(8.0);
}
