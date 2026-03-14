// Copyright (c) 2026 Cédric Renzi
// SPDX-License-Identifier: GPL-3.0-only
// Commercial license available — contact cedric.renzi@laposte.net

use egui::*;
use crate::app::{EcuAnalyzerApp, ActiveTab};

fn shorten_path_middle(path: &str, max_len: usize) -> String {
    if path.len() <= max_len {
        return path.to_string();
    }
    let keep = max_len.saturating_sub(3);
    let front = keep / 2;
    let back = keep - front;
    format!("{}...{}", &path[..front], &path[path.len() - back..])
}

pub fn draw_toolbar(app: &mut EcuAnalyzerApp, ctx: &Context) {
    TopBottomPanel::top("toolbar").show(ctx, |ui| {
        ui.horizontal(|ui| {
            ui.heading("Analyzer");
            ui.separator();
            
            ui.heading("🚘💻 ECU");
            // ui.label(egui::RichText::new("🚘💻 ECU").size(22.0).strong());

            // Load XML
            if ui.button("📂 Load XML").clicked() {
                let mut dialog = rfd::FileDialog::new();
                if let Some(path) = dialog
                    .add_filter("MaxiECU XML", &["xml"])
                    .set_directory(&app.last_folder)
                    .pick_file()
                {
                    if let Some(parent) = path.parent() {
                        if let Some(parent_str) = parent.to_str() {
                            app.last_folder = parent_str.to_string();
                        }
                    }
                    app.load_xml(path.to_str().unwrap_or(""));
                    app.active_tab = ActiveTab::Statistics;
                }
            }
            // Tabs
            ui.selectable_value(&mut app.active_tab, ActiveTab::Statistics,  "📊 Statistics");
            ui.selectable_value(&mut app.active_tab, ActiveTab::Channels,    "📈 Channels");
            ui.selectable_value(&mut app.active_tab, ActiveTab::Derived,     "⚙️ Derived");
                       
            ui.separator();

            ui.heading("🎶 NVH");
            // ui.label(egui::RichText::new("🎶 NVH").size(22.0).strong());
            
            // Load WAV
            if ui.button("🔊 Load WAV").clicked() {
                let mut dialog = rfd::FileDialog::new();
                if let Some(path) = dialog
                    .add_filter("WAV audio", &["wav"])
                    .set_directory(&app.last_folder)
                    .pick_file()
                {
                    if let Some(parent) = path.parent() {
                       if let Some(parent_str) = parent.to_str() {
                            app.last_folder = parent_str.to_string();
                        }
                    }
                    app.load_audio(path.to_str().unwrap_or(""));
                    app.active_tab = ActiveTab::Acoustic;
                }
            }
            ui.selectable_value(&mut app.active_tab, ActiveTab::Acoustic,    "🎵 Acoustic");

            ui.separator();

            ui.heading("📄 Status");
            // ui.label(egui::RichText::new("🗃️ Status").size(22.0).strong());

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                // Error
                if let Some(ref err) = app.error_msg.clone() {
                    ui.colored_label(Color32::RED, format!("⚠ {err}"));
                }
                else {
                    ui.colored_label(Color32::GREEN, format!("✅"));
                }

                ui.separator();

                if let ref dir = shorten_path_middle(&app.last_folder, 60) {
                    ui.label(format!("Last folder: {dir}"));
                }
            });
        });
    });
}
