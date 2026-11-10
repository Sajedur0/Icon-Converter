//! Minimal light-only design tokens, subsetted typography and widget helpers.

use egui::{Color32, FontData, FontDefinitions, FontFamily, FontId, RichText, Rounding, Stroke};

pub const RADIUS_CARD: f32 = 10.0;
pub const RADIUS_CONTROL: f32 = 7.0;

const FONT_REGULAR: &str = "Inter-Regular";
const FONT_SEMI: &str = "Inter-SemiBold";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Weight {
    Regular,
    SemiBold,
}

/// Compile-time light palette — dark mode is intentionally disabled for a clean, minimal UI.
#[derive(Clone, Copy)]
pub struct Palette {
    pub bg: Color32,
    pub surface: Color32,
    pub surface_alt: Color32,
    pub surface_hover: Color32,
    pub border: Color32,
    pub border_strong: Color32,
    pub text: Color32,
    pub muted: Color32,
    pub subtle: Color32,
    pub accent: Color32,
    pub accent_hover: Color32,
    pub accent_soft: Color32,
    pub on_accent: Color32,
    pub success: Color32,
    pub success_soft: Color32,
    pub danger: Color32,
    pub danger_soft: Color32,
    pub overlay: Color32,
    pub checker_a: Color32,
    pub checker_b: Color32,
}

pub const PALETTE: Palette = Palette {
    bg: Color32::from_rgb(247, 249, 252),
    surface: Color32::WHITE,
    surface_alt: Color32::from_rgb(243, 246, 251),
    surface_hover: Color32::from_rgb(235, 240, 248),
    border: Color32::from_rgb(226, 232, 240),
    border_strong: Color32::from_rgb(203, 213, 225),
    text: Color32::from_rgb(15, 23, 42),
    muted: Color32::from_rgb(100, 116, 139),
    subtle: Color32::from_rgb(148, 163, 184),
    accent: Color32::from_rgb(37, 99, 235),
    accent_hover: Color32::from_rgb(29, 78, 216),
    accent_soft: Color32::from_rgba_unmultiplied(37, 99, 235, 20),
    on_accent: Color32::WHITE,
    success: Color32::from_rgb(5, 122, 85),
    success_soft: Color32::from_rgba_unmultiplied(16, 185, 129, 20),
    danger: Color32::from_rgb(185, 28, 28),
    danger_soft: Color32::from_rgba_unmultiplied(239, 68, 68, 18),
    overlay: Color32::from_black_alpha(96),
    checker_a: Color32::WHITE,
    checker_b: Color32::from_rgb(229, 234, 242),
};

/// Installs subsetted Inter Regular & SemiBold (~46 KB total, with `egui`'s default fonts disabled).
pub fn install_fonts(ctx: &egui::Context) {
    let mut fonts = FontDefinitions::empty();

    fonts.font_data.insert(
        FONT_REGULAR.to_owned(),
        FontData::from_static(include_bytes!("../assets/fonts/Inter-Regular.ttf")),
    );
    fonts.font_data.insert(
        FONT_SEMI.to_owned(),
        FontData::from_static(include_bytes!("../assets/fonts/Inter-SemiBold.ttf")),
    );

    fonts.families.insert(
        FontFamily::Proportional,
        vec![FONT_REGULAR.to_owned(), FONT_SEMI.to_owned()],
    );
    fonts
        .families
        .insert(FontFamily::Monospace, vec![FONT_REGULAR.to_owned()]);
    fonts.families.insert(
        FontFamily::Name(FONT_SEMI.into()),
        vec![FONT_SEMI.to_owned(), FONT_REGULAR.to_owned()],
    );

    ctx.set_fonts(fonts);
}

pub fn family(weight: Weight) -> FontFamily {
    static SEMI: std::sync::OnceLock<FontFamily> = std::sync::OnceLock::new();

    match weight {
        Weight::Regular => FontFamily::Proportional,
        Weight::SemiBold => SEMI
            .get_or_init(|| FontFamily::Name(FONT_SEMI.into()))
            .clone(),
    }
}

