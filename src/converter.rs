//! Image loading, square framing and ICO encoding.

use anyhow::{Context, Result};
use image::{imageops::FilterType, DynamicImage, Rgba, RgbaImage};
use std::path::Path;

pub const AVAILABLE_SIZES: [u32; 7] = [16, 24, 32, 48, 64, 128, 256];

/// How a non-square source image is fitted into a square icon.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FitMode {
    /// Centre-crop to a square, then downscale. Best for photos and logos.
    Cover,
    /// Fit the whole image and pad the empty area with transparency.
    Contain,
}

impl FitMode {
    pub fn label(self) -> &'static str {
        match self {
            FitMode::Cover => "Fill & crop",
            FitMode::Contain => "Fit & pad",
        }
    }

    pub fn hint(self) -> &'static str {
        match self {
            FitMode::Cover => "Non-square images are centre-cropped",
            FitMode::Contain => "Empty space stays transparent",
        }
    }
}

pub fn load_image(path: &Path) -> Result<DynamicImage> {
    image::open(path).context("The file is corrupt or uses an unsupported image format")
}

pub fn decode_bytes(bytes: &[u8]) -> Result<DynamicImage> {
    image::load_from_memory(bytes).context("The dropped file is not a readable image")
}

/// Renders one square, high-quality RGBA frame for the given icon size.
pub fn render_frame(img: &DynamicImage, size: u32, mode: FitMode) -> RgbaImage {
    let (w, h) = (img.width(), img.height());
    if w == 0 || h == 0 {
        return RgbaImage::from_pixel(size, size, Rgba([0, 0, 0, 0]));
    }

    match mode {
        FitMode::Cover => {
            let side = w.min(h).max(1);
            let x = (w - side) / 2;
            let y = (h - side) / 2;
            img.crop_imm(x, y, side, side)
                .resize_exact(size, size, FilterType::Lanczos3)
                .to_rgba8()
        }
        FitMode::Contain => {
            let ratio = (size as f32 / w as f32).min(size as f32 / h as f32);
            let nw = ((w as f32 * ratio).round() as u32).clamp(1, size);
            let nh = ((h as f32 * ratio).round() as u32).clamp(1, size);
            let scaled = img.resize(nw, nh, FilterType::Lanczos3).to_rgba8();
            let mut canvas = RgbaImage::from_pixel(size, size, Rgba([0, 0, 0, 0]));
            image::imageops::overlay(
                &mut canvas,
                &scaled,
                ((size - nw) / 2) as i64,
                ((size - nh) / 2) as i64,
            );
            canvas
        }
    }
}

/// Writes a single multi-resolution .ico file containing every selected size.
pub fn export_ico(
    img: &DynamicImage,
    sizes: &[u32],
    mode: FitMode,
    output_path: &Path,
) -> Result<()> {
    let mut icon_dir = ico::IconDir::new(ico::ResourceType::Icon);

    for &size in sizes {
        let frame = render_frame(img, size, mode);
        let (w, h) = (frame.width(), frame.height());
        let icon_image = ico::IconImage::from_rgba_data(w, h, frame.into_raw());
        let entry = ico::IconDirEntry::encode(&icon_image)
            .context("Failed to encode one of the icon frames")?;
        icon_dir.add_entry(entry);
    }

    let file = std::fs::File::create(output_path).context("Cannot create the output file")?;
    icon_dir
        .write(file)
        .context("Failed to write the ICO file")?;
    Ok(())
}

/// A sensible default name for the save dialog, based on the source file.
pub fn suggested_output_name(source: Option<&Path>) -> String {
    source
        .and_then(|p| p.file_stem())
        .map(|stem| format!("{}.ico", stem.to_string_lossy()))
        .unwrap_or_else(|| "icon.ico".to_owned())
}
