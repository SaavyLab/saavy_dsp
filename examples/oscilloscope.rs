use color_eyre::{Result};
use crossterm::event::{self, Event};
use ratatui::{
    layout::{Constraint, Layout}, style::{Color, Style}, text::Line, widgets::{Axis, Block, Borders, Chart, Dataset, Paragraph}, DefaultTerminal, Frame
};
use rtrb::RingBuffer;
use saavy_dsp::{
    graph::{envelope::EnvNode, extensions::NodeExt, filter::FilterNode, oscillator::OscNode},
    synth::{factory::VoiceFactory, message::SynthMessage, poly::PolySynth},
};

fn main() -> Result<()> {
    color_eyre::install()?;
    let terminal = ratatui::init();

    let (mut tx, rx) = RingBuffer::<SynthMessage>::new(64);
    let factory = || {
        let osc = OscNode::square();
        let env = EnvNode::adsr(0.05, 0.1, 0.6, 0.1);
        let _lowpass = FilterNode::lowpass(200.0);
        let _highpass = FilterNode::highpass(200.0);

        osc.amplify(env).through(_lowpass)
    };

    let mut synth = PolySynth::new(48_000.0, 4, factory, rx);
    let mut buffer: Vec<f32> = vec![0.0; 2048];

    let _ = tx.push(SynthMessage::NoteOn {
        note: 60,
        velocity: 100,
    });

    let result = run(terminal, &mut synth, &mut buffer, tx);
    ratatui::restore();
    result
}

fn run<F: VoiceFactory>(
    mut terminal: DefaultTerminal,
    synth: &mut PolySynth<F>,
    buffer: &mut [f32],
    mut tx: rtrb::Producer<SynthMessage>,
) -> Result<()> {
    use std::time::{Duration, Instant};

    // Arpeggio pattern: C-E-G-C
    let notes = [60, 64, 67, 72];
    let mut note_index = 0;
    let mut last_note_time = Instant::now();
    let note_duration = Duration::from_millis(500);
    let mut current_note: Option<u8> = None;

    loop {
        // Check if it's time to play next note
        if last_note_time.elapsed() >= note_duration {
            // Release previous note
            if let Some(prev_note) = current_note {
                let _ = tx.push(SynthMessage::NoteOff { note: prev_note, velocity: 0 });
            }

            // Play new note
            let note = notes[note_index];
            let _ = tx.push(SynthMessage::NoteOn { note, velocity: 100 });
            current_note = Some(note);

            note_index = (note_index + 1) % notes.len();
            last_note_time = Instant::now();
        }

        // Render audio
        synth.render_block(buffer);

        // Draw UI
        terminal.draw(|frame| {
            render_waveform(frame, buffer);
        })?;

        // Check for quit (non-blocking)
        if event::poll(Duration::from_millis(16))? {
            if matches!(event::read()?, Event::Key(_)) {
                break Ok(());
            }
        }
    }
}

fn render_waveform(frame: &mut Frame, buffer: &[f32]) {
    let chunks = Layout::default()
        .direction(ratatui::layout::Direction::Vertical)
        .constraints([Constraint::Percentage(80), Constraint::Percentage(20)])
        .split(frame.area());
    
    let data: Vec<(f64, f64)> = buffer
        .iter()
        .enumerate()
        .map(|(i, &sample)| (i as f64, sample as f64))
        .collect();
    
    let peak = buffer.iter().fold(0.0f32, |acc, &x| acc.max(x.abs()));

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
              .borders(Borders::ALL)
        )
        .x_axis(
          Axis::default()
          .title("Sample")
          .style(Style::default().fg(Color::Gray))
          .bounds([0.0, buffer.len() as f64])
        )
        .y_axis(
          Axis::default()
          .title("Amplitude")
          .style(Style::default().fg(Color::Gray))
          .bounds([-1.0, 1.0])
          .labels(vec![
            "-1.0",
            "-0.5",
            "0.0",
            "0.5",
            "1.0",
          ]),
        );
    
    let info_text = vec![
      Line::from(format!("Peak Amplitude: {:.3}", peak)),
      Line::from(format!("Buffer Size: {} samples", buffer.len())),
      Line::from(format!("Sample Rate: 48k Hz")),
    ];

    let info = Paragraph::new(info_text)
    .block(Block::default().title("Info").borders(Borders::ALL));

    frame.render_widget(chart, chunks[0]);
    frame.render_widget(info, chunks[1]);
}
