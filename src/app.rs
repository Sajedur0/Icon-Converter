use crate::converter;
use crate::theme;
use crate::widgets::{about, convert, drop_zone, header, sizes, status};
use std::collections::HashSet;
use std::path::PathBuf;

pub struct IcoConverterApp {
    pub(crate) source_path: Option<PathBuf>,
    pub(crate) preview: Option<egui::TextureHandle>,
    pub(crate) checker: egui::TextureHandle,
    pub(crate) selected_sizes: HashSet<u32>,
    pub(crate) status_message: String,
    pub(crate) status_is_error: bool,
    pub(crate) show_about: bool,
}

impl IcoConverterApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let ctx = cc.egui_ctx.clone();
        theme::apply(&ctx);

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
            selected_sizes: converter::AVAILABLE_SIZES.iter().copied().collect(),
            status_message: String::new(),
            status_is_error: false,
            show_about: false,
        }
    }

    pub(crate) fn handle_file_pick(&mut self, ctx: &egui::Context) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Images", &["png", "jpg", "jpeg", "bmp", "gif", "webp", "tiff"])
            .pick_file()
        {
            self.load_preview(ctx, &path);
            self.source_path = Some(path);
        }
    }

    pub(crate) fn load_preview(&mut self, ctx: &egui::Context, path: &std::path::Path) {
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

    pub(crate) fn convert_and_save(&mut self) {
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
                    .fill(theme::BG)
                    .inner_margin(egui::Margin::same(16.0)),
            )
            .show(ctx, |ui| {
                ui.spacing_mut().item_spacing = egui::vec2(10.0, 12.0);
                header::show(self, ui);
                drop_zone::show(self, ui, ctx, is_dragging);
                convert::browse_row(self, ui, ctx);
                sizes::show(self, ui);
                convert::convert_button(self, ui);
                status::show(self, ui);
            });

        if self.show_about {
            about::modal(self, ctx);
        }
    }
}
