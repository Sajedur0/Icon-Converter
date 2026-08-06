use crate::converter::{self, AVAILABLE_SIZES};
use std::path::PathBuf;
use std::collections::HashSet;

const ACCENT: egui::Color32 = egui::Color32::from_rgb(37, 99, 235);
const ACCENT_HOVER: egui::Color32 = egui::Color32::from_rgb(29, 78, 216);
const BG: egui::Color32 = egui::Color32::WHITE;
const CARD: egui::Color32 = egui::Color32::from_rgb(250, 250, 250);
const CARD_HOVER: egui::Color32 = egui::Color32::from_rgb(232, 232, 232);
const CARD_SOFT: egui::Color32 = egui::Color32::from_rgb(240, 240, 240);
const BORDER: egui::Color32 = egui::Color32::from_rgb(170, 170, 170);
const BORDER_HOVER: egui::Color32 = egui::Color32::from_rgb(90, 90, 90);
const TEXT: egui::Color32 = egui::Color32::BLACK;
const MUTED: egui::Color32 = egui::Color32::from_rgb(96, 96, 96);
const SUCCESS: egui::Color32 = egui::Color32::BLACK;
const ERROR: egui::Color32 = egui::Color32::BLACK;

#[allow(dead_code)]
pub struct IcoConverterApp {
    pub source_path: Option<PathBuf>,
    pub preview: Option<egui::TextureHandle>,
    pub checker: egui::TextureHandle,
    pub selected_sizes: HashSet<u32>,
    pub status_message: String,
    pub status_is_error: bool,
    pub is_converting: bool,
    pub show_about: bool,
}

