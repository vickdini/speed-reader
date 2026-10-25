#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use eframe::egui;
use speed_reader::app::SpeedReaderApp;

fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_maximized(true)
            .with_min_inner_size([640.0, 420.0])
            .with_title("Speed Reader — Distraction-Free Book Reader")
            .with_drag_and_drop(true),
        ..Default::default()
    };

    eframe::run_native(
        "Speed Reader",
        native_options,
        Box::new(|cc| Ok(Box::new(SpeedReaderApp::new(cc)))),
    )
}
