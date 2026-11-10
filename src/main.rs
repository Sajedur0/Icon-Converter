#![windows_subsystem = "windows"]

mod app;
mod converter;
mod icons;
mod theme;
mod utils;
mod widgets;

/// Decodes the bundled `icon.ico` into a native window icon.
fn window_icon() -> Option<egui::IconData> {
    let bytes: &[u8] = include_bytes!("../icon.ico");
    let decoder = image::codecs::ico::IcoDecoder::new(std::io::Cursor::new(bytes)).ok()?;
    let rgba = image::DynamicImage::from_decoder(decoder).ok()?.to_rgba8();
    let (width, height) = (rgba.width(), rgba.height());
    Some(egui::IconData {
        rgba: rgba.into_raw(),
        width,
        height,
    })
}

fn main() -> eframe::Result<()> {
    let mut viewport = egui::ViewportBuilder::default()
        .with_app_id("com.sajedur0.ico-converter")
        .with_inner_size([520.0, 610.0])
        .with_min_inner_size([440.0, 540.0])
        .with_resizable(true);

    if let Some(icon) = window_icon() {
        viewport = viewport.with_icon(icon);
    }

    let options = eframe::NativeOptions {
        viewport,
        follow_system_theme: false,
        default_theme: eframe::Theme::Light,
        ..Default::default()
    };

    eframe::run_native(
        "ICO Converter",
        options,
        Box::new(|cc| Ok(Box::new(app::IcoConverterApp::new(cc)))),
    )
}
