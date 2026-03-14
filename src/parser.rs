// Copyright (c) 2026 Cédric Renzi
// SPDX-License-Identifier: GPL-3.0-only
// Commercial license available — contact cedric.renzi@laposte.net

//! MaxiECU XML parser.
//! Handles both "full XML" (81-param) and "LiveGraph" (reduced param) formats.

use std::collections::HashMap;
use chrono::NaiveDateTime;

/// One time-series channel: (timestamp_ms, value) pairs.
#[derive(Debug, Clone)]
pub struct Channel {
    pub name:    String,
    pub unit:    String,
    /// Sorted (timestamp_ms, value) pairs — raw, not interpolated.
    pub samples: Vec<(f64, f64)>,
}

impl Channel {
    pub fn n_samples(&self) -> usize { self.samples.len() }
    pub fn duration_s(&self) -> f64 {
        if self.samples.len() < 2 { return 0.0; }
        (self.samples.last().unwrap().0 - self.samples.first().unwrap().0) / 1000.0
    }
    pub fn sampling_median_ms(&self) -> f64 {
        if self.samples.len() < 2 { return 0.0; }
        let mut diffs: Vec<f64> = self.samples.windows(2)
            .map(|w| w[1].0 - w[0].0)
            .filter(|&d| d > 0.0)
            .collect();
        if diffs.is_empty() { return 0.0; }
        diffs.sort_by(|a, b| a.partial_cmp(b).unwrap());
        diffs[diffs.len() / 2]
    }
    pub fn sampling_hz(&self) -> f64 {
        let m = self.sampling_median_ms();
        if m <= 0.0 { 0.0 } else { 1000.0 / m }
    }
    pub fn min_val(&self) -> f64 { self.samples.iter().map(|s| s.1).fold(f64::INFINITY,    f64::min) }
    pub fn max_val(&self) -> f64 { self.samples.iter().map(|s| s.1).fold(f64::NEG_INFINITY, f64::max) }
    pub fn mean_val(&self) -> f64 {
        if self.samples.is_empty() { return 0.0; }
        self.samples.iter().map(|s| s.1).sum::<f64>() / self.samples.len() as f64
    }

    /// Return (time_s_from_start, value) pairs for plotting.
    pub fn plot_points(&self) -> Vec<[f64; 2]> {
        let t0 = self.samples.first().map(|s| s.0).unwrap_or(0.0);
        self.samples.iter().map(|s| [(s.0 - t0) / 1000.0, s.1]).collect()
    }

    /// Linear interpolation onto a regular grid with given step_ms.
    pub fn interpolated(&self, step_ms: f64) -> Vec<(f64, f64)> {
        if self.samples.len() < 2 { return self.samples.clone(); }
        let t_start = self.samples.first().unwrap().0;
        let t_end   = self.samples.last().unwrap().0;
        let mut out = Vec::new();
        let mut t = t_start;
        let mut i = 0usize;
        while t <= t_end {
            while i + 1 < self.samples.len() && self.samples[i + 1].0 < t { i += 1; }
            let val = if i + 1 < self.samples.len() {
                let (t0, v0) = self.samples[i];
                let (t1, v1) = self.samples[i + 1];
                let alpha = (t - t0) / (t1 - t0).max(1e-9);
                v0 + alpha * (v1 - v0)
            } else {
                self.samples[i].1
            };
            out.push((t, val));
            t += step_ms;
        }
        out
    }
}

/// Metadata extracted from the XML header.
#[derive(Debug, Clone, Default)]
pub struct AcquisitionMeta {
    pub vehicle:      String,
    pub ecu:          String,
    pub operator:     String,
    pub date:         Option<NaiveDateTime>,
    pub filename:     String,
    pub n_samples_total: usize,
}

/// Full parsed session.
#[derive(Debug, Clone)]
pub struct Session {
    pub meta:     AcquisitionMeta,
    pub channels: HashMap<String, Channel>,
}

impl Session {
    pub fn channel_names_sorted(&self) -> Vec<String> {
        let mut names: Vec<_> = self.channels.keys().cloned().collect();
        names.sort();
        names
    }
    pub fn duration_s(&self) -> f64 {
        self.channels.values().map(|c| c.duration_s()).fold(0.0_f64, f64::max)
    }
}