pub fn font(size: f32, weight: Weight) -> FontId {
    FontId::new(size, family(weight))
}

pub fn txt(text: impl Into<String>, size: f32, weight: Weight, color: Color32) -> RichText {
    RichText::new(text).font(font(size, weight)).color(color)
}

/// Applies crisp light-mode visuals once at startup.
pub fn apply_visuals(ctx: &egui::Context) {
    let p = PALETTE;
    let mut v = egui::Visuals::light();

    v.panel_fill = p.bg;
    v.window_fill = p.surface;
    v.window_stroke = Stroke::new(1.0_f32, p.border);
    v.window_rounding = Rounding::same(RADIUS_CARD);
    v.window_shadow = egui::Shadow::NONE;
    v.popup_shadow = egui::Shadow::NONE;
    v.extreme_bg_color = p.surface_alt;
    v.faint_bg_color = p.surface_alt;
    v.override_text_color = Some(p.text);
    v.hyperlink_color = p.accent;
    v.selection.bg_fill = p.accent_soft;
    v.selection.stroke = Stroke::new(1.0_f32, p.accent);

    let corner = Rounding::same(RADIUS_CONTROL);
    v.widgets.noninteractive.rounding = corner;
    v.widgets.inactive.rounding = corner;
    v.widgets.hovered.rounding = corner;
    v.widgets.active.rounding = corner;

    v.widgets.noninteractive.bg_fill = p.surface_alt;
    v.widgets.noninteractive.weak_bg_fill = p.surface_alt;
    v.widgets.noninteractive.fg_stroke = Stroke::new(1.0_f32, p.muted);
    v.widgets.inactive.bg_fill = p.surface_alt;
    v.widgets.inactive.weak_bg_fill = p.surface;
    v.widgets.inactive.bg_stroke = Stroke::new(1.0_f32, p.border);
    v.widgets.inactive.fg_stroke = Stroke::new(1.0_f32, p.text);
    v.widgets.hovered.bg_fill = p.surface_hover;
    v.widgets.hovered.weak_bg_fill = p.surface_hover;
    v.widgets.hovered.bg_stroke = Stroke::new(1.0_f32, p.border_strong);
    v.widgets.hovered.fg_stroke = Stroke::new(1.0_f32, p.accent);
    v.widgets.active.bg_fill = p.accent_soft;
    v.widgets.active.weak_bg_fill = p.accent_soft;
    v.widgets.active.bg_stroke = Stroke::new(1.0_f32, p.accent);
    v.widgets.active.fg_stroke = Stroke::new(1.0_f32, p.accent);

    ctx.set_visuals(v);

    let mut style = (*ctx.style()).clone();
    style.spacing.item_spacing = egui::vec2(7.0, 8.0);
    style.spacing.button_padding = egui::vec2(9.0, 5.0);
    ctx.set_style(style);
}

/// Clean flat minimal card with a crisp 1px border.
pub fn card() -> egui::Frame {
    egui::Frame::none()
        .fill(PALETTE.surface)
        .rounding(Rounding::same(RADIUS_CARD))
        .stroke(Stroke::new(1.0_f32, PALETTE.border))
        .inner_margin(egui::Margin::same(12.0))
}

/// Compact outline button for secondary actions.
pub fn secondary_button(label: &str) -> egui::Button<'static> {
    egui::Button::new(txt(label, 10.0, Weight::SemiBold, PALETTE.text))
        .fill(PALETTE.surface)
        .stroke(Stroke::new(1.0_f32, PALETTE.border_strong))
        .rounding(Rounding::same(6.0))
        .min_size(egui::vec2(0.0, 24.0))
}

/// Borderless text action ("All", "None").
pub fn text_button(label: &str) -> egui::Button<'static> {
    egui::Button::new(txt(label, 10.0, Weight::SemiBold, PALETTE.accent)).frame(false)
}
