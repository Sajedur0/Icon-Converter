//! Application state, background preparation pipeline and user actions.

use crate::converter::{self, FitMode, PreparedModeSet, PreparedSource};
use crate::theme::{self, PALETTE};
use crate::utils;
use crate::widgets;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::Arc;
use std::time::{Duration, Instant};

pub struct SourceImage {
    pub name: String,
    pub path: Option<PathBuf>,
    pub prepared: Arc<PreparedSource>,
    pub mode_cache: [Option<Arc<PreparedModeSet>>; 3],
    pub size_bytes: Option<u64>,
    pub format: String,
    pub prep_duration: Duration,
}

pub struct PreviewTextures {
    pub main: egui::TextureHandle,
    pub ruler: [egui::TextureHandle; converter::RULER_SIZES.len()],
}

struct ReadyPayload {
    name: String,
    path: Option<PathBuf>,
    prepared: Arc<PreparedSource>,
    initial_mode: FitMode,
    initial_set: Arc<PreparedModeSet>,
    size_bytes: Option<u64>,
    format: String,
    prep_duration: Duration,
}

struct LoadError {
    title: String,
    detail: String,
}

struct LoadMessage {
    id: u64,
    outcome: Result<Box<ReadyPayload>, LoadError>,
}

struct DroppedItem {
    path: Option<PathBuf>,
    bytes: Option<Arc<[u8]>>,
    label: String,
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
    pub previews: Option<PreviewTextures>,
    pub checker: egui::TextureHandle,
    pub selected_sizes: [bool; converter::AVAILABLE_SIZES.len()],
    pub fit_mode: FitMode,
    pub status: Option<Status>,
    pub show_about: bool,
    pub loading_label: Option<String>,
    load_id: u64,
    load_tx: Sender<LoadMessage>,
    load_rx: Receiver<LoadMessage>,
}

fn checker_texture(ctx: &egui::Context) -> egui::TextureHandle {
    let mut image = egui::ColorImage::new([16, 16], egui::Color32::TRANSPARENT);
    for y in 0..16 {
        for x in 0..16 {
            image[(x, y)] = if (x / 8 + y / 8) % 2 == 0 {
                PALETTE.checker_a
            } else {
                PALETTE.checker_b
            };
        }
    }
    ctx.load_texture("checkerboard", image, egui::TextureOptions::NEAREST_REPEAT)
}

fn rgba_to_texture(
    ctx: &egui::Context,
    label: &str,
    img: &image::RgbaImage,
) -> egui::TextureHandle {
    let color = egui::ColorImage::from_rgba_unmultiplied(
        [img.width() as usize, img.height() as usize],
        img.as_raw(),
    );
    ctx.load_texture(label, color, egui::TextureOptions::LINEAR)
}

fn build_preview_textures(
    ctx: &egui::Context,
    mode_set: &PreparedModeSet,
) -> Option<PreviewTextures> {
    let main_frame = mode_set.frame(256).or_else(|| mode_set.frames.last())?;
    let main = rgba_to_texture(ctx, "icon_preview", &main_frame.image);

    let ruler = std::array::from_fn(|idx| {
        let sz = converter::RULER_SIZES[idx];
        let img = mode_set
            .frame(sz)
            .map(|f| &f.image)
            .unwrap_or(&main_frame.image);
        rgba_to_texture(ctx, &format!("icon_ruler_{sz}"), img)
    });

    Some(PreviewTextures { main, ruler })
}

