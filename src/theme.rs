//! Design tokens for the whole application.
//!
//! One place for colours, typography and widget styling. Both the light and the
//! dark theme are built from the very same tokens, so every widget stays
//! consistent when the user switches appearance.

use egui::{Color32, FontData, FontDefinitions, FontFamily, FontId, RichText, Rounding, Stroke};

pub const RADIUS_CARD: f32 = 14.0;
pub const RADIUS_CONTROL: f32 = 9.0;

const FONT_REGULAR: &str = "Inter-Regular";
const FONT_SEMI: &str = "Inter-SemiBold";
const FONT_BOLD: &str = "Inter-Bold";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Weight {
    Regular,
    SemiBold,
    Bold,
}

/// All colours used by the UI. `dark` also tells egui which base visuals to use.
#[derive(Clone, Copy)]
pub struct Palette {
    pub dark: bool,
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
    pub shadow: Color32,
    pub overlay: Color32,
    pub checker_a: Color32,
    pub checker_b: Color32,
}

impl Palette {
    pub fn new(dark: bool) -> Self {
        if dark {
            Self::dark_theme()
        } else {
            Self::light_theme()
        }
    }

    fn light_theme() -> Self {
        Self {
            dark: false,
            bg: Color32::from_rgb(245, 247, 251),
            surface: Color32::WHITE,
            surface_alt: Color32::from_rgb(243, 246, 251),
            surface_hover: Color32::from_rgb(233, 238, 248),
            border: Color32::from_rgb(225, 231, 241),
            border_strong: Color32::from_rgb(199, 209, 226),
            text: Color32::from_rgb(17, 24, 39),
            muted: Color32::from_rgb(98, 110, 130),
            subtle: Color32::from_rgb(148, 160, 180),
            accent: Color32::from_rgb(79, 70, 229),
            accent_hover: Color32::from_rgb(64, 55, 202),
            accent_soft: Color32::from_rgba_unmultiplied(79, 70, 229, 20),
            on_accent: Color32::WHITE,
            success: Color32::from_rgb(6, 118, 71),
            success_soft: Color32::from_rgba_unmultiplied(16, 150, 100, 20),
            danger: Color32::from_rgb(180, 35, 24),
            danger_soft: Color32::from_rgba_unmultiplied(220, 38, 38, 18),
            shadow: Color32::from_black_alpha(18),
            overlay: Color32::from_black_alpha(105),
            checker_a: Color32::from_rgb(255, 255, 255),
            checker_b: Color32::from_rgb(226, 232, 243),
        }
    }

    fn dark_theme() -> Self {
        Self {
            dark: true,
            bg: Color32::from_rgb(13, 17, 24),
            surface: Color32::from_rgb(23, 29, 39),
            surface_alt: Color32::from_rgb(30, 37, 49),
            surface_hover: Color32::from_rgb(41, 50, 65),
            border: Color32::from_rgb(43, 53, 69),
            border_strong: Color32::from_rgb(63, 76, 97),
            text: Color32::from_rgb(238, 242, 249),
            muted: Color32::from_rgb(153, 165, 184),
            subtle: Color32::from_rgb(108, 121, 140),
            accent: Color32::from_rgb(124, 108, 246),
            accent_hover: Color32::from_rgb(146, 132, 255),
            accent_soft: Color32::from_rgba_unmultiplied(124, 108, 246, 42),
            on_accent: Color32::WHITE,
            success: Color32::from_rgb(74, 214, 154),
            success_soft: Color32::from_rgba_unmultiplied(74, 214, 154, 24),
            danger: Color32::from_rgb(255, 124, 116),
            danger_soft: Color32::from_rgba_unmultiplied(255, 124, 116, 24),
            shadow: Color32::from_black_alpha(90),
            overlay: Color32::from_black_alpha(165),
            checker_a: Color32::from_rgb(37, 44, 56),
            checker_b: Color32::from_rgb(49, 58, 73),
        }
    }
}

/// Installs Inter (regular / semibold / bold) while keeping egui's emoji
/// fallbacks. Bundled fonts mean the app looks identical on every machine.
pub fn install_fonts(ctx: &egui::Context) {
    let mut fonts = FontDefinitions::default();

    fonts.font_data.insert(
        FONT_REGULAR.to_owned(),
        FontData::from_static(include_bytes!("../assets/fonts/Inter-Regular.ttf")),
    );
    fonts.font_data.insert(
        FONT_SEMI.to_owned(),
        FontData::from_static(include_bytes!("../assets/fonts/Inter-SemiBold.ttf")),
    );
    fonts.font_data.insert(
        FONT_BOLD.to_owned(),
        FontData::from_static(include_bytes!("../assets/fonts/Inter-Bold.ttf")),
    );

    let emoji = vec!["NotoEmoji-Regular".to_owned(), "emoji-icon-font".to_owned()];

    let mut proportional = vec![
        FONT_REGULAR.to_owned(),
        FONT_SEMI.to_owned(),
        FONT_BOLD.to_owned(),
    ];
    proportional.extend(emoji.iter().cloned());
    fonts
        .families
        .insert(FontFamily::Proportional, proportional);

    fonts.families.insert(
        FontFamily::Monospace,
        vec![
            "Hack".to_owned(),
            FONT_REGULAR.to_owned(),
            "NotoEmoji-Regular".to_owned(),
        ],
    );

    fonts.families.insert(
        FontFamily::Name(FONT_SEMI.into()),
        vec![
            FONT_SEMI.to_owned(),
            FONT_REGULAR.to_owned(),
            "NotoEmoji-Regular".to_owned(),
        ],
    );
    fonts.families.insert(
        FontFamily::Name(FONT_BOLD.into()),
        vec![
            FONT_BOLD.to_owned(),
            FONT_SEMI.to_owned(),
            FONT_REGULAR.to_owned(),
            "NotoEmoji-Regular".to_owned(),
        ],
    );

    ctx.set_fonts(fonts);
}

