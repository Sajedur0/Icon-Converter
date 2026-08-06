pub(crate) const ACCENT: egui::Color32 = egui::Color32::from_rgb(37, 99, 235);
pub(crate) const ACCENT_HOVER: egui::Color32 = egui::Color32::from_rgb(29, 78, 216);
pub(crate) const BG: egui::Color32 = egui::Color32::WHITE;
pub(crate) const CARD: egui::Color32 = egui::Color32::from_rgb(250, 250, 250);
pub(crate) const CARD_HOVER: egui::Color32 = egui::Color32::from_rgb(232, 232, 232);
pub(crate) const CARD_SOFT: egui::Color32 = egui::Color32::from_rgb(240, 240, 240);
pub(crate) const BORDER: egui::Color32 = egui::Color32::from_rgb(170, 170, 170);
pub(crate) const BORDER_HOVER: egui::Color32 = egui::Color32::from_rgb(90, 90, 90);
pub(crate) const TEXT: egui::Color32 = egui::Color32::BLACK;
pub(crate) const MUTED: egui::Color32 = egui::Color32::from_rgb(96, 96, 96);
pub(crate) const SUCCESS: egui::Color32 = egui::Color32::BLACK;
pub(crate) const ERROR: egui::Color32 = egui::Color32::BLACK;

pub(crate) fn apply(ctx: &egui::Context) {
    let mut visuals = egui::Visuals::light();
    visuals.panel_fill = BG;
    visuals.window_fill = CARD;
    visuals.window_stroke = egui::Stroke::new(1.0_f32, BORDER);
    visuals.window_rounding = egui::Rounding::same(12.0);
    visuals.extreme_bg_color = CARD_SOFT;
    visuals.faint_bg_color = CARD_SOFT;
    visuals.override_text_color = Some(TEXT);
    visuals.hyperlink_color = ACCENT;
    visuals.selection.bg_fill = ACCENT;
    visuals.selection.stroke = egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(90, 90, 90));

    let corner = egui::Rounding::same(8.0);
    visuals.widgets.noninteractive.rounding = corner;
    visuals.widgets.inactive.rounding = corner;
    visuals.widgets.hovered.rounding = corner;
    visuals.widgets.active.rounding = corner;
    visuals.widgets.noninteractive.bg_fill = CARD_SOFT;
    visuals.widgets.inactive.bg_fill = CARD;
    visuals.widgets.hovered.bg_fill = ACCENT_HOVER;
    visuals.widgets.active.bg_fill = ACCENT_HOVER;
    visuals.widgets.hovered.weak_bg_fill = ACCENT_HOVER;
    visuals.widgets.active.weak_bg_fill = ACCENT_HOVER;
    visuals.widgets.inactive.bg_stroke = egui::Stroke::new(1.0_f32, BORDER);
    visuals.widgets.hovered.bg_stroke = egui::Stroke::new(1.0_f32, BORDER_HOVER);
    visuals.widgets.active.bg_stroke = egui::Stroke::new(1.0_f32, ACCENT);
    ctx.set_visuals(visuals);
}
