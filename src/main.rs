#![windows_subsystem = "windows"]

mod app;
mod converter;
mod theme;
#[allow(dead_code)]
mod utils;
mod widgets;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([500.0, 660.0])
            .with_min_inner_size([420.0, 580.0])
            .with_resizable(true),
        ..Default::default()
    };

    eframe::run_native(
        "ICO Converter",
        options,
        Box::new(|cc| Ok(Box::new(app::IcoConverterApp::new(cc)))),
    )
}
