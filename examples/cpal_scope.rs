use color_eyre::eyre::{eyre, Result as EyreResult, WrapErr};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    symbols,
    widgets::{Axis, Block, Borders, Chart, Dataset, GraphType, Paragraph},
    DefaultTerminal, Frame,
};
use rtrb::{PushError, RingBuffer};
use rustfft::{num_complex::Complex, Fft, FftPlanner};
use saavy_dsp::{
    graph::{
        delay::{DelayNode, DelayParam},
        extensions::NodeExt,
        filter::FilterNode,
        lfo::LfoNode,
        oscillator::OscNode,
    },
    synth::{
        message::SynthMessage,
        poly::{PolySynth, VoiceEnvelope},
        voice::VoiceState,
    },
    MAX_BLOCK_SIZE,
};
use std::{thread, time::Duration};

// Tunables
const VIS_BLOCK_LEN: usize = 1024; // Analyzer/window size (≈47 FPS @ 48 kHz)
const SPECTRUM_BINS: usize = 48;
const SPECTRUM_UPDATE_INTERVAL: usize = 1; // Update every frame
const AUDIO_RING_BLOCKS: usize = 16; // Capacity in blocks for audio→UI ring
const ENVELOPE_HISTORY_LEN: usize = 256;
const VOICE_COLORS: [Color; 8] = [
    Color::LightRed,
    Color::LightGreen,
    Color::LightYellow,
    Color::LightBlue,
    Color::LightMagenta,
    Color::LightCyan,
    Color::White,
    Color::Gray,
];

fn main() -> EyreResult<()> {
    color_eyre::install()?;
    let terminal = ratatui::init();

    let res = run(terminal);

    ratatui::restore();
    res
}