impl IcoConverterApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let ctx = cc.egui_ctx.clone();

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

        let mut cb = egui::ColorImage::new([16, 16], egui::Color32::TRANSPARENT);
        let c1 = egui::Color32::from_rgb(235, 235, 235);
        let c2 = egui::Color32::from_rgb(215, 215, 215);
        for y in 0..16 {
            for x in 0..16 {
                cb[(x, y)] = if (x / 8 + y / 8) % 2 == 0 { c1 } else { c2 };
            }
        }
        let checker = ctx.load_texture("checkerboard", cb, egui::TextureOptions::NEAREST);

        Self {
            source_path: None,
            preview: None,
            checker,
            selected_sizes: AVAILABLE_SIZES.iter().copied().collect(),
            status_message: String::new(),
            status_is_error: false,
            is_converting: false,
            show_about: false,
        }
    }

    fn handle_file_pick(&mut self, ctx: &egui::Context) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Images", &["png", "jpg", "jpeg", "bmp", "gif", "webp", "tiff"])
            .pick_file()
        {
            self.load_preview(ctx, &path);
            self.source_path = Some(path);
        }
    }

    fn load_preview(&mut self, ctx: &egui::Context, path: &std::path::Path) {
        match converter::load_image(path) {
            Ok(img) => {
                let rgba = img.to_rgba8();
                let (w, h) = rgba.dimensions();
                let color_image = egui::ColorImage::from_rgba_unmultiplied(
                    [w as usize, h as usize],
                    &rgba,
                );
                self.preview = Some(ctx.load_texture("preview", color_image, Default::default()));
                self.status_message = String::new();
                self.status_is_error = false;
            }
            Err(_) => {
                self.status_message = "Failed to load image — unsupported format or corrupt file".into();
                self.status_is_error = true;
            }
        }
    }

    fn convert_and_save(&mut self) {
        let Some(src) = &self.source_path.clone() else { return };
        let Ok(img) = converter::load_image(src) else {
            self.status_message = "Failed to load image".into();
            self.status_is_error = true;
            return;
        };

        let mut sizes: Vec<u32> = self.selected_sizes.iter().copied().collect();
        sizes.sort_unstable();

        if let Some(save_path) = rfd::FileDialog::new()
            .add_filter("ICO File", &["ico"])
            .set_file_name("icon.ico")
            .save_file()
        {
            let resized = converter::resize_to_sizes(&img, &sizes);
            match converter::export_ico(resized, &save_path) {
                Ok(_) => {
                    self.status_message = format!("Saved successfully: {}", save_path.display());
                    self.status_is_error = false;
                }
                Err(e) => {
                    self.status_message = format!("Failed: {e}");
                    self.status_is_error = true;
                }
            }
        }
    }

    fn header(&mut self, ui: &mut egui::Ui) {
        ui.add_space(2.0);
        let about_rect = egui::Rect::from_min_size(
            egui::pos2(ui.max_rect().right() - 38.0, ui.max_rect().top() + 4.0),
            egui::vec2(32.0, 28.0),
        );
        ui.allocate_ui_at_rect(about_rect, |ui| {
            let btn = egui::Button::new(
                egui::RichText::new("ℹ").size(20.0).color(ACCENT_HOVER),
            )
            .frame(false);
            if ui.add(btn).on_hover_text("About").clicked() {
                self.show_about = !self.show_about;
            }
        });
        ui.vertical_centered(|ui| {
            ui.label(egui::RichText::new("ICO Converter").size(30.0).strong().color(TEXT));
            ui.add_space(2.0);
            ui.label(
                egui::RichText::new("PNG · JPG · WEBP  →  .ICO")
                    .size(13.0)
                    .color(MUTED),
            );
        });
        ui.add_space(8.0);
    }

    fn drop_zone(&mut self, ui: &mut egui::Ui, ctx: &egui::Context, is_dragging: bool) {
        let full = ui.available_width();
        let (rect, resp) = ui.allocate_exact_size(egui::vec2(full, 190.0), egui::Sense::click());

        let hovered = resp.hovered() || is_dragging;
        let fill = if hovered { CARD_HOVER } else { CARD };
        let stroke = egui::Stroke::new(
            if hovered { 2.0_f32 } else { 1.0_f32 },
            if hovered { ACCENT } else { BORDER },
        );

        let painter = ui.painter();
        painter.rect_filled(rect, egui::Rounding::same(14.0), fill);
        painter.rect_stroke(rect, egui::Rounding::same(14.0), stroke);

        if resp.clicked() {
            self.handle_file_pick(ctx);
            return;
        }

        match &self.preview {
            Some(tex) => {
                let img = tex.size_vec2();
                let inner = rect.shrink2(egui::vec2(16.0, 30.0));
                let scale = (inner.width() / img.x).min(inner.height() / img.y).min(3.0);
                let draw = img * scale;
                let img_rect = egui::Rect::from_center_size(inner.center(), draw);
                let uv = egui::Rect::from_min_max(
                    egui::Pos2::ZERO,
                    egui::pos2(img_rect.width() / 16.0, img_rect.height() / 16.0),
                );
                painter.image(self.checker.id(), img_rect, uv, egui::Color32::WHITE);
                painter.image(
                    tex.id(),
                    img_rect,
                    egui::Rect::from_min_max(egui::Pos2::ZERO, egui::pos2(1.0, 1.0)),
                    egui::Color32::WHITE,
                );

                let name = self
                    .source_path
                    .as_ref()
                    .and_then(|p| p.file_name().map(|s| s.to_string_lossy().into_owned()))
                    .unwrap_or_default();
                let dims = tex.size_vec2();
                let info = format!("{name}   ·   {} × {}", dims.x as u32, dims.y as u32);
                let info_pos = egui::pos2(rect.center().x, rect.bottom() - 14.0);
                painter.text(
                    info_pos,
                    egui::Align2::CENTER_CENTER,
                    info,
                    egui::FontId::proportional(12.0),
                    MUTED,
                );
            }
            None => {
                let center = rect.center();
                painter.text(
                    egui::pos2(center.x, center.y - 10.0),
                    egui::Align2::CENTER_CENTER,
                    "Drop your image here",
                    egui::FontId::proportional(16.0),
                    if is_dragging { ACCENT_HOVER } else { TEXT },
                );
                painter.text(
                    egui::pos2(center.x, center.y + 14.0),
                    egui::Align2::CENTER_CENTER,
                    "or click to browse",
                    egui::FontId::proportional(13.0),
                    MUTED,
                );
            }
        }
    }

    fn browse_row(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        let btn = egui::Button::new(
            egui::RichText::new("Browse Image")
                .size(15.0)
                .strong()
                .color(egui::Color32::WHITE),
        )
        .fill(ACCENT)
        .stroke(egui::Stroke::NONE);
        if ui.add_sized([ui.available_width(), 44.0], btn).clicked() {
            self.handle_file_pick(ctx);
        }
    }

    fn sizes_section(&mut self, ui: &mut egui::Ui) {
        ui.add_space(6.0);
        let count = self.selected_sizes.len();
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("Sizes").size(14.0).strong().color(TEXT));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(
                    egui::RichText::new(format!("{count} selected"))
                        .size(12.0)
                        .color(MUTED),
                );
                let reset = egui::Button::new(
                    egui::RichText::new("Reset").size(12.0).color(ACCENT_HOVER),
                )
                .frame(false);
                if ui.add(reset).clicked() {
                    self.selected_sizes = AVAILABLE_SIZES.iter().copied().collect();
                }
            });
        });
        ui.add_space(6.0);

        let row_w = ui.available_width();
        let gap = 8.0;
        let chip_w = (row_w - gap * (AVAILABLE_SIZES.len() as f32 - 1.0)) / AVAILABLE_SIZES.len() as f32;
        ui.horizontal(|ui| {
            for &size in AVAILABLE_SIZES.iter() {
                let selected = self.selected_sizes.contains(&size);
                let label = egui::SelectableLabel::new(
                    selected,
                    egui::RichText::new(format!("{size}"))
                        .size(13.0)
                        .strong()
                        .color(if selected { egui::Color32::WHITE } else { TEXT }),
                );
                let resp = ui.add_sized([chip_w, 36.0], label);
                if resp.clicked() {
                    if selected {
                        self.selected_sizes.remove(&size);
                    } else {
                        self.selected_sizes.insert(size);
                    }
                }
            }
        });
    }

    fn convert_button(&mut self, ui: &mut egui::Ui) {
        ui.add_space(8.0);
        let can_convert = self.source_path.is_some() && !self.selected_sizes.is_empty();
        let btn = egui::Button::new(
            egui::RichText::new("Convert & Save ICO")
                .size(16.0)
                .strong()
                .color(egui::Color32::WHITE),
        )
        .fill(ACCENT)
        .stroke(egui::Stroke::NONE)
        .min_size(egui::vec2(ui.available_width(), 48.0));
        let resp = ui.add_enabled(can_convert, btn);
        if resp.clicked() {
            self.convert_and_save();
        }
        if !can_convert {
            ui.add_space(4.0);
            ui.label(
                egui::RichText::new("Select an image and at least one size to continue.")
                    .size(11.5)
                    .color(MUTED),
            );
        }
    }

    fn status_line(&mut self, ui: &mut egui::Ui) {
        if self.status_message.is_empty() {
            return;
        }
        ui.add_space(4.0);
        let color = if self.status_is_error { ERROR } else { SUCCESS };
        let bg = egui::Color32::from_rgba_unmultiplied(96, 96, 96, 18);
        egui::Frame::none()
            .fill(bg)
            .stroke(egui::Stroke::new(1.0_f32, egui::Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), 60)))
            .rounding(egui::Rounding::same(8.0))
            .inner_margin(egui::Margin::same(10.0))
            .show(ui, |ui| {
                ui.label(
                    egui::RichText::new(&self.status_message).color(color).size(13.0),
                );
            });
    }
}

