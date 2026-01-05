//! Spectrum analyzer widget
//!
//! FFT-based frequency spectrum visualization with log-spaced bins.

use ratatui::{
    layout::Rect,
    style::{Color, Style},
    symbols,
    widgets::{Axis, Block, Borders, Chart, Dataset, GraphType},
    Frame,
};
use rustfft::{num_complex::Complex, Fft, FftPlanner};
use std::sync::Arc;

/// Number of frequency bins to display
const SPECTRUM_BINS: usize = 48;

/// Spectrum analyzer with FFT processing
pub struct SpectrumAnalyzer {
    /// Hann window coefficients
    window: Vec<f32>,
    /// Frequency values for each bin (Hz)
    freq_bins: Vec<f64>,
    /// FFT bin indices corresponding to each frequency
    bin_indices: Vec<usize>,
    /// FFT processor
    fft: Arc<dyn Fft<f32>>,
    /// Scratch buffer for FFT computation
    scratch: Vec<Complex<f32>>,
    /// Current spectrum data: (frequency_hz, magnitude_db)
    spectrum: Vec<(f64, f64)>,
    /// Frame counter for update throttling
    frame_counter: usize,
    /// Update every N frames
    update_interval: usize,
}

impl SpectrumAnalyzer {
    /// Create a new spectrum analyzer
    ///
    /// # Arguments
    /// * `buffer_len` - FFT size (should match audio buffer length)
    /// * `sample_rate` - Audio sample rate in Hz
    pub fn new(buffer_len: usize, sample_rate: f32) -> Self {
        let mut planner = FftPlanner::new();
        let fft = planner.plan_fft_forward(buffer_len);

        // Hann window - reduces spectral leakage
        let window: Vec<f32> = (0..buffer_len)
            .map(|i| {
                if buffer_len > 1 {
                    let denom = (buffer_len - 1) as f32;
                    0.5 * (1.0 - (2.0 * std::f32::consts::PI * i as f32 / denom).cos())
                } else {
                    1.0
                }
            })
            .collect();

        // Log-spaced frequency bins (20 Hz to Nyquist)
        let mut freq_bins = Vec::with_capacity(SPECTRUM_BINS);
        let mut bin_indices = Vec::with_capacity(SPECTRUM_BINS);
        let max_freq = (sample_rate / 2.0).min(20_000.0).max(1.0);
        let min_freq = 20.0f32.min(max_freq);
        let ratio = if max_freq > min_freq {
            (max_freq / min_freq) as f64
        } else {
            1.0
        };
        let half = buffer_len.saturating_div(2).max(1);

        for i in 0..SPECTRUM_BINS {
            let t = if SPECTRUM_BINS > 1 {
                i as f64 / (SPECTRUM_BINS - 1) as f64
            } else {
                0.0
            };
            let freq = if ratio > 1.0 {
                min_freq as f64 * ratio.powf(t)
            } else {
                min_freq as f64 + (max_freq as f64 - min_freq as f64) * t
            };
            let mut index = (freq * buffer_len as f64 / sample_rate as f64).round() as usize;
            if index >= half {
                index = half - 1;
            }
            freq_bins.push(freq);
            bin_indices.push(index);
        }

        let scratch = vec![Complex::new(0.0, 0.0); buffer_len];
        let spectrum = freq_bins.iter().map(|&f| (f, -120.0)).collect();

        Self {
            window,
            freq_bins,
            bin_indices,
            fft,
            scratch,
            spectrum,
            frame_counter: 0,
            update_interval: 1,
        }
    }

    /// Update the spectrum from new audio samples
    ///
    /// Only processes if buffer length matches and update interval elapsed.
    pub fn update(&mut self, buffer: &[f32]) {
        if buffer.len() != self.window.len() {
            return;
        }

        let should_update =
            self.frame_counter % self.update_interval == 0 || self.spectrum.is_empty();
        self.frame_counter = self.frame_counter.wrapping_add(1);

        if !should_update {
            return;
        }

        // Apply window and prepare for FFT
        for (i, sample) in buffer.iter().enumerate() {
            self.scratch[i].re = *sample * self.window[i];
            self.scratch[i].im = 0.0;
        }

        // Compute FFT
        self.fft.process(&mut self.scratch);

        // Extract magnitudes at log-spaced frequencies
        let half = (self.scratch.len() / 2).max(1);
        for (i, &idx) in self.bin_indices.iter().enumerate() {
            if let Some((freq, magnitude_db)) = self.spectrum.get_mut(i) {
                let index = idx.min(half.saturating_sub(1));
                let bin = self.scratch[index];
                let power = (bin.re * bin.re + bin.im * bin.im).max(1e-12);
                *freq = self.freq_bins[i];
                *magnitude_db = 10.0 * (power as f64).log10();
            }
        }
    }

    /// Get the current spectrum data
    pub fn data(&self) -> &[(f64, f64)] {
        &self.spectrum
    }
}

/// Render the spectrum analyzer widget
pub fn render_spectrum(frame: &mut Frame, area: Rect, spectrum: &[(f64, f64)]) {
    let block = Block::default()
        .title(" Spectrum ")
        .borders(Borders::ALL);

    let dataset = Dataset::default()
        .marker(symbols::Marker::Braille)
        .graph_type(GraphType::Line)
        .style(Style::default().fg(Color::Green))
        .data(spectrum);

    let max_freq = spectrum
        .iter()
        .map(|(f, _)| *f)
        .fold(0.0, f64::max)
        .max(1.0);
    let max_db = spectrum
        .iter()
        .map(|(_, db)| *db)
        .fold(-100.0, f64::max);

    let chart = Chart::new(vec![dataset])
        .block(block)
        .x_axis(
            Axis::default()
                .bounds([0.0, max_freq])
                .style(Style::default().fg(Color::DarkGray)),
        )
        .y_axis(
            Axis::default()
                .bounds([-100.0, max_db.max(0.0) + 10.0])
                .labels(vec!["-100", "-60", "-20", "0"])
                .style(Style::default().fg(Color::DarkGray)),
        );

    frame.render_widget(chart, area);
}
