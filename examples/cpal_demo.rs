use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

use rtrb::RingBuffer;
use saavy_dsp::{
    graph::{envelope::EnvNode, extensions::NodeExt, filter::FilterNode, oscillator::OscNode},
    synth::{message::SynthMessage, poly::PolySynth},
    MAX_BLOCK_SIZE,
};
use std::{thread, time::Duration};

fn main() {
    if let Err(err) = run() {
        eprintln!("cpal demo error: {err}");
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
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

    let (mut tx, rx) = RingBuffer::<SynthMessage>::new(64);

    let factory = || {
        let osc = OscNode::saw();
        let env = EnvNode::adsr(0.05, 0.1, 0.6, 0.2);
        let lowpass = FilterNode::lowpass(250.0);
        let highpass = FilterNode::highpass(600.0);
        osc.amplify(env).through(highpass).through(lowpass)
    };

    let mut synth = PolySynth::new(sample_rate, 4, factory, rx);
    let mut buffer = vec![0.0; MAX_BLOCK_SIZE];

    let stream = device.build_output_stream(
        &config.into(),
        move |data: &mut [f32], _| {
            let total_frames = data.len() / channels;
            let mut frames_written = 0;

            while frames_written < total_frames {
                let frames_remaining = total_frames - frames_written;
                let frames_to_render = frames_remaining.min(MAX_BLOCK_SIZE);

                let buf = &mut buffer[..frames_to_render];
                synth.render_block(buf);

                // Debug helper: log when we have audible signal
                let peak = buf.iter().fold(0.0f32, |acc, &x| acc.max(x.abs()));
                if peak > 0.001 {
                    eprintln!("Audio peak: {:.3}", peak);
                }

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
    println!("Playing repeating C-major arpeggio (C4–E4–G4–C5) at ~120 BPM. Press Ctrl+C to stop.");

    play_arpeggio(&mut tx);

    Ok(())
}

fn play_arpeggio(tx: &mut rtrb::Producer<SynthMessage>) {
    let notes = [60, 64, 67, 72]; // MIDI: C4, E4, G4, C5
    let note_duration = Duration::from_millis(450); // ≈ quarter note at 120 BPM
    let gap = Duration::from_millis(50); // short release gap between notes

    loop {
        for &note in &notes {
            eprintln!("NoteOn {note}");
            let _ = tx.push(SynthMessage::NoteOn {
                note,
                velocity: 100,
            });
            thread::sleep(note_duration);

            eprintln!("NoteOff {note}");
            let _ = tx.push(SynthMessage::NoteOff { note, velocity: 0 });
            thread::sleep(gap);
        }
    }
}
