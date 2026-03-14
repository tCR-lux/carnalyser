// Copyright (c) 2026 Cédric Renzi
// SPDX-License-Identifier: GPL-3.0-only
// Commercial license available — contact cedric.renzi@laposte.net

use egui::*;
use crate::app::EcuAnalyzerApp;

pub fn draw_statistics(app: &mut EcuAnalyzerApp, ctx: &Context) {
    SidePanel::left("stats_sidebar").resizable(true).default_width(260.0)
        .show(ctx, |ui| {
            ui.heading("Acquisition Info");
            if let Some(ref s) = app.session {
                Grid::new("meta_grid").num_columns(2).spacing([8.0, 4.0]).show(ui, |ui| {
                    ui.label("Vehicle:");   ui.label(&s.meta.vehicle);          ui.end_row();
                    ui.label("ECU:");       ui.label(&s.meta.ecu);              ui.end_row();
                    ui.label("Operator:");  ui.label(&s.meta.operator);         ui.end_row();
                    ui.label("Channels:");  ui.label(s.channels.len().to_string()); ui.end_row();
                    ui.label("Duration:");  ui.label(format!("{:.1} s", s.duration_s())); ui.end_row();
                    ui.label("Samples:");   ui.label(s.meta.n_samples_total.to_string()); ui.end_row();
                    ui.label("File:");      ui.label(&s.meta.filename);         ui.end_row();
                });
            } else {
                ui.label("No file loaded.");
            }
        });

    CentralPanel::default().show(ctx, |ui| {
        ui.heading("Channel Statistics");

        ui.horizontal(|ui| {
            ui.label("Filter:");
            ui.text_edit_singleline(&mut app.stats_filter);
        });
        ui.separator();

        let filter = app.stats_filter.to_lowercase();
        let stats: Vec<_> = app.stats.iter()
            .filter(|s| filter.is_empty() || s.name.to_lowercase().contains(&filter))
            .collect();

        ScrollArea::vertical().show(ui, |ui| {
            Grid::new("stats_grid")
                .num_columns(7)
                .striped(true)
                .spacing([8.0, 3.0])
                .show(ui, |ui| {
                    // Header
                    for h in &["Parameter", "Unit", "Samples", "Duration(s)", "Hz", "Min", "Max", "Mean"] {
                        ui.label(RichText::new(*h).strong());
                    }
                    ui.end_row();

                    for st in &stats {
                        ui.label(&st.name);
                        ui.label(&st.unit);
                        ui.label(st.n_samples.to_string());
                        ui.label(format!("{:.1}", st.duration_s));
                        ui.label(format!("{:.3}", st.sampling_hz));
                        ui.label(format!("{:.2}", st.min));
                        ui.label(format!("{:.2}", st.max));
                        ui.label(format!("{:.2}", st.mean));
                        ui.end_row();
                    }
                });
        });
    });
}