fn run(mut terminal: DefaultTerminal) -> EyreResult<()> {
    // --- Set up CPAL ---

    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .ok_or_else(|| eyre!("no default output device available"))?;
    let config = device
        .default_output_config()
        .wrap_err("failed to fetch default output config")?;
    // if config.sample_format() != cpal::SampleFormat::F32 {
    //     return Err("cpal_scope requires f32 sample format");
    // }
    let sample_rate = config.sample_rate().0 as f32;
    let channels = config.channels() as usize;

    // --- Cross-thread rings ---
    let (msg_tx, msg_rx) = RingBuffer::<SynthMessage>::new(64);
    let (audio_tx, mut audio_rx) = RingBuffer::<f32>::new(VIS_BLOCK_LEN * AUDIO_RING_BLOCKS);
    let (env_tx, mut env_rx) = RingBuffer::<Vec<VoiceEnvelope>>::new(32);

    // --- Voice factory (sound design) ---
    let factory = || {
        let osc = OscNode::triangle();
        let env = saavy_dsp::graph::envelope::EnvNode::adsr(0.05, 0.1, 0.6, 0.2);
        let osc_saw = OscNode::sawtooth();
        let lfo_dly = LfoNode::sawtooth(0.5);
        let delay = DelayNode::new(30.0, 0.2, 0.4).modulate(lfo_dly, DelayParam::DelayTime, 10.0);
        let lowpass = FilterNode::lowpass(1000.0);

        osc.mix(osc_saw, 0.5)
            .amplify(env)
            .through(lowpass)
            .through(delay)
    };

    // Buffer reused by audio callback
    let mut render_buf = vec![0.0f32; MAX_BLOCK_SIZE];

    // Move into callback
    let stream = device
        .build_output_stream(
            &config.into(),
            {
                let mut synth = PolySynth::new(sample_rate, 4, factory, msg_rx);
                let mut audio_tx = audio_tx;
                let mut env_tx = env_tx;
                let mut env_scratch: Vec<VoiceEnvelope> = Vec::with_capacity(8);
                move |data: &mut [f32], _| {
                    let total_frames = data.len() / channels;
                    let mut frames_written = 0;
                    while frames_written < total_frames {
                        let frames_remaining = total_frames - frames_written;
                        let frames_to_render = frames_remaining.min(MAX_BLOCK_SIZE);

                        let block = &mut render_buf[..frames_to_render];
                        synth.render_block(block);

                        // Duplicate mono to all channels and write to device
                        let out_off = frames_written * channels;
                        for (i, &s) in block.iter().enumerate() {
                            for ch in 0..channels {
                                data[out_off + i * channels + ch] = s;
                            }
                        }

                        // Push mono block to UI ring, non-blocking (drop on overflow)
                        for &s in block.iter() {
                            if let Err(PushError::Full(_)) = audio_tx.push(s) {
                                break; // drop remainder if full
                            }
                        }

                        // Collect envelopes snapshot and send to UI (non-blocking)
                        env_scratch.clear();
                        synth.collect_voice_envelopes(&mut env_scratch);
                        if !env_scratch.is_empty() {
                            let _ = env_tx.push(env_scratch.clone());
                        }

                        frames_written += frames_to_render;
                    }
                }
            },
            move |err| eprintln!("Stream error: {err}"),
            None,
        )
        .wrap_err("failed to build output stream")?;

    stream.play().wrap_err("failed to start output stream")?;

    // --- Kick a simple arpeggio driver so there's sound ---
    thread::spawn({
        let mut tx = msg_tx;
        move || loop {
            for &note in &[60u8, 64, 67, 72] {
                let _ = tx.push(SynthMessage::NoteOn {
                    note,
                    velocity: 100,
                });
                thread::sleep(Duration::from_millis(450));
                let _ = tx.push(SynthMessage::NoteOff { note, velocity: 0 });
                thread::sleep(Duration::from_millis(50));
            }
        }
    });

    // --- UI state ---
    let mut vis_buffer = vec![0.0f32; VIS_BLOCK_LEN];
    let mut spectrum = SpectrumAnalyzer::new(
        VIS_BLOCK_LEN,
        sample_rate,
        SPECTRUM_BINS,
        SPECTRUM_UPDATE_INTERVAL,
    );
    let mut env_history = EnvelopeHistory::new(8, ENVELOPE_HISTORY_LEN);
    let mut env_scratch_ui: Vec<VoiceEnvelope> = Vec::with_capacity(8);

    // --- UI loop ---
    loop {
        // Drain up to one analysis block of samples
        let mut filled = 0usize;
        while filled < VIS_BLOCK_LEN {
            match audio_rx.pop() {
                Ok(s) => {
                    vis_buffer[filled] = s;
                    filled += 1;
                }
                Err(_) => break,
            }
        }
        if filled == VIS_BLOCK_LEN {
            spectrum.maybe_update(&vis_buffer);
        }

        // Drain latest envelope snapshot(s)
        env_scratch_ui.clear();
        while let Ok(snapshot) = env_rx.pop() {
            env_scratch_ui = snapshot; // keep most recent
        }
        if !env_scratch_ui.is_empty() {
            env_history.push(&env_scratch_ui);
        }

        // Draw
        terminal.draw(|frame| {
            render_ui(
                frame,
                &vis_buffer,
                sample_rate,
                spectrum.data(),
                &env_history,
            );
        })?;

        // Exit on key press
        if crossterm::event::poll(Duration::from_millis(1))? {
            if matches!(crossterm::event::read()?, crossterm::event::Event::Key(_)) {
                break;
            }
        }
    }

    Ok(())
}

// -------------------- UI --------------------

