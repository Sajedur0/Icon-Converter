//! Application state and all user actions.

use crate::converter::{self, FitMode};
use crate::theme::{self, Palette};
use crate::utils;
use crate::widgets;
use image::DynamicImage;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub struct SourceImage {
    pub name: String,
    pub path: Option<PathBuf>,
    pub image: DynamicImage,
    pub size_bytes: Option<u64>,
    pub format: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StatusKind {
    Success,
    Error,
}

#[derive(Clone)]
pub struct Status {
    pub kind: StatusKind,
    pub message: String,
    pub detail: Option<String>,
}

pub struct IcoConverterApp {
    pub source: Option<SourceImage>,
    pub preview: Option<egui::TextureHandle>,
    pub checker: egui::TextureHandle,
    pub selected_sizes: HashSet<u32>,
    pub fit_mode: FitMode,
    pub dark_mode: bool,
    pub status: Option<Status>,
    pub show_about: bool,
}

fn checker_texture(ctx: &egui::Context, p: &Palette) -> egui::TextureHandle {
    let mut image = egui::ColorImage::new([16, 16], egui::Color32::TRANSPARENT);
    for y in 0..16 {
        for x in 0..16 {
            image[(x, y)] = if (x / 8 + y / 8) % 2 == 0 {
                p.checker_a
            } else {
                p.checker_b
            };
        }
    }
    ctx.load_texture("checkerboard", image, egui::TextureOptions::NEAREST_REPEAT)
}

impl IcoConverterApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let ctx = cc.egui_ctx.clone();
        theme::install_fonts(&ctx);
        let palette = Palette::new(false);
        theme::apply_visuals(&ctx, &palette);
        let checker = checker_texture(&ctx, &palette);

        Self {
            source: None,
            preview: None,
            checker,
            selected_sizes: converter::AVAILABLE_SIZES.iter().copied().collect(),
            fit_mode: FitMode::Cover,
            dark_mode: false,
            status: None,
            show_about: false,
        }
    }

    pub fn palette(&self) -> Palette {
        Palette::new(self.dark_mode)
    }

    pub(crate) fn set_status(&mut self, kind: StatusKind, message: &str, detail: Option<String>) {
        self.status = Some(Status {
            kind,
            message: message.to_owned(),
            detail,
        });
    }

    fn set_error(&mut self, message: &str, detail: impl Into<String>) {
        self.set_status(StatusKind::Error, message, Some(detail.into()));
    }

    pub(crate) fn set_dark_mode(&mut self, ctx: &egui::Context, dark: bool) {
        self.dark_mode = dark;
        let p = self.palette();
        theme::apply_visuals(ctx, &p);
        self.checker = checker_texture(ctx, &p);
        ctx.request_repaint();
    }

    pub(crate) fn handle_file_pick(&mut self, ctx: &egui::Context) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Images", utils::SUPPORTED_EXTENSIONS)
            .pick_file()
        {
            self.load_from_path(ctx, &path);
        }
    }

    fn install_source(
        &mut self,
        ctx: &egui::Context,
        name: String,
        path: Option<PathBuf>,
        image: DynamicImage,
        size_bytes: Option<u64>,
        format: String,
    ) {
        let rgba = image.to_rgba8();
        let (w, h) = (rgba.width(), rgba.height());
        let color_image = egui::ColorImage::from_rgba_unmultiplied([w as usize, h as usize], &rgba);
        self.preview =
            Some(ctx.load_texture("source_preview", color_image, egui::TextureOptions::LINEAR));
        self.source = Some(SourceImage {
            name,
            path,
            image,
            size_bytes,
            format,
        });
        self.status = None;
        ctx.request_repaint();
    }

    pub(crate) fn load_from_path(&mut self, ctx: &egui::Context, path: &Path) {
        if !utils::is_supported_extension(path) {
            self.set_error(
                "Unsupported file type",
                "Choose a PNG, JPG, BMP, GIF, WEBP or TIFF image.",
            );
            return;
        }

        match converter::load_image(path) {
            Ok(image) => {
                let name = path
                    .file_name()
                    .map(|s| s.to_string_lossy().into_owned())
                    .unwrap_or_else(|| "Image".to_owned());
                let format = utils::format_label(path);
                let size_bytes = std::fs::metadata(path).ok().map(|m| m.len());
                self.install_source(
                    ctx,
                    name,
                    Some(path.to_path_buf()),
                    image,
                    size_bytes,
                    format,
                );
            }
            Err(e) => self.set_error("Couldn't open this image", format!("{e:#}")),
        }
    }

    pub(crate) fn load_from_bytes(&mut self, ctx: &egui::Context, bytes: &[u8], name: &str) {
        match converter::decode_bytes(bytes) {
            Ok(image) => self.install_source(
                ctx,
                name.to_owned(),
                None,
                image,
                Some(bytes.len() as u64),
                "IMAGE".to_owned(),
            ),
            Err(e) => self.set_error("Couldn't open this image", format!("{e:#}")),
        }
    }

    pub(crate) fn clear_source(&mut self) {
        self.source = None;
        self.preview = None;
        self.status = None;
    }

    pub(crate) fn convert_and_save(&mut self) {
        let mut sizes: Vec<u32> = self.selected_sizes.iter().copied().collect();
        sizes.sort_unstable();

        if sizes.is_empty() {
            self.set_error("No sizes selected", "Pick at least one icon resolution.");
            return;
        }

        let fit_mode = self.fit_mode;

        // Keep every borrow of `self` inside this block so the status can be
        // assigned afterwards without cloning the (potentially large) image.
        let outcome: Option<(PathBuf, anyhow::Result<()>)> = {
            let Some(src) = self.source.as_ref() else {
                self.set_error("No image selected", "Add a source image first.");
                return;
            };
            let suggested = converter::suggested_output_name(src.path.as_deref());
            let save_path = rfd::FileDialog::new()
                .add_filter("Windows Icon", &["ico"])
                .set_file_name(suggested.as_str())
                .save_file();
            save_path.map(|path| {
                let result = converter::export_ico(&src.image, &sizes, fit_mode, &path);
                (path, result)
            })
        };

        if let Some((path, result)) = outcome {
            match result {
                Ok(()) => {
                    let bytes = std::fs::metadata(&path).ok().map(|m| m.len());
                    let mut detail = utils::short_path(&path);
                    if let Some(bytes) = bytes {
                        detail.push_str(&format!("  ·  {}", utils::human_size(bytes)));
                    }
                    self.set_status(
                        StatusKind::Success,
                        &format!("Icon saved with {} resolutions", sizes.len()),
                        Some(detail),
                    );
                }
                Err(e) => self.set_error("Conversion failed", format!("{e:#}")),
            }
        }
    }
}

