use color_eyre::Result;
use crossterm::event::{self, Event};
use ratatui::{
    layout::{Constraint, Layout},
    style::{Color, Style},
    symbols,
    text::Line,
    widgets::{Axis, Block, Borders, Chart, Dataset, GraphType, Paragraph},
    DefaultTerminal, Frame,
};
use rtrb::{PushError, RingBuffer};
use rustfft::{num_complex::Complex, FftPlanner};
use saavy_dsp::{
    graph::{envelope::EnvNode, extensions::NodeExt, filter::FilterNode, oscillator::OscNode},
    synth::{factory::VoiceFactory, message::SynthMessage, poly::PolySynth},
};
use std::{thread, time::Duration};

fn main() -> Result<()> {
    color_eyre::install()?;
    let terminal = ratatui::init();

    let (tx, rx) = RingBuffer::<SynthMessage>::new(64);
    let factory = || {
        let osc = OscNode::sine();
        let env = EnvNode::adsr(0.05, 0.1, 0.6, 0.1);
        let _lowpass = FilterNode::lowpass(200.0);
        let _highpass = FilterNode::highpass(200.0);

        osc.amplify(env)
    };

    let mut synth = PolySynth::new(48_000.0, 4, factory, rx);
    let mut buffer: Vec<f32> = vec![0.0; 2048];

    let result = run(terminal, &mut synth, &mut buffer, tx, 48_000.0);
    ratatui::restore();
    result
}

fn run<F: VoiceFactory>(
    mut terminal: DefaultTerminal,
    synth: &mut PolySynth<F>,
    buffer: &mut [f32],
    mut tx: rtrb::Producer<SynthMessage>,
    sample_rate: f32,
) -> Result<()> {
    if buffer.is_empty() {
        return Ok(());
    }

    let block_samples = buffer.len();
    let block_duration = Duration::from_secs_f32(block_samples as f32 / sample_rate);

    let notes = [60, 64, 67, 72];
    let note_duration_samples = (sample_rate * 0.5) as usize;
    let mut samples_into_note = note_duration_samples; // trigger first note immediately
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

        terminal.draw(|frame| {
            render_ui(frame, buffer);
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

fn render_ui(frame: &mut Frame, buffer: &[f32]) {
    // Split screen: left=waveform, right=spectrum+info
    let main_chunks = Layout::default()
        .direction(ratatui::layout::Direction::Horizontal)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(frame.area());

    // Right side: spectrum (top) + info (bottom)
    let right_chunks = Layout::default()
        .direction(ratatui::layout::Direction::Vertical)
        .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
        .split(main_chunks[1]);

    let data: Vec<(f64, f64)> = buffer
        .iter()
        .enumerate()
        .map(|(i, &sample)| (i as f64, sample as f64))
        .collect();

    let peak = buffer.iter().fold(0.0f32, |acc, &x| acc.max(x.abs()));

    let (mut sum, mut sum2) = (0.0f64, 0.0);
    for &s in buffer {
        let x = s as f64;
        sum += x;
        sum2 += x * x;
    }
    let dc = sum / buffer.len() as f64;
    let rms = (sum2 / buffer.len() as f64).sqrt();

    let dataset = Dataset::default()
        .name("Waveform")
        .marker(ratatui::symbols::Marker::Braille)
        .graph_type(ratatui::widgets::GraphType::Line)
        .style(Style::default().fg(Color::Cyan))
        .data(&data);

    let chart = Chart::new(vec![dataset])
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

    let info_text = vec![
        Line::from(format!("Peak Amplitude: {:.3}", peak)),
        Line::from(format!("RMS: {:.3}", rms)),
        Line::from(format!("DC: {:.3}", dc)),
        Line::from(format!("Buffer Size: {} samples", buffer.len())),
        Line::from(format!("Sample Rate: 48k Hz")),
    ];

    let info =
        Paragraph::new(info_text).block(Block::default().title("Info").borders(Borders::ALL));

    // Compute spectrum
    let spectrum_data = compute_spectrum(buffer, 48_000.0);

    // Render spectrum
    let spectrum_widget = render_spectrum(&spectrum_data);

    // Render all widgets
    frame.render_widget(chart, main_chunks[0]);
    frame.render_widget(spectrum_widget, right_chunks[0]);
    frame.render_widget(info, right_chunks[1]);
}

fn compute_spectrum(buffer: &[f32], sample_rate: f32) -> Vec<(f64, f64)> {
    let n = buffer.len();
    if n == 0 {
        return Vec::new();
    }

    // Apply Hann window to reduce spectral leakage
    let mut windowed: Vec<Complex<f32>> = buffer
        .iter()
        .enumerate()
        .map(|(i, &sample)| {
            let window = if n > 1 {
                let denom = (n - 1) as f32;
                0.5 * (1.0 - (2.0 * std::f32::consts::PI * i as f32 / denom).cos())
            } else {
                1.0
            };
            Complex::new(sample * window, 0.0)
        })
        .collect();

    // Compute FFT
    let mut planner = FftPlanner::new();
    let fft = planner.plan_fft_forward(n);
    fft.process(&mut windowed);

    // Logarithmic frequency bins (like musical octaves)
    let min_freq = 20.0; // 20 Hz (lowest audible)
    let max_freq = (sample_rate / 2.0).min(20_000.0); // Nyquist or 20kHz
    let num_bins = 48; // More bins for better resolution

    let mut spectrum = Vec::new();

    for i in 0..num_bins {
        // Logarithmic frequency spacing
        let t = i as f64 / (num_bins - 1) as f64;
        let freq = min_freq as f64 * (max_freq as f64 / min_freq as f64).powf(t);

        // Map frequency to FFT bin
        let bin_index = (freq * n as f64 / sample_rate as f64).round() as usize;

        if bin_index >= windowed.len() / 2 {
            break;
        }

        // Get magnitude at this bin
        let c = &windowed[bin_index];
        let magnitude = (c.re * c.re + c.im * c.im).sqrt();

        // Convert to decibels (with floor to avoid log(0))
        let magnitude_db = if magnitude > 1e-10 {
            20.0 * (magnitude as f64).log10()
        } else {
            -100.0 // Floor at -100 dB
        };

        spectrum.push((freq, magnitude_db));
    }

    spectrum
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