fn render_ui(
    frame: &mut Frame,
    buffer: &[f32],
    sample_rate: f32,
    spectrum: &[(f64, f64)],
    envelopes: &EnvelopeHistory,
) {
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(frame.area());
    let right_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(55),
            Constraint::Percentage(25),
            Constraint::Percentage(20),
        ])
        .split(main_chunks[1]);

    // Downsample waveform to chart width
    let target_w = main_chunks[0].width.max(1) as usize;
    let step = (buffer.len() + target_w - 1) / target_w; // ceil_div
    let mut pts: Vec<(f64, f64)> = Vec::with_capacity(target_w);
    let mut i = 0usize;
    while i < buffer.len() {
        pts.push((i as f64, buffer[i] as f64));
        i = i.saturating_add(step);
    }

    let wave = Chart::new(vec![Dataset::default()
        .name("Waveform")
        .marker(symbols::Marker::Braille)
        .graph_type(GraphType::Line)
        .style(Style::default().fg(Color::Cyan))
        .data(&pts)])
    .block(
        Block::default()
            .title("Oscilloscope - Press any key to quit")
            .borders(Borders::ALL),
    )
    .x_axis(
        Axis::default()
            .title("Sample")
            .bounds([0.0, buffer.len() as f64]),
    )
    .y_axis(Axis::default().title("Amp").bounds([-1.0, 1.0]));

    let spec_chart = render_spectrum(spectrum);

    // Info (parity with oscilloscope.rs)
    let peak = buffer.iter().fold(0.0f32, |acc, &x| acc.max(x.abs()));
    let rms = (buffer.iter().map(|&x| x * x).sum::<f32>() / buffer.len() as f32).sqrt();
    let dc = buffer.iter().map(|&x| x as f64).sum::<f64>() / buffer.len() as f64;
    let info_lines = vec![
        format!("Peak: {:.3}", peak).into(),
        format!("RMS:  {:.3}", rms).into(),
        format!("DC:   {:.3}", dc).into(),
        format!("Frames: {}", buffer.len()).into(),
        format!("Sample Rate: {:.1} Hz", sample_rate).into(),
    ];
    let info =
        Paragraph::new(info_lines).block(Block::default().title("Info").borders(Borders::ALL));

    // Envelope panel
    let envelope_traces = envelopes.build_traces();
    let envelope_datasets: Vec<_> = envelope_traces
        .iter()
        .map(|trace| {
            let base_color = VOICE_COLORS[trace.voice_index % VOICE_COLORS.len()];
            let style = if trace.is_active {
                Style::default().fg(base_color)
            } else {
                Style::default().fg(base_color)
            };
            Dataset::default()
                .name(format!("V{}", trace.voice_index + 1))
                .marker(symbols::Marker::Braille)
                .graph_type(GraphType::Line)
                .style(style)
                .data(&trace.data)
        })
        .collect();

    let envelope_chart = Chart::new(envelope_datasets)
        .block(Block::default().title("Envelopes").borders(Borders::ALL))
        .x_axis(
            Axis::default()
                .title("Frames")
                .bounds([0.0, envelopes.capacity() as f64]),
        )
        .y_axis(Axis::default().title("Level").bounds([0.0, 1.0]));

    frame.render_widget(wave, main_chunks[0]);
    frame.render_widget(spec_chart, right_chunks[0]);
    frame.render_widget(envelope_chart, right_chunks[1]);
    frame.render_widget(info, right_chunks[2]);
}

fn render_spectrum(data: &[(f64, f64)]) -> Chart {
    let dataset = Dataset::default()
        .name("Spectrum")
        .marker(symbols::Marker::Braille)
        .graph_type(GraphType::Line)
        .style(Style::default().fg(Color::Green))
        .data(data);

    let max_freq = data.iter().map(|(f, _)| *f).fold(0.0, f64::max).max(1.0);
    let max_db = data.iter().map(|(_, db)| *db).fold(-100.0, f64::max);

    Chart::new(vec![dataset])
        .block(
            Block::default()
                .title("Spectrum Analyzer")
                .borders(Borders::ALL),
        )
        .x_axis(Axis::default().title("Hz").bounds([0.0, max_freq]))
        .y_axis(
            Axis::default()
                .title("dB")
                .bounds([-100.0, max_db.max(0.0) + 10.0])
                .labels(vec!["-100", "-60", "-20", "0"]),
        )
}

// -------------------- Spectrum Analyzer --------------------