/// Parse a MaxiECU XML file. Supports both full (81-param) and LiveGraph formats.
pub fn parse_maxiecu_xml(path: &str) -> Result<Session, String> {
    let xml = std::fs::read_to_string(path)
        .map_err(|e| format!("Cannot read file: {e}"))?;

    let doc = roxmltree::Document::parse(&xml)
        .map_err(|e| format!("XML parse error: {e}"))?;

    // Locate worksheet rows
    let rows: Vec<_> = doc.descendants()
        .filter(|n| n.tag_name().name() == "Row")
        .collect();

    if rows.len() < 8 {
        return Err("File too short — not a valid MaxiECU XML".into());
    }

    // Helper: extract text from all Data children of a Row node
    let row_data = |row: &roxmltree::Node| -> Vec<String> {
        row.descendants()
            .filter(|n| n.tag_name().name() == "Data")
            .map(|n| n.text().unwrap_or("").trim().to_string())
            .collect()
    };

    // ── Rows 0-4: metadata ─────────────────────────────────────────
    let mut meta = AcquisitionMeta { filename: path.to_string(), ..Default::default() };
    // Row 0 usually contains "Vehicle  <name>" etc; we do a best-effort scan
    for ri in 0..5.min(rows.len()) {
        let cells = row_data(&rows[ri]);
        for (i, cell) in cells.iter().enumerate() {
            let lc = cell.to_lowercase();
            if lc.contains("vehicle") || lc.contains("volvo") {
                if i + 1 < cells.len() { meta.vehicle = cells[i + 1].clone(); }
                else { meta.vehicle = cell.clone(); }
            }
            if lc.contains("ecu") || lc.contains("module") || lc.contains("bosch") || lc.contains("aisin") {
                if i + 1 < cells.len() { meta.ecu = cells[i + 1].clone(); }
                else { meta.ecu = cell.clone(); }
            }
            if lc.contains("operator") {
                if i + 1 < cells.len() { meta.operator = cells[i + 1].clone(); }
            }
        }
    }
    // Best-effort: extract vehicle/ecu from filename
    let fname = std::path::Path::new(path)
        .file_name().unwrap_or_default()
        .to_str().unwrap_or("");
    if meta.vehicle.is_empty() { meta.vehicle = fname.to_string(); }
    if meta.ecu.is_empty() {
        // e.g. "…Gearbox-TCM-Transmission-Control-Module…"
        for keyword in &["Gearbox-TCM","Engine-Diesel","EDC17","Aisin"] {
            if fname.contains(keyword) { meta.ecu = keyword.to_string(); break; }
        }
    }

    // ── Row 5: parameter names, Row 6: units ───────────────────────
    let names = row_data(&rows[5]);
    let units = row_data(&rows[6]);

    // Build map: value_col_index → (name, unit)
    // Structure: alternating (timestamp_col, value_col) pairs
    // names and units have one entry per parameter (not doubled)
    let mut param_map: HashMap<usize, (String, String)> = HashMap::new();
    for (param_idx, name) in names.iter().enumerate() {
        if name.is_empty() { continue; }
        let val_col = param_idx * 2 + 1;
        let unit = units.get(val_col).cloned().unwrap_or_default();
        if unit == "msec" { continue; } // skip timestamp columns wrongly named
        param_map.insert(val_col, (name.clone(), unit));
    }

    // Also support "units row has alternating msec/unit" format (LiveGraph)
    // Re-scan with that assumption if param_map is small
    if param_map.len() < 2 {
        param_map.clear();
        let mut param_idx = 0usize;
        for (col, unit) in units.iter().enumerate() {
            if unit != "msec" && !unit.is_empty() {
                let name = names.get(param_idx).cloned().unwrap_or_else(|| format!("param_{param_idx}"));
                if !name.is_empty() {
                    param_map.insert(col, (name, unit.clone()));
                }
                param_idx += 1;
            }
        }
    }

    // ── Rows 7+: data ──────────────────────────────────────────────
    let mut channels: HashMap<String, Channel> = HashMap::new();
    let data_rows = &rows[7..];
    let mut n_total = 0usize;

    for row in data_rows {
        let cells = row_data(row);
        for (&val_col, (name, unit)) in &param_map {
            let ts_col = val_col.saturating_sub(1);
            if ts_col < cells.len() && val_col < cells.len() {
                let ts_str  = cells[ts_col].trim();
                let val_str = cells[val_col].trim();
                if ts_str.is_empty() || val_str.is_empty() { continue; }
                let ts:  f64 = match ts_str.parse()  { Ok(v) => v, Err(_) => continue };
                let val: f64 = match val_str.parse() { Ok(v) => v, Err(_) => continue };
                let ch = channels.entry(name.clone()).or_insert_with(|| Channel {
                    name: name.clone(), unit: unit.clone(), samples: Vec::new(),
                });
                ch.samples.push((ts, val));
                n_total += 1;
            }
        }
    }

    // Sort all channels by timestamp
    for ch in channels.values_mut() {
        ch.samples.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
    }

    meta.n_samples_total = n_total;

    Ok(Session { meta, channels })
}
