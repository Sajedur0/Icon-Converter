use std::path::Path;

pub fn is_supported_extension(path: &Path) -> bool {
    matches!(
        path.extension()
            .and_then(|e| e.to_str())
            .map(|e| e.to_lowercase())
            .as_deref(),
        Some("png" | "jpg" | "jpeg" | "bmp" | "gif" | "webp" | "tiff" | "tif")
    )
}

pub fn format_error(msg: &str) -> String {
    format!("❌ {msg}")
}

pub fn format_success(msg: &str) -> String {
    format!("✅ {msg}")
}
