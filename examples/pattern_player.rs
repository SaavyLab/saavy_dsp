//! Simple example demonstrating the Pattern API with audio output.
//!
//! Run with: cargo run --example pattern_player

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use rtrb::RingBuffer;
use saavy_dsp::{
    graph::{envelope::EnvNode, extensions::NodeExt, filter::FilterNode, oscillator::OscNode},
    pattern,
    sequencing::*,
    synth::{message::SynthMessage, synth::Synth},
    MAX_BLOCK_SIZE,
};
use std::{thread, time::Duration};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Pattern Player ===\n");

    // --- Define patterns using the new API ---

    // Simple C major arpeggio
    let arp = pattern!(4/4 => [C4, E4, G4, C5]);

    // Bassline with rests
    let bass = pattern!(4/4 => [C3, _, C3, _]);

    // Melody with some subdivisions (eighth notes)
    let melody = pattern!(4/4 => [G4, [A4, B4], C5, _]);

    // Triplet pattern
    let triplet = pattern!(4/4 => [[C4, E4, G4], _, [G4, E4, C4], _]);

    // Chain them together: bass, arp 2x, then melody, then triplet
    let song = bass.clone().repeat(2)
        .then(arp.clone())
        .then(arp.clone())
        .then(melody.clone())
        .then(triplet.clone());

    // Convert to sequence for playback
    let ppq = 480;
    let sequence = song.to_sequence(ppq);

    println!("Patterns created:");
    println!("  - Bass:     C3 _ C3 _");
    println!("  - Arpeggio: C4 E4 G4 C5");
    println!("  - Melody:   G4 [A4 B4] C5 _");
    println!("  - Triplet:  [C4 E4 G4] _ [G4 E4 C4] _");
    println!("\nPlaying: bass x2 -> arp x2 -> melody -> triplet");
    println!("Total events: {}", sequence.events.len());
    println!("Total ticks: {}\n", sequence.total_ticks);

    // --- Set up audio ---

    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .ok_or("no output device")?;
    let config = device.default_output_config()?;
    let sample_rate = config.sample_rate().0 as f32;
    let channels = config.channels() as usize;

    println!("Audio: {} Hz, {} channels\n", sample_rate, channels);

    // Message queue for synth
    let (msg_tx, msg_rx) = RingBuffer::<SynthMessage>::new(64);

    // Voice factory - simple subtractive patch
    let factory = || {
        let osc = OscNode::sawtooth();
        let env = EnvNode::adsr(0.01, 0.1, 0.5, 0.3);
        let filter = FilterNode::lowpass(2000.0);
        osc.amplify(env).through(filter)
    };

    // Audio callback
    let mut render_buf = vec![0.0f32; MAX_BLOCK_SIZE];
    let stream = device.build_output_stream(
        &config.into(),
        {
            let mut synth = Synth::new(sample_rate, 8, factory, msg_rx);
            move |data: &mut [f32], _| {
                let total_frames = data.len() / channels;
                let mut frames_written = 0;
                while frames_written < total_frames {
                    let frames_remaining = total_frames - frames_written;
                    let frames_to_render = frames_remaining.min(MAX_BLOCK_SIZE);

                    let block = &mut render_buf[..frames_to_render];
                    synth.render_block(block);

                    // Mono to all channels
                    let out_off = frames_written * channels;
                    for (i, &s) in block.iter().enumerate() {
                        for ch in 0..channels {
                            data[out_off + i * channels + ch] = s;
                        }
                    }
                    frames_written += frames_to_render;
                }
            }
        },
        |err| eprintln!("Audio error: {err}"),
        None,
    )?;

    stream.play()?;

    // --- Sequencer thread ---
    // Simple tick-based playback

    let bpm = 120.0;
    let ticks_per_beat = ppq;
    let seconds_per_tick = 60.0 / (bpm * ticks_per_beat as f64);
    let ms_per_tick = (seconds_per_tick * 1000.0) as u64;

    println!("Playing at {} BPM ({:.1}ms per tick)", bpm, seconds_per_tick * 1000.0);
    println!("Press Ctrl+C to stop\n");

    thread::spawn({
        let mut tx = msg_tx;
        let events = sequence.events.clone();
        let total_ticks = sequence.total_ticks;

        move || {
            loop {
                let mut current_tick = 0u32;
                let mut event_idx = 0;

                // Active notes (note, end_tick)
                let mut active_notes: Vec<(u8, u32)> = Vec::new();

                while current_tick <= total_ticks {
                    // Trigger any events at this tick
                    while event_idx < events.len() {
                        let event = &events[event_idx];
                        let event_tick = (event.tick_offset as i32 + event.offset_ticks) as u32;

                        if event_tick <= current_tick {
                            if let Some(note) = event.note {
                                // Note on
                                let _ = tx.push(SynthMessage::NoteOn {
                                    note,
                                    velocity: event.velocity,
                                });
                                // Schedule note off
                                active_notes.push((note, current_tick + event.duration_ticks));
                                print!("{} ", note_name(note));
                            }
                            event_idx += 1;
                        } else {
                            break;
                        }
                    }

                    // Check for note offs
                    active_notes.retain(|&(note, end_tick)| {
                        if current_tick >= end_tick {
                            let _ = tx.push(SynthMessage::NoteOff { note, velocity: 0 });
                            false
                        } else {
                            true
                        }
                    });

                    // Advance time
                    thread::sleep(Duration::from_millis(ms_per_tick));
                    current_tick += 1;
                }

                // Release any remaining notes
                for (note, _) in active_notes.drain(..) {
                    let _ = tx.push(SynthMessage::NoteOff { note, velocity: 0 });
                }

                println!("\n[loop]");
            }
        }
    });

    // Keep main thread alive
    loop {
        thread::sleep(Duration::from_secs(1));
    }
}

/// Convert MIDI note to name for display
fn note_name(note: u8) -> &'static str {
    const NAMES: [&str; 12] = ["C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"];
    let octave = (note / 12) as i32 - 1;
    let name = NAMES[(note % 12) as usize];
    // Leak a string for simplicity (it's a demo)
    Box::leak(format!("{}{}", name, octave).into_boxed_str())
}
