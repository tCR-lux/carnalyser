// Copyright (c) 2026 Cédric Renzi
// SPDX-License-Identifier: GPL-3.0-only
// Commercial license available — contact cedric.renzi@laposte.net

//! WAV loading + FFT / DFT analysis.

use rustfft::{FftPlanner, num_complex::Complex};

#[derive(Debug, Clone)]
pub struct AudioFile {
    pub path:        String,
    pub sample_rate: u32,
    pub n_channels:  u16,
    pub n_samples:   usize,
    pub duration_s:  f64,
    /// Mono f32 samples (channel 0 only)
    pub samples:     Vec<f32>,
}

pub fn load_wav(path: &str) -> Result<AudioFile, String> {
    let reader = hound::WavReader::open(path)
        .map_err(|e| format!("WAV open error: {e}"))?;
    let spec = reader.spec();
    let n_channels  = spec.channels;
    let sample_rate = spec.sample_rate;

    let samples_raw: Vec<f32> = match spec.sample_format {
        hound::SampleFormat::Float => {
            reader.into_samples::<f32>()
                  .step_by(n_channels as usize)
                  .filter_map(|s| s.ok())
                  .collect()
        }
        hound::SampleFormat::Int => {
            let bits = spec.bits_per_sample as f32;
            let scale = 2f32.powf(bits - 1.0);
            reader.into_samples::<i32>()
                  .step_by(n_channels as usize)
                  .filter_map(|s| s.ok())
                  .map(|s| s as f32 / scale)
                  .collect()
        }
    };

    let n_samples  = samples_raw.len();
    let duration_s = n_samples as f64 / sample_rate as f64;

    Ok(AudioFile {
        path: path.to_string(),
        sample_rate,
        n_channels,
        n_samples,
        duration_s,
        samples: samples_raw,
    })
}

/// Hann window applied to a slice.
fn hann_window(samples: &[f32]) -> Vec<f32> {
    let n = samples.len();
    samples.iter().enumerate()
        .map(|(i, &s)| {
            let w = 0.5 * (1.0 - (2.0 * std::f32::consts::PI * i as f32 / (n as f32 - 1.0)).cos());
            s * w
        })
        .collect()
}

/// Compute FFT spectrum.
/// Returns Vec<[frequency_hz, magnitude_db]>.
pub fn compute_fft(audio: &AudioFile, window_start_s: f64, window_len_s: f64)
    -> Vec<[f64; 2]>
{
    let sr = audio.sample_rate as f64;
    let start = (window_start_s * sr) as usize;
    let len   = (window_len_s   * sr) as usize;
    let end   = (start + len).min(audio.samples.len());
    if start >= end { return vec![]; }

    // Next power of 2 for efficiency
    let n_fft = (end - start).next_power_of_two();
    let mut windowed = hann_window(&audio.samples[start..end]);
    windowed.resize(n_fft, 0.0);

    let mut planner = FftPlanner::<f32>::new();
    let fft = planner.plan_fft_forward(n_fft);
    let mut buffer: Vec<Complex<f32>> = windowed.iter().map(|&s| Complex { re: s, im: 0.0 }).collect();
    fft.process(&mut buffer);

    let half = n_fft / 2;
    let freq_step = sr / n_fft as f64;
    let epsilon   = 1e-10f32;

    (0..half).map(|i| {
        let freq = i as f64 * freq_step;
        let mag  = buffer[i].norm();
        let db   = 20.0 * (mag + epsilon).log10() as f64;
        [freq, db]
    }).collect()
}

/// Return waveform as [time_s, amplitude] pairs (downsampled for display).
pub fn waveform_display(audio: &AudioFile, max_points: usize) -> Vec<[f64; 2]> {
    let total = audio.samples.len();
    let step  = (total / max_points).max(1);
    audio.samples.iter().enumerate().step_by(step)
        .map(|(i, &s)| [i as f64 / audio.sample_rate as f64, s as f64])
        .collect()
}
