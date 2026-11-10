//! Small formatting / validation helpers.

use image::ImageFormat;
use std::path::{Path, PathBuf};
use std::time::Duration;

pub const SUPPORTED_EXTENSIONS: &[&str] = &[
    "png", "apng", "jpg", "jpeg", "jfif", "bmp", "gif", "webp", "tiff", "tif", "ico", "tga", "qoi",
];

pub fn is_supported_extension(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .is_some_and(|ext| {
            SUPPORTED_EXTENSIONS
                .iter()
                .any(|supported| ext.eq_ignore_ascii_case(supported))
        })
}

pub fn format_label(path: &Path) -> String {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| {
            let ext = e.to_ascii_uppercase();
            match ext.as_str() {
                "JPG" | "JFIF" => "JPEG".to_owned(),
                "TIF" => "TIFF".to_owned(),
                "APNG" => "PNG".to_owned(),
                _ => ext,
            }
        })
        .unwrap_or_else(|| "IMAGE".to_owned())
}

pub fn detected_format_label(format: Option<ImageFormat>, path: Option<&Path>) -> String {
    if let Some(fmt) = format {
        let name = match fmt {
            ImageFormat::Png => Some("PNG"),
            ImageFormat::Jpeg => Some("JPEG"),
            ImageFormat::Gif => Some("GIF"),
            ImageFormat::WebP => Some("WEBP"),
            ImageFormat::Bmp => Some("BMP"),
            ImageFormat::Tiff => Some("TIFF"),
            ImageFormat::Ico => Some("ICO"),
            ImageFormat::Tga => Some("TGA"),
            ImageFormat::Qoi => Some("QOI"),
            ImageFormat::Pnm => Some("PNM"),
            ImageFormat::Avif => Some("AVIF"),
            ImageFormat::Hdr => Some("HDR"),
            ImageFormat::OpenExr => Some("EXR"),
            ImageFormat::Farbfeld => Some("FF"),
            ImageFormat::Dds => Some("DDS"),
            _ => None,
        };
        if let Some(label) = name {
            return label.to_owned();
        }
    }
    path.map(format_label).unwrap_or_else(|| "IMAGE".to_owned())
}

pub fn human_size(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{bytes} B")
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    }
}

pub fn format_duration(duration: Duration) -> String {
    let micros = duration.as_micros();
    if micros < 1_000 {
        format!("{:.2} ms", micros as f64 / 1_000.0)
    } else if micros < 1_000_000 {
        format!("{:.1} ms", micros as f64 / 1_000.0)
    } else {
        format!("{:.2} s", duration.as_secs_f64())
    }
}

/// Ensures the destination path has an `.ico` extension if none was typed.
pub fn ensure_ico_extension(mut path: PathBuf) -> PathBuf {
    let has_ico = path
        .extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| e.eq_ignore_ascii_case("ico"));
    if !has_ico && path.extension().is_none() {
        path.set_extension("ico");
    }
    path
}

/// Keeps long file names / paths readable in a narrow window.
pub fn truncate_middle(text: &str, max_chars: usize) -> String {
    let chars: Vec<char> = text.chars().collect();
    if chars.len() <= max_chars {
        return text.to_owned();
    }
    let keep = max_chars.saturating_sub(1);
    let head = keep / 2;
    let tail = keep - head;
    let mut out: String = chars[..head].iter().collect();
    out.push('…');
    out.extend(chars[chars.len() - tail..].iter());
    out
}

/// Shortens a path for status messages, always keeping the file name.
pub fn short_path(path: &Path) -> String {
    let full = path.to_string_lossy().into_owned();
    truncate_middle(&full, 54)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extension_check_is_case_insensitive_and_allocation_free() {
        assert!(is_supported_extension(Path::new("logo.PNG")));
        assert!(is_supported_extension(Path::new("photo.JpEg")));
        assert!(is_supported_extension(Path::new("app.ico")));
        assert!(!is_supported_extension(Path::new("notes.txt")));
    }

    #[test]
    fn detected_format_prefers_magic_bytes_over_extension() {
        assert_eq!(
            detected_format_label(Some(ImageFormat::WebP), Some(Path::new("misnamed.png"))),
            "WEBP"
        );
        assert_eq!(
            detected_format_label(None, Some(Path::new("photo.jpg"))),
            "JPEG"
        );
    }

    #[test]
    fn ensure_ico_extension_adds_when_missing() {
        assert_eq!(
            ensure_ico_extension(PathBuf::from("/tmp/app_icon")),
            PathBuf::from("/tmp/app_icon.ico")
        );
        assert_eq!(
            ensure_ico_extension(PathBuf::from("/tmp/app_icon.ICO")),
            PathBuf::from("/tmp/app_icon.ICO")
        );
    }
}
