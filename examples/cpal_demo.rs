
#[cfg(feature = "cpal-demo")]
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

#[cfg(feature = "cpal-demo")]
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    terminal,
};
#[cfg(feature = "cpal-demo")]
use rtrb::RingBuffer;
#[cfg(feature = "cpal-demo")]
use saavy_dsp::{
    graph::{envelope::EnvNode, extensions::NodeExt, filter::FilterNode, oscillator::OscNode},
    synth::{message::SynthMessage, poly::PolySynth},
    MAX_BLOCK_SIZE,
};

#[cfg(feature = "cpal-demo")]
fn main() {
    if let Err(err) = run() {
        eprintln!("cpal demo error: {err}");
    }
}

#[cfg(feature = "cpal-demo")]
fn run() -> Result<(), Box<dyn std::error::Error>> {
    // Setup audio device
    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .ok_or_else(|| "no default output device available")?;
    let config = device.default_output_config()?;

    if config.sample_format() != cpal::SampleFormat::F32 {
        return Err("cpal demo currently supports only f32 output".into());
    }

    let sample_rate = config.sample_rate().0 as f32;
    let channels = config.channels() as usize;

    // Create synth with message queue
    let (mut tx, rx) = RingBuffer::<SynthMessage>::new(64);

    let factory = || {
        let osc = OscNode::sine();
        let env = EnvNode::adsr(0.05, 0.1, 0.6, 0.2);
        let filter = FilterNode::lowpass(500.0);
        osc.amplify(env).through(filter)
    };

    let mut synth = PolySynth::new(sample_rate, 4, factory, rx);
    let mut buffer = vec![0.0; MAX_BLOCK_SIZE];

    // Audio callback - reads from synth
    let stream = device.build_output_stream(
        &config.into(),
        move |data: &mut [f32], _| {
            let total_frames = data.len() / channels;
            let mut frames_written = 0;

            // Process in chunks if requested size exceeds buffer
            while frames_written < total_frames {
                let frames_remaining = total_frames - frames_written;
                let frames_to_render = frames_remaining.min(MAX_BLOCK_SIZE);

                let buf = &mut buffer[..frames_to_render];
                synth.render_block(buf);

                // Debug: check if we're getting audio
                let peak = buf.iter().fold(0.0f32, |acc, &x| acc.max(x.abs()));
                if peak > 0.001 {
                    eprintln!("Audio peak: {:.3}", peak);
                }

                // Copy mono to all channels
                let output_offset = frames_written * channels;
                for (i, &sample) in buf.iter().enumerate() {
                    for ch in 0..channels {
                        data[output_offset + i * channels + ch] = sample;
                    }
                }

                frames_written += frames_to_render;
            }
        },
        move |err| eprintln!("Stream error: {err}"),
        None,
    )?;

    stream.play()?;
    println!("Space: gate on/off, Esc: quit");

    // Main thread - keyboard input, writes to tx
    control_loop(&mut tx)?;

    Ok(())
}

#[cfg(feature = "cpal-demo")]
fn control_loop(tx: &mut rtrb::Producer<SynthMessage>) -> Result<(), Box<dyn std::error::Error>> {
    terminal::enable_raw_mode()?;
    let mut gate_on = false;

    loop {
        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char(' ') => match key.kind {
                        KeyEventKind::Press if !gate_on => {
                            eprintln!("Sending NoteOn");
                            let _ = tx.push(SynthMessage::NoteOn {
                                note: 57, // A3
                                velocity: 100,
                            });
                            gate_on = true;
                        }
                        KeyEventKind::Release if gate_on => {
                            eprintln!("Sending NoteOff");
                            let _ = tx.push(SynthMessage::NoteOff {
                                note: 57,
                                velocity: 0,
                            });
                            gate_on = false;
                        }
                        _ => {}
                    },
                    KeyCode::Esc => break,
                    _ => {}
                }
            }
        }
    }

    terminal::disable_raw_mode()?;
    println!("\nExiting...");
    Ok(())
}

#[cfg(not(feature = "cpal-demo"))]
fn main() {
    eprintln!("Build with --features cpal-demo to run this example.");
}
