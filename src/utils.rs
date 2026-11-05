//! Small formatting / validation helpers.

use std::path::Path;

pub const SUPPORTED_EXTENSIONS: &[&str] =
    &["png", "jpg", "jpeg", "bmp", "gif", "webp", "tiff", "tif"];

pub fn is_supported_extension(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| SUPPORTED_EXTENSIONS.contains(&e.to_lowercase().as_str()))
        .unwrap_or(false)
}

pub fn format_label(path: &Path) -> String {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| {
            let ext = e.to_uppercase();
            if ext == "JPG" {
                "JPEG".to_owned()
            } else if ext == "TIF" {
                "TIFF".to_owned()
            } else {
                ext
            }
        })
        .unwrap_or_else(|| "IMAGE".to_owned())
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
    truncate_middle(&full, 58)
}
