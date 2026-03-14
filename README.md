# Carnalyser - ECU Diagnostic Analyzer

**Copyright © 2026 Cédric Renzi -- See licensing below**

A cross-platform desktop application for analyzing MaxiECU XML diagnostic data
and acoustic signals from engine/gearbox recordings.

---

## Product Vision

**Carnalyser** is a native desktop diagnostic analysis tool built for in-depth troubleshooting of a Volvo XC60,
based on MaxiECU acquisition results.

### Problem it solves
MaxiECU exports raw XML files (full 81-parameter acquisitions and LiveGraph reduced-parameter sessions).
These files are difficult to analyse manually — there is no tooling to:
- Visualise multiple channels simultaneously
- Cross-correlate channels against each other (X vs Y, not just X vs time)
- Compute derived signals (solenoid diffs, rail pressure delta, turbo delta)
- Analyse acoustic recordings (WAV) alongside ECU data via FFT

### What carnalyser does
- Parses both MaxiECU XML formats (full & LiveGraph)
- Displays per-channel statistics (samples, duration, Hz, min/max/mean)
- Plots multi-channel time series with configurable interpolation grid
- Plots any channel against any other channel (X/Y scatter via interpolation)
- Computes derived signals: solenoid diffs, rail pressure, turbo delta
- Loads WAV audio files and computes Hann-windowed FFT spectrum
- Displays waveform and frequency spectrum side by side with ECU data

### Target user
For a Single user — the vehicle owner/engineer performing diagnostics on a
personal Volvo XC60 — the tool is private, non-commercial, and not
intended for redistribution.
With commercial license and ad-hoc fees defined by the current repository software owner
and on a case-by-case basis, the tool could be integrated to other commercially licensed
software.

## Architecture

### Design principles
- **Single source of truth**: all application states lives in one struct
  (`EcuAnalyzerApp`). No global variables. No shared mutable state.
- **Immediate-mode UI**: `egui` redraws the entire UI every frame (~60fps).
  There are no persistent widget objects — each frame re-declares all
  widgets from scratch.
- **Strict data flow**: data moves in one direction only:
```
File on disk
→ parser / audio loader
→ EcuAnalyzerApp fields
→ ui/* draw_*() functions → rendered to screen
```
- **Separation of concerns**: data structures, parsing, analysis, audio,
and UI are in separate modules with no circular dependencies.

### Directory structure

```
src/
├── main.rs         — entry point
├── app.rs          — global application state
├── parser.rs       — MaxiECU XML parser (full & LiveGraph formats)
├── analysis.rs     — derived signals: solenoid diffs, engine health
├── audio.rs        — WAV loading, Hann-windowed FFT
├── ui_impl.rs      — eframe::App trait implementation
└── ui/
    ├── mod.rs
    ├── toolbar.rs
    ├── tab_statistics.rs
    ├── tab_channels.rs
    ├── tab_derived.rs
    └── tab_acoustic.rs
```

### Main loop
`eframe::run_native()` owns the event loop. It calls
`EcuAnalyzerApp::update()` on every frame. `update()` dispatches
rendering to the active tab's draw function. The developer never
writes an explicit loop.
```
main()
└── eframe::run_native() [never returns until window closed]
└── every frame:
EcuAnalyzerApp::update()
├── toolbar::draw_toolbar()
└── match active_tab:
Statistics → tab_statistics::draw_statistics()
Channels → tab_channels::draw_channels()
Derived → tab_derived::draw_derived()
Acoustic → tab_acoustic::draw_acoustic()
```

### Key data structures

| Struct | File | Role |
|---|---|---|
| `EcuAnalyzerApp` | `app.rs` | Central application state — single source of truth |
| `Session` | `parser.rs` | One parsed XML file: metadata + all channels |
| `Channel` | `parser.rs` | One time-series: name, unit, `Vec<(timestamp_ms, value)>` |
| `AcquisitionMeta` | `parser.rs` | Vehicle, ECU, operator, date, filename |
| `AudioFile` | `audio.rs` | WAV file: sample rate, samples `Vec<f32>`, duration |
| `ChannelStats` | `analysis.rs` | Per-channel statistics: n, hz, min, max, mean |
| `ActiveTab` | `app.rs` | Enum: Statistics / Channels / Derived / Acoustic |