impl eframe::App for IcoConverterApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let mut dropped: Option<PathBuf> = None;
        ctx.input(|i| {
            if let Some(d) = i.raw.dropped_files.first() {
                if let Some(path) = &d.path {
                    dropped = Some(path.clone());
                }
            }
        });
        if let Some(path) = dropped {
            self.load_preview(ctx, &path);
            self.source_path = Some(path);
        }

        let is_dragging = ctx.input(|i| !i.raw.hovered_files.is_empty());

        egui::CentralPanel::default()
            .frame(
                egui::Frame::none()
                    .fill(BG)
                    .inner_margin(egui::Margin::same(16.0)),
            )
            .show(ctx, |ui| {
                ui.spacing_mut().item_spacing = egui::vec2(10.0, 12.0);
                self.header(ui);
                self.drop_zone(ui, ctx, is_dragging);
                self.browse_row(ui, ctx);
                self.sizes_section(ui);
                self.convert_button(ui);
                self.status_line(ui);
            });

        if self.show_about {
            egui::Window::new("About")
                .collapsible(false)
                .resizable(false)
                .open(&mut self.show_about)
                .show(ctx, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.add_space(4.0);
                        ui.label(
                            egui::RichText::new("ICO Converter")
                                .size(17.0)
                                .strong()
                                .color(TEXT),
                        );
                        ui.add_space(6.0);
                        ui.label(
                            egui::RichText::new("Developed: Sajedur Rahman Roni")
                                .size(13.0)
                                .color(TEXT),
                        );
                        ui.add_space(10.0);
                        ui.hyperlink_to(
                            egui::RichText::new("GitHub: github.com/sajedur0")
                                .size(13.0)
                                .color(ACCENT),
                            "https://github.com/sajedur0",
                        );
                        ui.add_space(4.0);
                    });
                });
        }
    }
}