pub fn family(weight: Weight) -> FontFamily {
    match weight {
        Weight::Regular => FontFamily::Proportional,
        Weight::SemiBold => FontFamily::Name(FONT_SEMI.into()),
        Weight::Bold => FontFamily::Name(FONT_BOLD.into()),
    }
}

pub fn font(size: f32, weight: Weight) -> FontId {
    FontId::new(size, family(weight))
}

/// Rich text with an explicit weight — egui's `.strong()` only changes colour,
/// so real weights have to come from a real font family.
pub fn txt(text: impl Into<String>, size: f32, weight: Weight, color: Color32) -> RichText {
    RichText::new(text).font(font(size, weight)).color(color)
}

/// Applies egui's global visuals from our palette.
pub fn apply_visuals(ctx: &egui::Context, p: &Palette) {
    let mut v = if p.dark {
        egui::Visuals::dark()
    } else {
        egui::Visuals::light()
    };

    v.panel_fill = p.bg;
    v.window_fill = p.surface;
    v.window_stroke = Stroke::new(1.0, p.border);
    v.window_rounding = Rounding::same(RADIUS_CARD);
    v.window_shadow = egui::Shadow {
        offset: egui::vec2(0.0, 10.0),
        blur: 28.0,
        spread: 0.0,
        color: p.shadow,
    };
    v.popup_shadow = v.window_shadow;
    v.extreme_bg_color = p.surface_alt;
    v.faint_bg_color = p.surface_alt;
    v.override_text_color = Some(p.text);
    v.hyperlink_color = p.accent;
    v.selection.bg_fill = p.accent_soft;
    v.selection.stroke = Stroke::new(1.0, p.accent);

    let corner = Rounding::same(RADIUS_CONTROL);
    v.widgets.noninteractive.rounding = corner;
    v.widgets.inactive.rounding = corner;
    v.widgets.hovered.rounding = corner;
    v.widgets.active.rounding = corner;

    v.widgets.noninteractive.bg_fill = p.surface_alt;
    v.widgets.noninteractive.weak_bg_fill = p.surface_alt;
    v.widgets.noninteractive.fg_stroke = Stroke::new(1.0, p.muted);
    v.widgets.inactive.bg_fill = p.surface_alt;
    v.widgets.inactive.weak_bg_fill = p.surface;
    v.widgets.inactive.bg_stroke = Stroke::new(1.0, p.border);
    v.widgets.inactive.fg_stroke = Stroke::new(1.0, p.text);
    v.widgets.hovered.bg_fill = p.surface_hover;
    v.widgets.hovered.weak_bg_fill = p.surface_hover;
    v.widgets.hovered.bg_stroke = Stroke::new(1.0, p.border_strong);
    v.widgets.hovered.fg_stroke = Stroke::new(1.0, p.accent);
    v.widgets.active.bg_fill = p.accent_soft;
    v.widgets.active.weak_bg_fill = p.accent_soft;
    v.widgets.active.bg_stroke = Stroke::new(1.0, p.accent);
    v.widgets.active.fg_stroke = Stroke::new(1.0, p.accent);

    ctx.set_visuals(v);

    let mut style = (*ctx.style()).clone();
    style.spacing.item_spacing = egui::vec2(10.0, 12.0);
    style.spacing.button_padding = egui::vec2(12.0, 8.0);
    ctx.set_style(style);
}

/// Standard elevated card used for every section.
pub fn card(p: &Palette) -> egui::Frame {
    egui::Frame::none()
        .fill(p.surface)
        .rounding(Rounding::same(RADIUS_CARD))
        .stroke(Stroke::new(1.0, p.border))
        .shadow(egui::Shadow {
            offset: egui::vec2(0.0, 3.0),
            blur: 12.0,
            spread: 0.0,
            color: p.shadow,
        })
        .inner_margin(egui::Margin::same(16.0))
}

/// Simple secondary/outline button used for small actions.
pub fn secondary_button(label: &str, p: &Palette) -> egui::Button<'static> {
    egui::Button::new(txt(label, 12.0, Weight::SemiBold, p.text))
        .fill(p.surface)
        .stroke(Stroke::new(1.0, p.border_strong))
        .rounding(Rounding::same(8.0))
        .min_size(egui::vec2(0.0, 32.0))
}

/// Borderless text action ("Select all", "Clear", ...).
pub fn text_button(label: &str, p: &Palette) -> egui::Button<'static> {
    egui::Button::new(txt(label, 11.5, Weight::SemiBold, p.accent)).frame(false)
}
