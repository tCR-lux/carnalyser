// Copyright (c) 2026 Cédric Renzi
// SPDX-License-Identifier: GPL-3.0-only
// Commercial license available — contact cedric.renzi@laposte.net

// For commercial sublicensing, contact the copyright holder.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod parser;
mod analysis;
mod audio;
mod ui;
mod ui_impl;

fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1400.0, 900.0])
            .with_min_inner_size([800.0, 600.0])
            .with_title("Carnalyser - ECU Diagnostic Analyzer — © Cédric Renzi"),
        ..Default::default()
    };
    eframe::run_native(
        "Carnalyser - ECU Diagnostic Analyzer",
        native_options,
        Box::new(|cc| Ok(Box::new(app::EcuAnalyzerApp::new(cc)))),
    )
}
