- We consolidate all the analysis tools we produced during Volvo XC60 BVA6 troubleshooting.
- In the end, the application with a GUI allows the user to:
  - load xml files
  - analyse it, which outputs are:
    - general statistics on the acquisition (vehicle, dates, operator, ECU, parameters/channels available, sampling rates, min/max value for each parameter)
    - plot the data depending on the choice of the user (which parameter vs what, including time) with potential subsampling or interpolation
    - plot calculated data like the diffs we produced for the SLCs
    - plot other relevant data if it is an engine ECU
  - plot the DFT and signal of another acoustic file

The software shall be modular, sublicense is possible for its commercial usage (we sell the license).

# Project architecture
ecu-analyzer/
├── Cargo.toml
├── LICENSE
├── README.md
└── src/
    ├── main.rs         ← entry point, window 1400×900
    ├── app.rs          ← global state of application
    ├── parser.rs       ← parser MaxiECU XML (81-param + LiveGraph)
    ├── analysis.rs     ← diff solénoïdes, rail, turbo, stats
    ├── audio.rs        ← WAV + FFT Hann-windowed
    ├── ui_impl.rs      ← eframe::App trait
    └── ui/
        ├── toolbar.rs          ← bar + dialogs files
        ├── tab_statistics.rs   ← metadata + stats table
        ├── tab_channels.rs     ← plotter multi-channel
        ├── tab_derived.rs      ← diffs calculated + threshold
        └── tab_acoustic.rs     ← waveform + FFT/DFT spectrum

# Technology stack
| Crate              | Rôle                          | Raison                                                             |
| ------------------ | ----------------------------- | ------------------------------------------------------------------ |
| eframe / egui 0.31 | GUI native cross-platform     | Meilleur Rust GUI en 2025, WASM-compatible si besoin boringcactus​ |
| egui_plot          | Charts interactifs (zoom/pan) | Natif egui, pas de dépendance externe docs​                        |
| roxmltree          | Parseur XML rapide            | ~10× plus rapide que quick-xml pour ce cas d'usage                 |
| rustfft + hound    | FFT + lecture WAV             | Crates standards audio Rust lib+1                                  |
| rfd                | Dialog fichiers natif         | Support Linux/macOS/Windows                                        |

# Current implementation
- Tab Statistics — metadata véhicule/ECU, tableau filtrable (n, durée, Hz, min/max/mean) par canal
- Tab Channels — sélection multi-canaux, X=temps ou un canal quelconque, interpolation configurable
- Tab Derived — diffs |Req−Actual| pour SLC1/2/3/SLU (TCM) et rail pressure/turbo (moteur), ligne de seuil rouge configurable
- Tab Acoustic — waveform downsamplée + spectre FFT dBFS avec fenêtre de Hann, fenêtre d'analyse ajustable

# What is next ?
We still need export CSV/PNG, lime-freq spectrogram, and multi-file support to compare acquisitions.
