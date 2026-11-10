//! Crisp light-only design tokens, pixel-snapped typography and widget helpers.

use egui::{
    emath::GuiRounding, Align2, Color32, CornerRadius, FontData, FontDefinitions, FontFamily,
    FontId, Margin, Painter, Pos2, Rect, RichText, Stroke, StrokeKind,
};
use std::sync::Arc;

pub const RADIUS_CARD: u8 = 10;
pub const RADIUS_CONTROL: u8 = 7;

const FONT_REGULAR: &str = "Inter-Regular";
const FONT_SEMI: &str = "Inter-SemiBold";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Weight {
    Regular,
    SemiBold,
}

/// High-contrast light palette engineered for crisp text legibility on standard & HiDPI screens.
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
    bg: Color32::from_rgb(246, 248, 252),
    surface: Color32::WHITE,
    surface_alt: Color32::from_rgb(241, 245, 249),
    surface_hover: Color32::from_rgb(232, 238, 247),
    border: Color32::from_rgb(220, 228, 238),
    border_strong: Color32::from_rgb(194, 206, 222),
    text: Color32::from_rgb(15, 23, 42),
    muted: Color32::from_rgb(65, 78, 98),
    subtle: Color32::from_rgb(94, 109, 130),
    accent: Color32::from_rgb(37, 99, 235),
    accent_hover: Color32::from_rgb(29, 78, 216),
    accent_soft: Color32::from_rgb(232, 240, 254),
    on_accent: Color32::WHITE,
    success: Color32::from_rgb(5, 115, 78),
    success_soft: Color32::from_rgb(232, 248, 241),
    danger: Color32::from_rgb(185, 28, 28),
    danger_soft: Color32::from_rgb(254, 236, 236),
    overlay: Color32::from_black_alpha(104),
    checker_a: Color32::WHITE,
    checker_b: Color32::from_rgb(228, 233, 242),
};

/// Installs subsetted Inter Regular & SemiBold (~46 KB total).
pub fn install_fonts(ctx: &egui::Context) {
    let mut fonts = FontDefinitions::empty();

    fonts.font_data.insert(
        FONT_REGULAR.to_owned(),
        Arc::new(FontData::from_static(include_bytes!(
            "../assets/fonts/Inter-Regular.ttf"
        ))),
    );
    fonts.font_data.insert(
        FONT_SEMI.to_owned(),
        Arc::new(FontData::from_static(include_bytes!(
            "../assets/fonts/Inter-SemiBold.ttf"
        ))),
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

pub fn snap_pos(painter: &Painter, pos: Pos2) -> Pos2 {
    pos.round_to_pixels(painter.pixels_per_point())
}

pub fn snap_rect(painter: &Painter, rect: Rect) -> Rect {
    rect.round_to_pixels(painter.pixels_per_point())
}

/// Lays out single-line text and snaps its origin to an exact physical pixel to prevent bilinear blur.
pub fn paint_text(
    painter: &Painter,
    anchor_pos: Pos2,
    align: Align2,
    text: impl Into<String>,
    font_id: FontId,
    color: Color32,
) {
    let galley = painter.layout_no_wrap(text.into(), font_id, color);
    let rect = align.anchor_size(anchor_pos, galley.size());
    let snapped = snap_pos(painter, rect.min);
    painter.galley(snapped, galley, color);
}

/// Paints a pixel-snapped rounded box with a crisp inside stroke.
pub fn paint_box(painter: &Painter, rect: Rect, radius: u8, fill: Color32, stroke: Option<Stroke>) {
    let snapped = snap_rect(painter, rect);
    let cr = CornerRadius::same(radius);
    if fill != Color32::TRANSPARENT {
        painter.rect_filled(snapped, cr, fill);
    }
    if let Some(s) = stroke {
        painter.rect_stroke(snapped, cr, s, StrokeKind::Inside);
    }
}

/// Applies crisp light-mode visuals and linear glyph coverage (eliminating dark-mode gamma halo).
pub fn apply_visuals(ctx: &egui::Context) {
    ctx.set_theme(egui::Theme::Light);
    ctx.send_viewport_cmd(egui::ViewportCommand::SetTheme(egui::SystemTheme::Light));
    ctx.tessellation_options_mut(|opts| {
        opts.round_text_to_pixels = true;
    });

    let p = PALETTE;
    let mut v = egui::Visuals::light();
    v.text_alpha_from_coverage = egui::epaint::AlphaFromCoverage::LIGHT_MODE_DEFAULT;

    v.panel_fill = p.bg;
    v.window_fill = p.surface;
    v.window_stroke = Stroke::new(1.0_f32, p.border);
    v.window_corner_radius = CornerRadius::same(RADIUS_CARD);
    v.window_shadow = egui::Shadow::NONE;
    v.popup_shadow = egui::Shadow::NONE;
    v.extreme_bg_color = p.surface_alt;
    v.faint_bg_color = p.surface_alt;
    v.override_text_color = Some(p.text);
    v.hyperlink_color = p.accent;
    v.selection.bg_fill = p.accent_soft;
    v.selection.stroke = Stroke::new(1.0_f32, p.accent);

    let corner = CornerRadius::same(RADIUS_CONTROL);
    v.widgets.noninteractive.corner_radius = corner;
    v.widgets.inactive.corner_radius = corner;
    v.widgets.hovered.corner_radius = corner;
    v.widgets.active.corner_radius = corner;

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

    ctx.set_visuals_of(egui::Theme::Light, v);

    ctx.style_mut_of(egui::Theme::Light, |style| {
        style.spacing.item_spacing = egui::vec2(8.0, 8.0);
        style.spacing.button_padding = egui::vec2(10.0, 5.0);
    });
}

/// Clean flat minimal card with a crisp 1px border.
pub fn card() -> egui::Frame {
    egui::Frame::NONE
        .fill(PALETTE.surface)
        .corner_radius(CornerRadius::same(RADIUS_CARD))
        .stroke(Stroke::new(1.0_f32, PALETTE.border))
        .inner_margin(Margin::same(12))
}

/// Compact outline button for secondary actions.
pub fn secondary_button(label: &str) -> egui::Button<'static> {
    egui::Button::new(txt(label, 11.0, Weight::SemiBold, PALETTE.text))
        .fill(PALETTE.surface)
        .stroke(Stroke::new(1.0_f32, PALETTE.border_strong))
        .corner_radius(CornerRadius::same(6))
        .min_size(egui::vec2(0.0, 26.0))
}

/// Borderless text action ("All", "None").
pub fn text_button(label: &str) -> egui::Button<'static> {
    egui::Button::new(txt(label, 11.0, Weight::SemiBold, PALETTE.accent)).frame(false)
}
