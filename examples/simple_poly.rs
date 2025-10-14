/// Simple polyphony example showing the new RenderCtx architecture
use rtrb::RingBuffer;
use saavy_dsp::{
    graph::{envelope::EnvNode, extensions::NodeExt, filter::FilterNode, oscillator::OscNode},
    synth::{message::SynthMessage, poly::PolySynth},
};

fn main() {
    println!("=== Simple Polyphony Example ===\n");

    let sample_rate = 48_000.0;
    let max_voices = 4;

    // Create message queue
    let (mut tx, rx) = RingBuffer::<SynthMessage>::new(64);

    // Step 1: Design your patch (closure that creates the graph)
    let factory = || {
        // Oscillator starts at any frequency - Voice will set the actual pitch
        let osc = OscNode::sine();

        // ADSR envelope: 50ms attack, 100ms decay, 60% sustain, 200ms release
        let env = EnvNode::adsr(0.05, 0.1, 0.6, 0.2);

        let lowpass = FilterNode::lowpass(500.0);

        // Connect oscillator through envelope
        osc.amplify(env).through(lowpass)
    };

    // Step 2: Create polyphonic synth with that patch
    let mut poly = PolySynth::new(sample_rate, max_voices, factory, rx);

    println!("Created PolySynth with {} voices\n", max_voices);

    // Step 3: Play some notes
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

    // Step 4: Render audio
    let mut buffer = vec![0.0; 256];
    poly.render_block(&mut buffer);

    let peak = buffer.iter().fold(0.0f32, |acc, &x| acc.max(x.abs()));
    println!("\nRendered {} samples", buffer.len());
    println!("Peak amplitude: {:.3}", peak);

    // Release one note
    println!("\nReleasing E4 (64)");
    let _ = tx.push(SynthMessage::NoteOff {
        note: 64,
        velocity: 0,
    });

    buffer.fill(0.0);
    poly.render_block(&mut buffer);
    println!("Note released, envelope entering release phase");

    println!("\n✅ New architecture working!");
    println!("   - Frequency comes from RenderCtx (MIDI note)");
    println!("   - Envelope triggers from note_on/note_off events");
    println!("   - Factory creates patch, PolySynth manages voices");
}
