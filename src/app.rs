// Copyright (c) 2026 Cédric Renzi
// SPDX-License-Identifier: GPL-3.0-only
// Commercial license available — contact cedric.renzi@laposte.net

//! Application state.

use crate::{
    parser::{Session, parse_maxiecu_xml},
    analysis::{ChannelStats, EcuType, compute_stats, detect_ecu, all_solenoid_diffs,
                rail_pressure_diff, turbo_diff},
    audio::{AudioFile, load_wav, compute_fft, waveform_display},
};
use std::collections::HashMap;
use egui_plot::PlotPoints;

#[derive(Default, PartialEq, Clone, Copy)]
pub enum ActiveTab {
    #[default] Statistics,
    Channels,
    Derived,
    Acoustic,
}

pub struct EcuAnalyzerApp {
    // ── Session ───────────────────────────────────────────────────
    pub session:        Option<Session>,
    pub stats:          Vec<ChannelStats>,
    pub ecu_type:       EcuType,
    pub error_msg:      Option<String>,

    // ── UI state ──────────────────────────────────────────────────
    pub active_tab:     ActiveTab,
    pub last_folder:    String,

    // Channels tab
    pub selected_x:     String,          // "time" or channel name
    pub selected_y:     Vec<String>,     // multi-select
    pub interp_grid_ms: f64,
    pub stats_filter:   String,
    pub ch_zoom_fit_requested:   bool,
    pub ch_zoom_x_fit_requested: bool,
    pub ch_zoom_y_fit_requested: bool,

    // Derived tab
    pub solenoid_diffs: HashMap<String, Vec<[f64; 2]>>,
    pub rail_diff:      Option<Vec<[f64; 2]>>,
    pub turbo_diff_pts: Option<Vec<[f64; 2]>>,
    pub diff_threshold: f64,

    // Acoustic tab
    pub audio_file:       Option<AudioFile>,
    pub fft_points:       Vec<[f64; 2]>,
    pub waveform_points:  Vec<[f64; 2]>,
    pub fft_window_start: f64,
    pub fft_window_len:   f64,
    pub audio_error:      Option<String>,
}

impl Default for EcuAnalyzerApp {
    fn default() -> Self {
        Self {
            //
            session:               None,
            stats:                 Vec::new(),
            ecu_type:              EcuType::Unknown,
            error_msg:             None,
            //
            active_tab:            ActiveTab::Statistics,
            last_folder:           "/".to_string(),
            //
            selected_x:            "time".to_string(),
            selected_y:            Vec::new(),
            interp_grid_ms:        500.0,
            stats_filter:          String::new(),
            ch_zoom_fit_requested: false,
            ch_zoom_x_fit_requested: false,
            ch_zoom_y_fit_requested: false,
            //
            solenoid_diffs:        HashMap::new(),
            rail_diff:             None,
            turbo_diff_pts:        None,
            diff_threshold:        200.0,
            //
            audio_file:            None,
            fft_points:            Vec::new(),
            waveform_points:       Vec::new(),
            fft_window_start:      0.0,
            fft_window_len:        1.0,
            audio_error:           None,
        }
    }
}

impl EcuAnalyzerApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self::default()
    }

    pub fn load_xml(&mut self, path: &str) {
        match parse_maxiecu_xml(path) {
            Ok(session) => {
                self.stats      = compute_stats(&session);
                self.ecu_type   = detect_ecu(&session);
                self.solenoid_diffs = all_solenoid_diffs(&session, self.interp_grid_ms);
                self.rail_diff      = rail_pressure_diff(&session, self.interp_grid_ms);
                self.turbo_diff_pts = turbo_diff(&session, self.interp_grid_ms);
                self.selected_y.clear();
                self.session    = Some(session);
                self.error_msg  = None;
            }
            Err(e) => { self.error_msg = Some(e); }
        }
    }

    pub fn load_audio(&mut self, path: &str) {
        match load_wav(path) {
            Ok(af) => {
                self.fft_window_len  = af.duration_s.min(1.0);
                self.waveform_points = waveform_display(&af, 4096);
                self.recompute_fft_with(&af);
                self.audio_file  = Some(af);
                self.audio_error = None;
            }
            Err(e) => { self.audio_error = Some(e); }
        }
    }

    pub fn recompute_fft(&mut self) {
        if let Some(ref af) = self.audio_file.clone() {
            self.recompute_fft_with(af);
        }
    }

    fn recompute_fft_with(&mut self, af: &AudioFile) {
        self.fft_points = compute_fft(af, self.fft_window_start, self.fft_window_len);
    }

    /// Build PlotPoints for one channel (time on X).
    pub fn channel_plot_points(&self, name: &str) -> Option<Vec<[f64; 2]>> {
        let session = self.session.as_ref()?;
        let ch = session.channels.get(name)?;
        Some(ch.plot_points())
    }
}
