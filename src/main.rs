#![windows_subsystem = "windows"]

mod app;
mod converter;
mod icons;
mod theme;
mod utils;
mod widgets;

/// Uses the bundled `icon.ico` as the native window icon on every platform
/// (on Windows `build.rs` also embeds it into the executable itself).
fn window_icon() -> Option<egui::IconData> {
    let bytes: &[u8] = include_bytes!("../icon.ico");
    let decoder = image::codecs::ico::IcoDecoder::new(std::io::Cursor::new(bytes)).ok()?;
    let image = image::DynamicImage::from_decoder(decoder).ok()?;
    let rgba = image.to_rgba8();
    let (width, height) = (rgba.width(), rgba.height());
    let raw = rgba.into_raw();
    Some(egui::IconData {
        rgba: raw,
        width,
        height,
    })
}

fn main() -> eframe::Result<()> {
    let mut viewport = egui::ViewportBuilder::default()
        .with_app_id("com.sajedur0.ico-converter")
        .with_inner_size([600.0, 800.0])
        .with_min_inner_size([470.0, 660.0])
        .with_resizable(true);

    if let Some(icon) = window_icon() {
        viewport = viewport.with_icon(icon);
    }

    let options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };

    eframe::run_native(
        "ICO Converter",
        options,
        Box::new(|cc| Ok(Box::new(app::IcoConverterApp::new(cc)))),
    )
}