impl IcoConverterApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let ctx = cc.egui_ctx.clone();
        theme::install_fonts(&ctx);
        theme::apply_visuals(&ctx);
        let checker = checker_texture(&ctx);
        let (load_tx, load_rx) = mpsc::channel();

        Self {
            source: None,
            previews: None,
            checker,
            selected_sizes: [true; converter::AVAILABLE_SIZES.len()],
            fit_mode: FitMode::Cover,
            status: None,
            show_about: false,
            loading_label: None,
            load_id: 0,
            load_tx,
            load_rx,
        }
    }

    pub fn selected_count(&self) -> usize {
        self.selected_sizes.iter().filter(|&&on| on).count()
    }

    pub fn is_size_selected(&self, size: u32) -> bool {
        converter::AVAILABLE_SIZES
            .iter()
            .position(|&s| s == size)
            .is_some_and(|idx| self.selected_sizes[idx])
    }

    pub fn selected_sizes_vec(&self) -> Vec<u32> {
        converter::AVAILABLE_SIZES
            .iter()
            .zip(self.selected_sizes.iter())
            .filter_map(|(&size, &on)| on.then_some(size))
            .collect()
    }

    pub fn active_mode_set(&self) -> Option<&PreparedModeSet> {
        let src = self.source.as_ref()?;
        src.mode_cache[self.fit_mode.index()].as_deref()
    }

    pub fn estimated_ico_bytes(&self) -> Option<u64> {
        let mode_set = self.active_mode_set()?;
        let sizes = self.selected_sizes_vec();
        if sizes.is_empty() {
            None
        } else {
            Some(mode_set.estimated_ico_bytes(&sizes))
        }
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

    pub(crate) fn set_fit_mode(&mut self, ctx: &egui::Context, mode: FitMode) {
        if self.fit_mode == mode {
            return;
        }
        self.fit_mode = mode;

        if let Some(src) = self.source.as_mut() {
            let idx = mode.index();
            if src.mode_cache[idx].is_none() {
                src.mode_cache[idx] = Some(Arc::new(src.prepared.prepare_mode(mode)));
            }
            if let Some(mode_set) = src.mode_cache[idx].as_deref() {
                self.previews = build_preview_textures(ctx, mode_set);
            }
        }
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

    pub(crate) fn load_from_path(&mut self, ctx: &egui::Context, path: &Path) {
        if !utils::is_supported_extension(path) {
            self.set_error(
                "Unsupported format",
                "Supported: PNG, JPG, WEBP, BMP, GIF, TIFF, ICO.",
            );
            return;
        }

        let display_name = path
            .file_name()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_else(|| "Image".to_owned());

        self.load_id = self.load_id.wrapping_add(1);
        let job_id = self.load_id;
        let mode = self.fit_mode;
        let tx = self.load_tx.clone();
        let egui_ctx = ctx.clone();
        let owned_path = path.to_path_buf();

        self.loading_label = Some(display_name.clone());
        self.status = None;
        ctx.request_repaint();

        std::thread::spawn(move || {
            let started = Instant::now();
            let outcome = match converter::load_source_from_path(&owned_path) {
                Ok(loaded) => {
                    let format =
                        utils::detected_format_label(loaded.format, Some(owned_path.as_path()));
                    let prepared = Arc::new(loaded.prepared);
                    let initial_set = Arc::new(prepared.prepare_mode(mode));
                    Ok(Box::new(ReadyPayload {
                        name: display_name,
                        path: Some(owned_path),
                        prepared,
                        initial_mode: mode,
                        initial_set,
                        size_bytes: loaded.size_bytes,
                        format,
                        prep_duration: started.elapsed(),
                    }))
                }
                Err(e) => Err(LoadError {
                    title: "Import failed".to_owned(),
                    detail: format!("{e:#}"),
                }),
            };
            let _ = tx.send(LoadMessage {
                id: job_id,
                outcome,
            });
            egui_ctx.request_repaint();
        });
    }

    pub(crate) fn load_from_bytes(&mut self, ctx: &egui::Context, bytes: Arc<[u8]>, name: &str) {
        self.load_id = self.load_id.wrapping_add(1);
        let job_id = self.load_id;
        let mode = self.fit_mode;
        let tx = self.load_tx.clone();
        let egui_ctx = ctx.clone();
        let display_name = name.to_owned();

        self.loading_label = Some(display_name.clone());
        self.status = None;
        ctx.request_repaint();

        std::thread::spawn(move || {
            let started = Instant::now();
            let outcome = match converter::load_source_from_bytes(&bytes) {
                Ok(loaded) => {
                    let format = utils::detected_format_label(loaded.format, None);
                    let prepared = Arc::new(loaded.prepared);
                    let initial_set = Arc::new(prepared.prepare_mode(mode));
                    Ok(Box::new(ReadyPayload {
                        name: display_name,
                        path: None,
                        prepared,
                        initial_mode: mode,
                        initial_set,
                        size_bytes: loaded.size_bytes,
                        format,
                        prep_duration: started.elapsed(),
                    }))
                }
                Err(e) => Err(LoadError {
                    title: "Import failed".to_owned(),
                    detail: format!("{e:#}"),
                }),
            };
            let _ = tx.send(LoadMessage {
                id: job_id,
                outcome,
            });
            egui_ctx.request_repaint();
        });
    }

    fn poll_background_loads(&mut self, ctx: &egui::Context) {
        while let Ok(msg) = self.load_rx.try_recv() {
            if msg.id != self.load_id {
                continue;
            }
            self.loading_label = None;
            match msg.outcome {
                Ok(payload) => {
                    let mut mode_cache: [Option<Arc<PreparedModeSet>>; 3] = [None, None, None];
                    mode_cache[payload.initial_mode.index()] = Some(payload.initial_set);

                    let active_idx = self.fit_mode.index();
                    if mode_cache[active_idx].is_none() {
                        mode_cache[active_idx] =
                            Some(Arc::new(payload.prepared.prepare_mode(self.fit_mode)));
                    }

                    if let Some(active_set) = mode_cache[active_idx].as_deref() {
                        self.previews = build_preview_textures(ctx, active_set);
                    }

                    self.source = Some(SourceImage {
                        name: payload.name,
                        path: payload.path,
                        prepared: payload.prepared,
                        mode_cache,
                        size_bytes: payload.size_bytes,
                        format: payload.format,
                        prep_duration: payload.prep_duration,
                    });
                    self.status = None;
                }
                Err(err) => {
                    self.set_error(&err.title, err.detail);
                }
            }
        }
    }

    pub(crate) fn clear_source(&mut self) {
        self.load_id = self.load_id.wrapping_add(1);
        self.loading_label = None;
        self.source = None;
        self.previews = None;
        self.status = None;
    }

    pub(crate) fn convert_and_save(&mut self) {
        if self.loading_label.is_some() {
            self.set_error("Processing in progress", "Please wait a moment.");
            return;
        }

        let sizes = self.selected_sizes_vec();
        if sizes.is_empty() {
            self.set_error("No size selected", "Enable at least one output size.");
            return;
        }

        let fit_mode = self.fit_mode;
        let (suggested, mode_set) = {
            let Some(src) = self.source.as_mut() else {
                self.set_error("No source image", "Import an image first.");
                return;
            };
            let idx = fit_mode.index();
            if src.mode_cache[idx].is_none() {
                src.mode_cache[idx] = Some(Arc::new(src.prepared.prepare_mode(fit_mode)));
            }
            let suggested = converter::suggested_output_name(src.path.as_deref());
            let set = Arc::clone(src.mode_cache[idx].as_ref().expect("mode set initialized"));
            (suggested, set)
        };

        let Some(chosen_path) = rfd::FileDialog::new()
            .add_filter("Windows Icon", &["ico"])
            .set_file_name(suggested.as_str())
            .save_file()
        else {
            return;
        };

        let save_path = utils::ensure_ico_extension(chosen_path);
        let started = Instant::now();
        match mode_set.write_ico(&sizes, &save_path) {
            Ok(written_bytes) => {
                let elapsed = started.elapsed();
                let count = sizes.len();
                let noun = if count == 1 { "size" } else { "sizes" };
                let detail = format!(
                    "{} · {} · {}",
                    utils::short_path(&save_path),
                    utils::human_size(written_bytes),
                    utils::format_duration(elapsed),
                );
                self.set_status(
                    StatusKind::Success,
                    &format!("Exported .ICO ({count} {noun})"),
                    Some(detail),
                );
            }
            Err(e) => self.set_error("Export failed", format!("{e:#}")),
        }
    }
}

impl eframe::App for IcoConverterApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.poll_background_loads(ctx);

        let drops: Vec<DroppedItem> = ctx.input(|i| {
            i.raw
                .dropped_files
                .iter()
                .map(|f| {
                    let label = if f.name.is_empty() {
                        "Dropped image".to_owned()
                    } else {
                        f.name.clone()
                    };
                    DroppedItem {
                        path: f.path.clone(),
                        bytes: f.bytes.clone(),
                        label,
                    }
                })
                .collect()
        });
        for item in drops {
            if let Some(path) = item.path {
                self.load_from_path(ctx, &path);
            } else if let Some(bytes) = item.bytes {
                self.load_from_bytes(ctx, bytes, &item.label);
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

        let header_frame = egui::Frame::none()
            .fill(PALETTE.surface)
            .inner_margin(egui::Margin::symmetric(16.0, 9.0))
            .stroke(egui::Stroke::new(1.0_f32, PALETTE.border));
        egui::TopBottomPanel::top("app_header")
            .frame(header_frame)
            .show_separator_line(false)
            .show(ctx, |ui| {
                widgets::header::show(self, ui);
            });

        let footer_frame = egui::Frame::none()
            .fill(PALETTE.surface)
            .inner_margin(egui::Margin::symmetric(16.0, 9.0))
            .stroke(egui::Stroke::new(1.0_f32, PALETTE.border));
        egui::TopBottomPanel::bottom("app_footer")
            .frame(footer_frame)
            .show_separator_line(false)
            .show(ctx, |ui| {
                widgets::status::show(self, ui);
                widgets::footer::show(self, ui);
            });

        let content_frame = egui::Frame::none()
            .fill(PALETTE.bg)
            .inner_margin(egui::Margin::symmetric(16.0, 10.0));
        egui::CentralPanel::default()
            .frame(content_frame)
            .show(ctx, |ui| {
                ui.spacing_mut().item_spacing = egui::vec2(7.0, 10.0);
                egui::ScrollArea::vertical().show(ui, |ui| {
                    widgets::drop_zone::show(self, ui, ctx, is_dragging);
                    widgets::sizes::show(self, ui, ctx);
                });
            });

        if self.show_about {
            widgets::about::show(self, ctx);
        }
    }
}
