// Copyright (c) 2026 Cédric Renzi
// SPDX-License-Identifier: GPL-3.0-only
// Commercial license available — contact cedric.renzi@laposte.net

//! Derived signal analysis: solenoid diffs, engine health, sampling report.

use crate::parser::{Session, Channel};
use std::collections::HashMap;

/// Interpolated diff between two channels on a common time grid.
pub fn solenoid_diff(session: &Session, req_name: &str, act_name: &str, grid_ms: f64)
    -> Option<Vec<[f64; 2]>>
{
    let req = session.channels.get(req_name)?;
    let act = session.channels.get(act_name)?;
    let req_i = req.interpolated(grid_ms);
    let act_i = act.interpolated(grid_ms);
    let t0 = req_i.first()?.0.min(act_i.first()?.0);
    // Build lookup for act
    let act_map: HashMap<u64, f64> = act_i.iter()
        .map(|&(t, v)| (t as u64, v))
        .collect();
    let mut out = Vec::new();
    for &(t, req_v) in &req_i {
        if let Some(&act_v) = act_map.get(&(t as u64)) {
            out.push([(t - t0) / 1000.0, (req_v - act_v).abs()]);
        }
    }
    Some(out)
}

/// Compute all available SLCx / SLU diffs from a TCM session.
pub fn all_solenoid_diffs(session: &Session, grid_ms: f64)
    -> HashMap<String, Vec<[f64; 2]>>
{
    let pairs = [
        ("Linear pressure solenoid SLC1", "SLC1_diff"),
        ("Linear pressure solenoid SLC2", "SLC2_diff"),
        ("Linear pressure solenoid SLC3", "SLC3_diff"),
        ("Lock up solenoid SLU",          "SLU_diff"),
    ];
    let mut out = HashMap::new();
    for (base, label) in pairs {
        let req_k = format!("{base}");   // heuristic: first occurrence = req, second = act
        // Find channels whose name starts with base
        let matching: Vec<_> = session.channels.keys()
            .filter(|k| k.contains(base))
            .cloned()
            .collect();
        if matching.len() >= 2 {
            let mut sorted = matching;
            sorted.sort();
            if let Some(diff) = solenoid_diff(session, &sorted[0], &sorted[1], grid_ms) {
                out.insert(label.to_string(), diff);
            }
        }
        let _ = req_k;
    }
    out
}

/// Per-channel statistics table entry.
#[derive(Debug, Clone)]
pub struct ChannelStats {
    pub name:         String,
    pub unit:         String,
    pub n_samples:    usize,
    pub duration_s:   f64,
    pub sampling_hz:  f64,
    pub min:          f64,
    pub max:          f64,
    pub mean:         f64,
}

pub fn compute_stats(session: &Session) -> Vec<ChannelStats> {
    let mut stats: Vec<ChannelStats> = session.channels.values().map(|ch| {
        ChannelStats {
            name:        ch.name.clone(),
            unit:        ch.unit.clone(),
            n_samples:   ch.n_samples(),
            duration_s:  ch.duration_s(),
            sampling_hz: ch.sampling_hz(),
            min:         ch.min_val(),
            max:         ch.max_val(),
            mean:        ch.mean_val(),
        }
    }).collect();
    stats.sort_by(|a, b| a.name.cmp(&b.name));
    stats
}

/// Detect ECU type from channel names.
#[derive(Debug, Clone, PartialEq)]
pub enum EcuType { TCM, Engine, Unknown }

pub fn detect_ecu(session: &Session) -> EcuType {
    let names_concat: String = session.channels.keys().cloned().collect::<Vec<_>>().join(" ");
    if names_concat.contains("solenoid") || names_concat.contains("SLC") || names_concat.contains("SLU") {
        EcuType::TCM
    } else if names_concat.contains("Fuel pressure") || names_concat.contains("Engine speed")
           || names_concat.contains("Air mass") || names_concat.contains("EGR") {
        EcuType::Engine
    } else {
        EcuType::Unknown
    }
}

/// Engine-specific derived signals.
pub fn rail_pressure_diff(session: &Session, grid_ms: f64) -> Option<Vec<[f64; 2]>> {
    // Try to find actual and reference rail pressure channels
    let actual_key = session.channels.keys()
        .find(|k| k.to_lowercase().contains("fuel pressure") && !k.to_lowercase().contains("ref"))?
        .clone();
    let ref_key = session.channels.keys()
        .find(|k| k.to_lowercase().contains("fuel pressure") && k.to_lowercase().contains("ref"))?
        .clone();
    solenoid_diff(session, &ref_key, &actual_key, grid_ms)
}

pub fn turbo_diff(session: &Session, grid_ms: f64) -> Option<Vec<[f64; 2]>> {
    let actual_key = session.channels.keys()
        .find(|k| k.to_lowercase().contains("charging pressure sensor"))?
        .clone();
    let ref_key = session.channels.keys()
        .find(|k| k.to_lowercase().contains("boost pressure setpoint"))?
        .clone();
    solenoid_diff(session, &ref_key, &actual_key, grid_ms)
}