impl eframe::App for IcoConverterApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let drops: Vec<(Option<PathBuf>, Option<Arc<[u8]>>)> = ctx.input(|i| {
            i.raw
                .dropped_files
                .iter()
                .map(|f| (f.path.clone(), f.bytes.clone()))
                .collect()
        });
        for (path, bytes) in drops {
            if let Some(path) = path {
                self.load_from_path(ctx, &path);
            } else if let Some(bytes) = bytes {
                self.load_from_bytes(ctx, &bytes, "Dropped image");
            }
        }

        let is_dragging = ctx.input(|i| !i.raw.hovered_files.is_empty());
        let (escape, convert_shortcut, open_shortcut) = ctx.input(|i| {
            (
                i.key_pressed(egui::Key::Escape),
                i.modifiers.ctrl && i.key_pressed(egui::Key::Enter),
                i.modifiers.ctrl && i.key_pressed(egui::Key::O),
            )
        });
        if escape {
            self.show_about = false;
        }
        if convert_shortcut {
            self.convert_and_save();
        }
        if open_shortcut {
            self.handle_file_pick(ctx);
        }

        let mut p = self.palette();

        let header_frame = egui::Frame::none()
            .fill(p.surface)
            .inner_margin(egui::Margin::symmetric(20.0, 14.0))
            .shadow(egui::Shadow {
                offset: egui::vec2(0.0, 4.0),
                blur: 12.0,
                spread: 0.0,
                color: p.shadow,
            });
        egui::TopBottomPanel::top("app_header")
            .frame(header_frame)
            .show(ctx, |ui| {
                widgets::header::show(self, ui, ctx, &p);
            });

        p = self.palette();

        let footer_frame = egui::Frame::none()
            .fill(p.surface)
            .inner_margin(egui::Margin::symmetric(20.0, 14.0))
            .shadow(egui::Shadow {
                offset: egui::vec2(0.0, -4.0),
                blur: 12.0,
                spread: 0.0,
                color: p.shadow,
            });
        egui::TopBottomPanel::bottom("app_footer")
            .frame(footer_frame)
            .show(ctx, |ui| {
                widgets::status::show(self, ui, &p);
                widgets::footer::show(self, ui, &p);
            });

        let content_frame = egui::Frame::none()
            .fill(p.bg)
            .inner_margin(egui::Margin::symmetric(20.0, 16.0));
        egui::CentralPanel::default()
            .frame(content_frame)
            .show(ctx, |ui| {
                ui.spacing_mut().item_spacing = egui::vec2(10.0, 14.0);
                egui::ScrollArea::vertical().show(ui, |ui| {
                    widgets::section::show(
                        ui,
                        &p,
                        "01",
                        "Source image",
                        if self.source.is_some() {
                            Some("Ready".to_owned())
                        } else {
                            None
                        },
                    );
                    widgets::drop_zone::show(self, ui, ctx, is_dragging, &p);

                    ui.add_space(6.0);
                    widgets::section::show(
                        ui,
                        &p,
                        "02",
                        "Icon sizes",
                        Some(format!("{} of 7 selected", self.selected_sizes.len())),
                    );
                    widgets::sizes::show(self, ui, &p);
                    ui.add_space(4.0);
                });
            });

        if self.show_about {
            widgets::about::show(self, ctx, &p);
        }
    }
}
