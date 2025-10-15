use color_eyre::Result;
use crossterm::event::{self, Event};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    symbols,
    text::Line,
    widgets::{Axis, Block, Borders, Chart, Dataset, GraphType, Paragraph},
    DefaultTerminal, Frame,
};
use rtrb::{PushError, RingBuffer};
use rustfft::{num_complex::Complex, Fft, FftPlanner};
use saavy_dsp::{
    graph::{envelope::EnvNode, extensions::NodeExt, filter::FilterNode, oscillator::OscNode},
    synth::{
        factory::VoiceFactory,
        message::SynthMessage,
        poly::{PolySynth, VoiceEnvelope},
        voice::VoiceState,
    },
};
use std::{sync::Arc, thread, time::Duration};

const ENVELOPE_HISTORY_LEN: usize = 256;
const ENVELOPE_UPDATE_INTERVAL: usize = 6;
const SPECTRUM_BINS: usize = 48;
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

fn main() -> Result<()> {
    color_eyre::install()?;
    let terminal = ratatui::init();

    let sample_rate = 48_000.0;
    let buffer_len = 2048;
    let max_voices = 4;

    let (tx, rx) = RingBuffer::<SynthMessage>::new(64);

    let factory = || {
        let osc = OscNode::square();
        let env = EnvNode::adsr(0.05, 0.1, 0.6, 0.2);
        let filter = FilterNode::lowpass(2_000.0);
        osc.amplify(env).through(filter)
    };

    let mut synth = PolySynth::new(sample_rate, max_voices, factory, rx);
    let mut buffer = vec![0.0; buffer_len];
    let mut envelope_history = EnvelopeHistory::new(synth.max_voices(), ENVELOPE_HISTORY_LEN);

    let result = run(
        terminal,
        &mut synth,
        &mut buffer,
        &mut envelope_history,
        tx,
        sample_rate,
    );

    ratatui::restore();
    result
}

fn run<F: VoiceFactory>(
    mut terminal: DefaultTerminal,
    synth: &mut PolySynth<F>,
    buffer: &mut [f32],
    envelopes: &mut EnvelopeHistory,
    mut tx: rtrb::Producer<SynthMessage>,
    sample_rate: f32,
) -> Result<()> {
    if buffer.is_empty() {
        return Ok(());
    }

    let block_samples = buffer.len();
    let block_duration = Duration::from_secs_f32(block_samples as f32 / sample_rate);
    let mut spectrum_analyzer = SpectrumAnalyzer::new(
        block_samples,
        sample_rate,
        SPECTRUM_BINS,
        ENVELOPE_UPDATE_INTERVAL,
    );
    let mut envelope_scratch = Vec::with_capacity(synth.max_voices());

    let notes = [60, 64, 67, 72];
    let note_duration_samples = (sample_rate * 0.5) as usize;
    let mut samples_into_note = note_duration_samples;
    let mut note_index = 0usize;
    let mut current_note: Option<u8> = None;

    loop {
        if samples_into_note >= note_duration_samples {
            if let Some(prev_note) = current_note {
                send_message(
                    &mut tx,
                    SynthMessage::NoteOff {
                        note: prev_note,
                        velocity: 0,
                    },
                );
            }

            let note = notes[note_index];
            send_message(
                &mut tx,
                SynthMessage::NoteOn {
                    note,
                    velocity: 100,
                },
            );
            current_note = Some(note);

            note_index = (note_index + 1) % notes.len();
            samples_into_note = 0;
        }

        synth.render_block(buffer);
        samples_into_note = samples_into_note.saturating_add(block_samples);
        spectrum_analyzer.maybe_update(buffer);

        envelope_scratch.clear();
        synth.collect_voice_envelopes(&mut envelope_scratch);
        envelopes.push(&envelope_scratch);

        terminal.draw(|frame| {
            render_ui(
                frame,
                buffer,
                sample_rate,
                spectrum_analyzer.data(),
                envelopes,
            );
        })?;

        if event::poll(Duration::from_millis(1))? {
            if matches!(event::read()?, Event::Key(_)) {
                break Ok(());
            }
        }

        thread::sleep(block_duration);
    }
}

fn send_message(tx: &mut rtrb::Producer<SynthMessage>, message: SynthMessage) {
    let mut pending = message;
    loop {
        match tx.push(pending) {
            Ok(_) => break,
            Err(PushError::Full(returned)) => {
                if tx.is_abandoned() {
                    eprintln!(
                        "Synth message queue abandoned; dropping message {:?}",
                        returned
                    );
                    break;
                }
                pending = returned;
                thread::sleep(Duration::from_micros(200));
            }
        }
    }
}

