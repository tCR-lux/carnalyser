// Copyright (c) 2026 Cédric Renzi
// SPDX-License-Identifier: GPL-3.0-only
// Commercial license available — contact cedric.renzi@laposte.net

//! eframe::App implementation — dispatches to sub-UI modules.

use eframe::egui;
use crate::app::{EcuAnalyzerApp, ActiveTab};
use crate::ui::{toolbar, tab_statistics, tab_channels, tab_derived, tab_acoustic};

impl eframe::App for EcuAnalyzerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        toolbar::draw_toolbar(self, ctx);
        match self.active_tab {
            ActiveTab::Statistics => tab_statistics::draw_statistics(self, ctx),
            ActiveTab::Channels   => tab_channels::draw_channels(self, ctx),
            ActiveTab::Derived    => tab_derived::draw_derived(self, ctx),
            ActiveTab::Acoustic   => tab_acoustic::draw_acoustic(self, ctx),
        }
    }
}
