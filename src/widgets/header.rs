//! Top application bar: brand, tagline and quick actions.

use crate::app::IcoConverterApp;
use crate::icons;
use crate::theme::{self, Palette, Weight};

enum HeaderIcon {
    Info,
    Theme,
}

fn icon_button(
    ui: &mut egui::Ui,
    p: &Palette,
    dark: bool,
    kind: HeaderIcon,
    tooltip: &str,
) -> egui::Response {
    let (rect, resp) = ui.allocate_exact_size(egui::vec2(34.0, 34.0), egui::Sense::click());
    let hovered = resp.hovered();
    let painter = ui.painter();

    if hovered {
        painter.rect_filled(rect, 9.0, p.surface_hover);
    }
    let color = if hovered { p.accent } else { p.muted };
    let inner = rect.shrink(8.0);
    match kind {
        HeaderIcon::Info => icons::info(painter, inner, color),
        HeaderIcon::Theme => {
            if dark {
                icons::sun(painter, inner, color);
            } else {
                let bg = if hovered { p.surface_hover } else { p.surface };
                icons::moon(painter, inner, color, bg);
            }
        }
    }
    resp.on_hover_text(tooltip)
}

pub fn show(app: &mut IcoConverterApp, ui: &mut egui::Ui, ctx: &egui::Context, p: &Palette) {
    let dark = app.dark_mode;

    let (info_clicked, theme_clicked) = ui
        .horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 12.0;

            let (mark, _) = ui.allocate_exact_size(egui::vec2(40.0, 40.0), egui::Sense::hover());
            let painter = ui.painter();
            painter.rect_filled(mark, 11.0, p.accent);
            icons::image(painter, mark.shrink(9.0), p.on_accent);

            ui.vertical(|ui| {
                ui.spacing_mut().item_spacing.y = 1.0;
                ui.label(theme::txt("ICO Converter", 17.0, Weight::SemiBold, p.text));
                ui.label(theme::txt(
                    "Image to Windows icon · fast & offline",
                    11.5,
                    Weight::Regular,
                    p.muted,
                ));
            });

            let responses = ui
                .with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.spacing_mut().item_spacing.x = 8.0;
                    let info = icon_button(ui, p, dark, HeaderIcon::Info, "About this app");
                    let tooltip = if dark {
                        "Switch to light mode"
                    } else {
                        "Switch to dark mode"
                    };
                    let switch = icon_button(ui, p, dark, HeaderIcon::Theme, tooltip);
                    (info.clicked(), switch.clicked())
                })
                .inner;
            responses
        })
        .inner;

    if info_clicked {
        app.show_about = !app.show_about;
    }
    if theme_clicked {
        app.set_dark_mode(ctx, !dark);
    }
}
