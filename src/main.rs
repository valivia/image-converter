#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use eframe::egui;

mod components;
mod process;
mod structs;
mod types;
mod ui;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        centered: true,
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([600.0, 400.0])
            .with_resizable(false)
            .with_maximize_button(false),
        ..Default::default()
    };

    eframe::run_native(
        "Image converter",
        options,
        Box::new(|_cc| Ok(Box::<ui::App>::default())),
    )
}