struct SpectrumAnalyzer {
    window: Vec<f32>,
    freq_bins: Vec<f64>,
    bin_indices: Vec<usize>,
    fft: std::sync::Arc<dyn Fft<f32>>,
    scratch: Vec<Complex<f32>>,
    spectrum: Vec<(f64, f64)>,
    frame_counter: usize,
    update_interval: usize,
}

impl SpectrumAnalyzer {
    fn new(buffer_len: usize, sample_rate: f32, num_bins: usize, update_interval: usize) -> Self {
        let mut planner = FftPlanner::new();
        let fft = planner.plan_fft_forward(buffer_len);

        // Hann window
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

        // Log-spaced bins
        let mut freq_bins = Vec::with_capacity(num_bins);
        let mut bin_indices = Vec::with_capacity(num_bins);
        let max_freq = (sample_rate / 2.0).min(20_000.0).max(1.0);
        let min_freq = 20.0f32.min(max_freq);
        let ratio = if max_freq > min_freq {
            (max_freq / min_freq) as f64
        } else {
            1.0
        };
        let half = buffer_len.saturating_div(2).max(1);
        for i in 0..num_bins {
            let t = if num_bins > 1 {
                i as f64 / (num_bins - 1) as f64
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
            update_interval: update_interval.max(1),
        }
    }

    fn maybe_update(&mut self, buffer: &[f32]) {
        if buffer.len() != self.window.len() {
            return;
        }
        let should_update =
            self.frame_counter % self.update_interval == 0 || self.spectrum.is_empty();
        self.frame_counter = self.frame_counter.wrapping_add(1);
        if !should_update {
            return;
        }

        for (i, sample) in buffer.iter().enumerate() {
            self.scratch[i].re = *sample * self.window[i];
            self.scratch[i].im = 0.0;
        }
        self.fft.process(&mut self.scratch);
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

    fn data(&self) -> &[(f64, f64)] {
        &self.spectrum
    }
}

// -------------------- Envelope History --------------------

struct EnvelopeHistory {
    traces: Vec<Vec<f32>>,
    states: Vec<VoiceState>,
    head: usize,
    capacity: usize,
    filled: usize,
}

impl EnvelopeHistory {
    fn new(voice_count: usize, capacity: usize) -> Self {
        Self {
            traces: vec![vec![0.0; capacity]; voice_count],
            states: vec![VoiceState::Free; voice_count],
            head: 0,
            capacity,
            filled: 0,
        }
    }

    fn push(&mut self, envelopes: &[VoiceEnvelope]) {
        if self.traces.is_empty() {
            return;
        }

        for trace in &mut self.traces {
            trace[self.head] = 0.0;
        }
        for state in &mut self.states {
            *state = VoiceState::Free;
        }

        for env in envelopes {
            if let Some(trace) = self.traces.get_mut(env.voice_index) {
                trace[self.head] = env.level;
                if let Some(state) = self.states.get_mut(env.voice_index) {
                    *state = env.state;
                }
            }
        }

        self.head = (self.head + 1) % self.capacity;
        if self.filled < self.capacity {
            self.filled += 1;
        }
    }

    fn capacity(&self) -> usize {
        self.capacity
    }

    fn build_traces(&self) -> Vec<EnvelopeTrace> {
        let mut traces = Vec::with_capacity(self.traces.len());
        for (voice_index, history) in self.traces.iter().enumerate() {
            let mut data = Vec::with_capacity(self.capacity);
            for i in 0..self.capacity {
                let idx = (self.head + i) % self.capacity;
                data.push((i as f64, history[idx] as f64));
            }
            let is_active = self
                .states
                .get(voice_index)
                .map(|state| *state != VoiceState::Free)
                .unwrap_or(false)
                || history.iter().any(|&level| level > 0.001);
            traces.push(EnvelopeTrace {
                voice_index,
                data,
                is_active,
            });
        }
        traces
    }
}

struct EnvelopeTrace {
    voice_index: usize,
    data: Vec<(f64, f64)>,
    is_active: bool,
}
