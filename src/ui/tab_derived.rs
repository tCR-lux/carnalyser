// Copyright (c) 2026 Cédric Renzi
// SPDX-License-Identifier: GPL-3.0-only
// Commercial license available — contact cedric.renzi@laposte.net

use egui::*;
use egui_plot::{Plot, Line, HLine, Legend, PlotPoints};
use crate::app::EcuAnalyzerApp;
use crate::analysis::EcuType;

pub fn draw_derived(app: &mut EcuAnalyzerApp, ctx: &Context) {
    SidePanel::left("derived_sidebar").resizable(true).default_width(220.0)
        .show(ctx, |ui| {
            ui.heading("Derived Signals");
            ui.separator();

            ui.label("Diff threshold (mA):");
            ui.add(Slider::new(&mut app.diff_threshold, 0.0..=1000.0));

            ui.separator();
            ui.label(RichText::new("Available diffs:").strong());
            for key in app.solenoid_diffs.keys() {
                ui.label(format!("• {key}"));
            }
            if let EcuType::Engine = app.ecu_type {
                if app.rail_diff.is_some()      { ui.label("• Rail pressure diff"); }
                if app.turbo_diff_pts.is_some() { ui.label("• Turbo pressure diff"); }
            }

            ui.separator();
            ui.label("Interpolation grid (ms):");
            ui.add(Slider::new(&mut app.interp_grid_ms, 50.0..=5000.0).logarithmic(true));
            if ui.button("Recompute diffs").clicked() {
                if let Some(ref s) = app.session.clone() {
                    app.solenoid_diffs = crate::analysis::all_solenoid_diffs(s, app.interp_grid_ms);
                    app.rail_diff      = crate::analysis::rail_pressure_diff(s, app.interp_grid_ms);
                    app.turbo_diff_pts = crate::analysis::turbo_diff(s, app.interp_grid_ms);
                }
            }
        });

    CentralPanel::default().show(ctx, |ui| {
        if app.session.is_none() {
            ui.centered_and_justified(|ui| { ui.label("Load an XML file to view derived signals."); });
            return;
        }

        ScrollArea::vertical().show(ui, |ui| {
            let height_each = (ui.available_height() / 3.0).max(160.0);

            // ── TCM Solenoid diffs ────────────────────────────────
            if !app.solenoid_diffs.is_empty() {
                ui.heading("Solenoid |Req − Actual| (mA)");
                Plot::new("sol_diff_plot")
                    .legend(Legend::default())
                    .allow_zoom(true).allow_drag(true)
                    .height(height_each)
                    .show(ui, |plot_ui| {
                        for (name, pts) in &app.solenoid_diffs {
                            plot_ui.line(
                                Line::new(PlotPoints::from(pts.clone())).name(name)
                            );
                        }
                        plot_ui.hline(HLine::new(app.diff_threshold)
                            .color(Color32::RED)
                            .name("Threshold"));
                    });
            }

            // ── Engine diffs ──────────────────────────────────────
            if let EcuType::Engine = app.ecu_type {
                if let Some(ref pts) = app.rail_diff.clone() {
                    ui.heading("Rail Pressure |Ref − Actual|");
                    Plot::new("rail_diff_plot")
                        .legend(Legend::default())
                        .allow_zoom(true).allow_drag(true)
                        .height(height_each)
                        .show(ui, |plot_ui| {
                            plot_ui.line(Line::new(PlotPoints::from(pts.clone())).name("Rail diff"));
                            plot_ui.hline(HLine::new(app.diff_threshold)
                                .color(Color32::RED).name("Threshold"));
                        });
                }
                if let Some(ref pts) = app.turbo_diff_pts.clone() {
                    ui.heading("Turbo Pressure |Setpoint − Actual|");
                    Plot::new("turbo_diff_plot")
                        .legend(Legend::default())
                        .allow_zoom(true).allow_drag(true)
                        .height(height_each)
                        .show(ui, |plot_ui| {
                            plot_ui.line(Line::new(PlotPoints::from(pts.clone())).name("Turbo diff"));
                            plot_ui.hline(HLine::new(app.diff_threshold)
                                .color(Color32::RED).name("Threshold"));
                        });
                }
            }
        });
    });
}