### Extendability

- **New ECU**: add detection in `analysis::detect_ecu()`, add derived signals in `analysis.rs`
- **New derived signal**: implement in `analysis.rs`, expose in `tab_derived.rs`
- **Acoustic features**: extend `audio.rs` (e.g. spectrogram, octave bands, peak detection)

---

## Features

### UI/UX

- **📊 Statistics tab** — acquisition metadata, per-channel stats (n, duration, Hz, min/max/mean)
- **📈 Channels tab** — interactive multi-channel plotter (X=time or any channel), zoom/pan
- **⚙️ Derived tab** — solenoid diffs (|Req−Actual|), engine rail/turbo diffs, configurable threshold
- **🎵 Acoustic tab** — WAV waveform + FFT spectrum (dBFS), configurable analysis window

### Supported ECUs

| ECU | Parameters |
|-----|-----------|
| Aisin TF-80SC / AF40 TCM | Recognition of SLC1/2/3, SLB1, SLU, SLT, RPM and further analyses|
| Bosch EDC17CP22 (D4/D5 diesel) | Recognition of Rail pressure, MAF, EGR, FAP, turbo and further analyses |
| Any MaxiECU XML | Generic channel display |

## Build

```bash
# Prerequisites: Rust toolchain (rustup.rs), cargo
cargo build --release
# Binary: target/release/carnalyser
```


Typical Workflow with Makefile:
- Développement quotidien:
```make run-debug```
- Before a commit:
```make fmt && make lint && make test```
- Before a release:
# Avant une release
```make release```

## Usage

1. Open `Carnalyser - ECU Diagnostic Analyzer`
2. Click **📂 Load XML** → select a MaxiECU XML file
3. Browse **Statistics** for acquisition overview
4. Go to **Channels** → select Y channels + X axis → interactive plot
5. Go to **Derived** → view solenoid/rail/turbo diffs with threshold line
6. Click **🔊 Load WAV** → go to **Acoustic** → adjust FFT window → analyse engine noise

## Dependencies

| Crate | Role |
|---|---|
| `eframe` | Native window host + OpenGL/wgpu rendering |
| `egui` | Immediate-mode UI widgets |
| `egui_plot` | 2D plot widget (Line, PlotPoints) |
| `roxmltree` | Read-only Fast XML DOM parser |
| `hound` | WAV file reader (PCM int + float) |
| `rustfft` | Fast Fourier Transform engine |
| `rfd` | Native OS file picker dialog |
| `chrono` | Date/time parsing for XML metadata |
| `serde` / `serde_json` | Serialization (available for future persistence) |

> **Current note on `rfd`**: pinned to `0.14.x` to avoid pulling in `zbus 5.x`
> / `zvariant 5.x` which require Cargo ≥ 1.85 (edition2024).
> Upgrade `rfd` to `0.15+` only after upgrading the Rust toolchain
> via `rustup update stable`.

---

## Licensing

This software is available under a **dual license**:

- **Open source** (GPL v3): free for personal, academic, and
  open-source projects. See [LICENSE](LICENSE).
- **Commercial**: required for proprietary or commercial use.
  See [LICENSE-COMMERCIAL](LICENSE-COMMERCIAL) and contact
  cedric.renzi@laposte.net to obtain a commercial license.

### Driving rationale

| Concern | Decision |
|---|---|
| Personal diagnostic tool, shareable as open source | GPL v3 ensures any open-source derivative stays open |
| Sole author, full copyright retained | Dual licensing is only possible because there is a single copyright holder |
| Potential future commercial use | Commercial license preserves monetisation rights without breaking GPL |
| Dependency compatibility | All core dependencies are MIT or Apache-2.0 — compatible with both GPL and commercial distribution |

### Dependency license audit

| Crate | License | Compatible with proprietary? |
|---|---|---|
| `eframe` / `egui` | MIT | ✅ Yes |
| `roxmltree` | MIT / Apache-2.0 | ✅ Yes |
| `hound` | Apache-2.0 | ✅ Yes |
| `rustfft` | MIT / Apache-2.0 | ✅ Yes |
| `rfd` | MIT | ✅ Yes |
| `chrono` | MIT / Apache-2.0 | ✅ Yes |

No GPL-licensed dependency is present in the direct dependency tree.
The project is free to remain proprietary.