struct SpectrumAnalyzer {
    window: Vec<f32>,
    freq_bins: Vec<f64>,
    bin_indices: Vec<usize>,
    fft: Arc<dyn Fft<f32>>,
    scratch: Vec<Complex<f32>>,
    spectrum: Vec<(f64, f64)>,
    frame_counter: usize,
    update_interval: usize,
}

impl SpectrumAnalyzer {
    fn new(buffer_len: usize, sample_rate: f32, num_bins: usize, update_interval: usize) -> Self {
        let mut planner = FftPlanner::new();
        let fft = planner.plan_fft_forward(buffer_len);

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

        let mut freq_bins = Vec::with_capacity(num_bins);
        let mut bin_indices = Vec::with_capacity(num_bins);
        let max_freq = (sample_rate / 2.0).min(20_000.0).max(1.0);
        let min_freq = 20.0_f32.min(max_freq);
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
                let magnitude = (bin.re * bin.re + bin.im * bin.im).sqrt().max(1e-6);
                *freq = self.freq_bins[i];
                *magnitude_db = 20.0 * (magnitude as f64).log10();
            }
        }
    }

    fn data(&self) -> &[(f64, f64)] {
        &self.spectrum
    }
}

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

    let waveform_points: Vec<(f64, f64)> = buffer
        .iter()
        .enumerate()
        .map(|(i, &sample)| (i as f64, sample as f64))
        .collect();

    let peak = buffer.iter().fold(0.0f32, |acc, &x| acc.max(x.abs()));
    let rms = (buffer.iter().map(|&x| x * x).sum::<f32>() / buffer.len() as f32).sqrt();
    let dc = buffer.iter().map(|&x| x as f64).sum::<f64>() / buffer.len() as f64;

    let waveform_chart = Chart::new(vec![Dataset::default()
        .name("Waveform")
        .marker(symbols::Marker::Braille)
        .graph_type(GraphType::Line)
        .style(Style::default().fg(Color::Cyan))
        .data(&waveform_points)])
    .block(
        Block::default()
            .title("Oscilloscope - Press any key to quit")
            .borders(Borders::ALL),
    )
    .x_axis(
        Axis::default()
            .title("Sample")
            .style(Style::default().fg(Color::Gray))
            .bounds([0.0, buffer.len() as f64]),
    )
    .y_axis(
        Axis::default()
            .title("Amplitude")
            .style(Style::default().fg(Color::Gray))
            .bounds([-1.0, 1.0])
            .labels(vec!["-1.0", "-0.5", "0.0", "0.5", "1.0"]),
    );

    let info_lines = vec![
        Line::from(format!("Peak: {:.3}", peak)),
        Line::from(format!("RMS:  {:.3}", rms)),
        Line::from(format!("DC:   {:.3}", dc)),
        Line::from(format!("Frames: {}", buffer.len())),
        Line::from(format!("Sample Rate: {:.1} Hz", sample_rate)),
    ];
    let info =
        Paragraph::new(info_lines).block(Block::default().title("Info").borders(Borders::ALL));

    let spectrum_chart = render_spectrum(spectrum);

    let envelope_traces = envelopes.build_traces();
    let envelope_datasets: Vec<_> = envelope_traces
        .iter()
        .map(|trace| {
            let base_color = VOICE_COLORS[trace.voice_index % VOICE_COLORS.len()];
            let style = if trace.is_active {
                Style::default().fg(base_color)
            } else {
                Style::default().fg(base_color).add_modifier(Modifier::DIM)
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
                .style(Style::default().fg(Color::Gray))
                .bounds([0.0, envelopes.capacity() as f64]),
        )
        .y_axis(
            Axis::default()
                .title("Level")
                .style(Style::default().fg(Color::Gray))
                .bounds([0.0, 1.0])
                .labels(vec!["0.0", "0.5", "1.0"]),
        );

    frame.render_widget(waveform_chart, main_chunks[0]);
    frame.render_widget(spectrum_chart, right_chunks[0]);
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

    let max_freq = data
        .iter()
        .map(|(f, _)| *f)
        .fold(0.0f64, |acc, f| acc.max(f))
        .max(1.0);

    let max_db = data
        .iter()
        .map(|(_, db)| *db)
        .fold(-100.0f64, |acc, db| acc.max(db));

    Chart::new(vec![dataset])
        .block(
            Block::default()
                .title("Spectrum Analyzer")
                .borders(Borders::ALL),
        )
        .x_axis(
            Axis::default()
                .title("Frequency (Hz)")
                .style(Style::default().fg(Color::Gray))
                .bounds([0.0, max_freq]),
        )
        .y_axis(
            Axis::default()
                .title("Magnitude (dB)")
                .style(Style::default().fg(Color::Gray))
                .bounds([-100.0, max_db.max(0.0) + 10.0])
                .labels(vec!["-100", "-80", "-60", "-40", "-20", "0"]),
        )
}
