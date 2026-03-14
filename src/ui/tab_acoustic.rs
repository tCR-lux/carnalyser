// Copyright (c) 2026 Cédric Renzi
// SPDX-License-Identifier: GPL-3.0-only
// Commercial license available — contact cedric.renzi@laposte.net

use egui::*;
use egui_plot::{Plot, Line, Legend, PlotPoints};
use crate::app::EcuAnalyzerApp;

pub fn draw_acoustic(app: &mut EcuAnalyzerApp, ctx: &Context) {
    SidePanel::left("acoustic_sidebar").resizable(true).default_width(240.0)
        .show(ctx, |ui| {
            ui.heading("Acoustic Analysis");
            ui.separator();
            if let Some(ref af) = app.audio_file {
                ui.label(RichText::new(
                    std::path::Path::new(&af.path)
                        .file_name().unwrap_or_default()
                        .to_str().unwrap_or("")
                ).strong());
                ui.separator();
                Grid::new("audio_meta").num_columns(2).spacing([8.0, 4.0]).show(ui, |ui| {
                    ui.label("Sample rate:"); ui.label(format!("{} Hz", af.sample_rate)); ui.end_row();
                    ui.label("Channels:");    ui.label(af.n_channels.to_string());        ui.end_row();
                    ui.label("Duration:");    ui.label(format!("{:.2} s", af.duration_s)); ui.end_row();
                    ui.label("Samples:");     ui.label(af.n_samples.to_string());         ui.end_row();
                });
                ui.separator();

                ui.label(RichText::new("FFT Window").strong());
                ui.label("Start (s):");
                let changed_start = ui.add(Slider::new(&mut app.fft_window_start,
                    0.0..=af.duration_s)).changed();
                ui.label("Length (s):");
                let changed_len = ui.add(Slider::new(&mut app.fft_window_len,
                    0.05..=af.duration_s.min(10.0)).logarithmic(true)).changed();
                if changed_start || changed_len {
                    app.recompute_fft();
                }

                ui.separator();
                // Frequency markers (engine diagnostics)
                ui.label(RichText::new("Expected frequencies").strong());
                ui.small("At idle ~750 RPM:");
                ui.small("• 5-cyl injection: ~31 Hz");
                ui.small("• Harmonics: 62, 93, 125 Hz");
                ui.small("• Injector solenoid: 5–10 kHz");

            } else {
                ui.label("Load a WAV file to analyse.");
                if let Some(ref err) = app.audio_error {
                    ui.colored_label(Color32::RED, err);
                }
            }
        });

    CentralPanel::default().show(ctx, |ui| {
        if app.audio_file.is_none() {
            ui.centered_and_justified(|ui| {
                ui.label("No audio file loaded — use 🔊 Load WAV in the toolbar.");
            });
            return;
        }

        let half = ui.available_height() / 2.0 - 24.0;

        // ── Waveform ──────────────────────────────────────────────
        ui.label(RichText::new("Waveform").strong());
        Plot::new("waveform_plot")
            .legend(Legend::default())
            .allow_zoom(true).allow_drag(true)
            .height(half)
            .show(ui, |plot_ui| {
                plot_ui.line(
                    Line::new(PlotPoints::from(app.waveform_points.clone()))
                        .name("Amplitude")
                        .color(Color32::from_rgb(100, 180, 255))
                );
            });

        ui.separator();

        // ── FFT Spectrum ──────────────────────────────────────────
        ui.label(RichText::new("FFT Spectrum (dBFS)").strong());
        Plot::new("fft_plot")
            .legend(Legend::default())
            .allow_zoom(true).allow_drag(true)
            .height(half)
            .show(ui, |plot_ui| {
                plot_ui.line(
                    Line::new(PlotPoints::from(app.fft_points.clone()))
                        .name("Magnitude (dBFS)")
                        .color(Color32::from_rgb(255, 160, 60))
                );
            });
    });
}
