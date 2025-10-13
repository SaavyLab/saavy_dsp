/// Demonstrates polyphonic synthesis without real-time audio
/// Shows voice allocation, note triggering, and mixing

use rtrb::RingBuffer;
use saavy_dsp::{
    synth::{message::SynthMessage, poly::PolySynth},
    MAX_BLOCK_SIZE,
};

fn main() {
    println!("=== Polyphony Demo (Offline) ===\n");

    let sample_rate = 48_000.0;
    let max_voices = 4;
    let block_size = 256;

    // Create message queue
    let (mut tx, rx) = RingBuffer::<SynthMessage>::new(64);

    // Create polyphonic synth with 4 voices
    let mut poly = PolySynth::new(
        sample_rate,
        max_voices,
        rx,
        0.05, // attack: 50ms
        0.1,  // decay: 100ms
        0.6,  // sustain: 60%
        0.2,  // release: 200ms
    );

    println!("Created PolySynth with {} voices\n", max_voices);

    // Play a C major chord (C4, E4, G4)
    println!("Playing C major chord:");
    println!("  Note On: C4 (60)");
    let _ = tx.push(SynthMessage::NoteOn {
        note: 60,
        velocity: 100,
    });
    println!("  Note On: E4 (64)");
    let _ = tx.push(SynthMessage::NoteOn {
        note: 64,
        velocity: 100,
    });
    println!("  Note On: G4 (67)");
    let _ = tx.push(SynthMessage::NoteOn {
        note: 67,
        velocity: 100,
    });

    // Render a block with 3 active voices
    let mut buffer = vec![0.0; block_size];
    poly.render_block(&mut buffer);

    let peak = buffer
        .iter()
        .fold(0.0f32, |acc, &x| acc.max(x.abs()));
    println!("\nAfter first render:");
    println!("  Active voices: 3");
    println!("  Peak amplitude: {:.3}", peak);

    // Add a 4th note
    println!("\nAdding 4th note: B4 (71)");
    let _ = tx.push(SynthMessage::NoteOn {
        note: 71,
        velocity: 100,
    });

    buffer.fill(0.0);
    poly.render_block(&mut buffer);
    let peak = buffer
        .iter()
        .fold(0.0f32, |acc, &x| acc.max(x.abs()));
    println!("  Active voices: 4 (max)");
    println!("  Peak amplitude: {:.3}", peak);

    // Try to add a 5th note (triggers voice stealing)
    println!("\nAdding 5th note: D5 (74) - triggers voice stealing!");
    let _ = tx.push(SynthMessage::NoteOn {
        note: 74,
        velocity: 100,
    });

    buffer.fill(0.0);
    poly.render_block(&mut buffer);
    println!("  Active voices: 4 (max, oldest voice stolen)");

    // Release two notes
    println!("\nReleasing E4 (64) and G4 (67)");
    let _ = tx.push(SynthMessage::NoteOff {
        note: 64,
        velocity: 0,
    });
    let _ = tx.push(SynthMessage::NoteOff {
        note: 67,
        velocity: 0,
    });

    buffer.fill(0.0);
    poly.render_block(&mut buffer);
    println!("  2 voices releasing, 2 still active");

    // Render many blocks to let released voices finish
    println!("\nRendering 100 blocks to let envelopes release...");
    for _ in 0..100 {
        buffer.fill(0.0);
        poly.render_block(&mut buffer);
    }
    println!("  Released voices should be free now");

    // All notes off
    println!("\nSending AllNotesOff");
    let _ = tx.push(SynthMessage::AllNotesOff);
    buffer.fill(0.0);
    poly.render_block(&mut buffer);
    println!("  All voices entering release phase");

    println!("\n=== Polyphony Architecture ===");
    println!("• Voice allocation: finds free voice or steals oldest releasing");
    println!("• Voice states: Free → Active → Releasing → Free");
    println!("• Lock-free messaging: control thread sends, audio thread receives");
    println!("• Mixing: all active voices rendered and summed");
    println!("\nSee cpal_demo.rs for real-time polyphonic playback!");
}
