use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

use rtrb::RingBuffer;
use saavy_dsp::{
    graph::{
        delay::{DelayNode, DelayParam}, envelope::EnvNode, extensions::NodeExt, filter::{FilterNode, FilterParam}, lfo::LfoNode, oscillator::OscNode
    },
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
        let osc = OscNode::sine();
        let env = EnvNode::adsr(0.05, 0.1, 0.6, 0.2);

        // Chorus effect: modulate delay time with slow LFO
        let delay_lfo = LfoNode::sine(0.5);  // 0.5 Hz - slow wobble
        let delay = DelayNode::new(30.0, 0.2, 0.5)
            .modulate(delay_lfo, DelayParam::DelayTime, 10.0);  // 30ms ±10ms = chorus

        osc.amplify(env).through(delay)
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
            let _ = tx.push(SynthMessage::NoteOn {
                note,
                velocity: 100,
            });
            thread::sleep(note_duration);

            let _ = tx.push(SynthMessage::NoteOff { note, velocity: 0 });
            thread::sleep(gap);
        }
    }
}
