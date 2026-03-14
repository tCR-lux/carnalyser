// Copyright (c) 2026 Cédric Renzi
// SPDX-License-Identifier: GPL-3.0-only
// Commercial license available — contact cedric.renzi@laposte.net

use egui::*;
use egui_plot::{Plot, Line, Legend, PlotPoints, PlotBounds};
use crate::app::EcuAnalyzerApp;

pub fn draw_channels(app: &mut EcuAnalyzerApp, ctx: &Context) {
    let channel_names = app.session.as_ref()
        .map(|s| s.channel_names_sorted())
        .unwrap_or_default();

    SidePanel::left("ch_sidebar").resizable(true).default_width(240.0)
        .show(ctx, |ui| {
            ui.heading("OBD Channels Analysis");
            ui.separator();
            // Session info
            if let Some(ref s) = app.session {
                ui.label(RichText::new(
                    &s.meta.vehicle
                ).strong());
                ui.label(format!("ECU: {}", s.meta.ecu));
                ui.label(format!("{} channels · {:.1}s",
                    s.channels.len(), s.duration_s()));
            }
            ui.separator();
            ui.label(RichText::new("X Axis").strong());
            ComboBox::from_id_salt("x_axis_combo")
                .selected_text(&app.selected_x)
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut app.selected_x, "time".to_string(), "Time (s)");
                    for name in &channel_names {
                        ui.selectable_value(&mut app.selected_x, name.clone(), name.as_str());
                    }
                });

            ui.separator();
            ui.label(RichText::new("Y Channels (multi-select)").strong());
            ScrollArea::vertical().max_height(400.0).show(ui, |ui| {
                for name in &channel_names {
                    let mut selected = app.selected_y.contains(name);
                    if ui.checkbox(&mut selected, name.as_str()).changed() {
                        if selected {
                            app.selected_y.push(name.clone());
                        } else {
                            app.selected_y.retain(|n| n != name);
                        }
                    }
                }
            });

            ui.separator();
            ui.label("Interpolation grid (ms):");
            ui.add(Slider::new(&mut app.interp_grid_ms, 50.0..=5000.0).logarithmic(true));

            ui.horizontal(|ui| {
                if ui.button("Clear selection").clicked() {
                    app.selected_y.clear();
                }
            });

            ui.separator();
            ui.label("Zoom controls");
            ui.horizontal(|ui| {
                if ui.button("Zoom fit").clicked() {
                    app.ch_zoom_fit_requested = true;
                }
                if ui.button("Zoom X").clicked() {
                    app.ch_zoom_x_fit_requested = true;
                }
                if ui.button("Zoom Y").clicked() {
                    app.ch_zoom_y_fit_requested = true;
                }
            });
        });

    CentralPanel::default().show(ctx, |ui| {
        if app.session.is_none() {
            ui.centered_and_justified(|ui| { ui.label("Load an XML file to start."); });
            return;
        }

        let mut min_x = f64::INFINITY;
        let mut max_x = f64::NEG_INFINITY;
        let mut min_y = f64::INFINITY;
        let mut max_y = f64::NEG_INFINITY;

        let mut plot = Plot::new("channels_plot")
            .legend(Legend::default())
            .allow_zoom(true)
            .allow_drag(true)
            .height(ui.available_height());
        
        plot.show(ui, |plot_ui| {
            for ch_name in &app.selected_y.clone() {
                if app.selected_x == "time" {
                    // “time” X-case
                    if let Some(pts_vec) = app.channel_plot_points(ch_name) {
                        // pts: PlotPoints = Vec<[f64; 2]>
                        for [x, y] in &pts_vec {
                            min_x = min_x.min(*x);
                            max_x = max_x.max(*x);
                            min_y = min_y.min(*y);
                            max_y = max_y.max(*y);
                        }
                        let pts = PlotPoints::from(pts_vec);
                        plot_ui.line(Line::new(pts).name(ch_name));
                    }
                } else {
                    // X = another channel — build scatter from interpolated grids
                    if let Some(ref session) = app.session {
                        let g = app.interp_grid_ms;
                        let x_ch = session.channels.get(&app.selected_x);
                        let y_ch = session.channels.get(ch_name);
                        if let (Some(xc), Some(yc)) = (x_ch, y_ch) {
                            let xi = xc.interpolated(g);
                            let yi = yc.interpolated(g);
                            let pts: Vec<[f64; 2]> = xi.iter().zip(yi.iter())
                                .map(|(&(_, xv), &(_, yv))| {
                                    min_x = min_x.min(xv);
                                    max_x = max_x.max(xv);
                                    min_y = min_y.min(yv);
                                    max_y = max_y.max(yv);
                                    [xv, yv]}
                                )
                                .collect();
                            plot_ui.line(Line::new(PlotPoints::from(pts)).name(ch_name));
                        }
                    }
                }
            }
            // At end of plot_ui closure:
            if min_x < max_x && min_y < max_y {
                // Get the current bounds so we can preserve one axis when fitting the other
                let current_bounds = plot_ui.plot_bounds();
                let (cur_min, cur_max) = (current_bounds.min(), current_bounds.max());
                
                if app.ch_zoom_fit_requested {
                    // Fit both axes: use plot_ui.set_plot_bounds
                    plot_ui.set_plot_bounds(
                        PlotBounds::from_min_max([min_x, min_y], [max_x, max_y])
                    );
                    app.ch_zoom_fit_requested = false;
                } else {
                    if app.ch_zoom_x_fit_requested {
                        // Fit X, keep current Y
                        plot_ui.set_plot_bounds(
                            PlotBounds::from_min_max(
                                [min_x, cur_min[1]],
                                [max_x, cur_max[1]],
                            )
                        );
                        app.ch_zoom_x_fit_requested = false;
                    }
                    if app.ch_zoom_y_fit_requested {
                        // Fit Y, keep current X
                        plot_ui.set_plot_bounds(
                            PlotBounds::from_min_max(
                                [cur_min[0], min_y],
                                [cur_max[0], max_y],
                            )
                        );
                        app.ch_zoom_y_fit_requested = false;
                    }
                }
            }
        });
    });
}

// workaround for borrow on session inside loop
trait ChannelInterp { fn interpolated(&self, g: f64) -> Vec<(f64, f64)>; }
impl ChannelInterp for crate::parser::Channel {
    fn interpolated(&self, g: f64) -> Vec<(f64, f64)> {
        crate::parser::Channel::interpolated(self, g)
    }
}
