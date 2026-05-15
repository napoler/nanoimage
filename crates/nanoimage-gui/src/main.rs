//! NanoImage GUI - egui 图像优化工具
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use eframe::egui;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_min_inner_size([800.0, 600.0])
            .with_title("NanoImage - 图像优化器"),
        ..Default::default()
    };

    eframe::run_native(
        "NanoImage",
        options,
        Box::new(|_cc| Ok(Box::new(nanoimage_gui::NanoImageApp::new()))),
    )
}