/// Regression test for polyphonic synthesis
/// Ensures that:
/// - PolySynth initializes correctly
/// - Audio rendering doesn't panic
/// - Output is in valid range [-1, 1]
/// - Output contains actual signal (not silence)
use rtrb::RingBuffer;
use saavy_dsp::{
    graph::{envelope::EnvNode, extensions::NodeExt, oscillator::OscNode},
    synth::{message::SynthMessage, poly::PolySynth},
};

#[test]
fn polysynth_renders_valid_audio() {
    let sample_rate = 48_000.0;
    let max_voices = 4;
    let block_size = 256;

    // Create polyphonic synth with factory
    let (mut tx, rx) = RingBuffer::<SynthMessage>::new(64);
    let factory = || {
        let osc = OscNode::sine();
        let env = EnvNode::adsr(0.01, 0.1, 0.7, 0.3);
        osc.amplify(env)
    };
    let mut synth = PolySynth::new(sample_rate, max_voices, factory, rx);

    // Trigger a note
    let _ = tx.push(SynthMessage::NoteOn {
        note: 60, // Middle C
        velocity: 100,
    });

    // Render audio
    let mut buffer = vec![0.0; block_size];
    synth.render_block(&mut buffer);

    // Assertions
    assert!(
        buffer.iter().any(|&s| s.abs() > 0.0),
        "Output should contain signal, not silence"
    );

    assert!(
        buffer.iter().all(|&s| s.abs() <= 1.0),
        "All samples should be in range [-1.0, 1.0]"
    );

    // Verify we get reasonable amplitude (envelope should be ramping up)
    let peak = buffer.iter().fold(0.0f32, |acc, &x| acc.max(x.abs()));
    assert!(
        peak > 0.001,
        "Peak amplitude too low: {peak}, envelope may not be working"
    );
}

#[test]
fn polysynth_handles_multiple_voices() {
    let sample_rate = 48_000.0;
    let max_voices = 4;
    let block_size = 256;

    let (mut tx, rx) = RingBuffer::<SynthMessage>::new(64);
    let factory = || {
        let osc = OscNode::sine();
        let env = EnvNode::adsr(0.01, 0.1, 0.7, 0.3);
        osc.amplify(env)
    };
    let mut synth = PolySynth::new(sample_rate, max_voices, factory, rx);

    // Play a chord (3 notes)
    let _ = tx.push(SynthMessage::NoteOn {
        note: 60,
        velocity: 100,
    });
    let _ = tx.push(SynthMessage::NoteOn {
        note: 64,
        velocity: 100,
    });
    let _ = tx.push(SynthMessage::NoteOn {
        note: 67,
        velocity: 100,
    });

    let mut buffer = vec![0.0; block_size];
    synth.render_block(&mut buffer);

    // Should produce louder output with 3 voices
    let peak = buffer.iter().fold(0.0f32, |acc, &x| acc.max(x.abs()));
    assert!(
        peak > 0.001,
        "Multi-voice output should have audible signal"
    );
    assert!(peak <= 1.0, "Output should not clip");
}

#[test]
fn polysynth_note_off_works() {
    let sample_rate = 48_000.0;
    let max_voices = 4;
    let block_size = 256;

    let (mut tx, rx) = RingBuffer::<SynthMessage>::new(64);
    let factory = || {
        let osc = OscNode::sine();
        let env = EnvNode::adsr(0.01, 0.1, 0.7, 0.3);
        osc.amplify(env)
    };
    let mut synth = PolySynth::new(sample_rate, max_voices, factory, rx);

    // Note on
    let _ = tx.push(SynthMessage::NoteOn {
        note: 60,
        velocity: 100,
    });

    let mut buffer = vec![0.0; block_size];
    synth.render_block(&mut buffer);
    let peak_during_note = buffer.iter().fold(0.0f32, |acc, &x| acc.max(x.abs()));

    // Note off
    let _ = tx.push(SynthMessage::NoteOff {
        note: 60,
        velocity: 0,
    });

    // Render many blocks to let envelope release
    for _ in 0..200 {
        buffer.fill(0.0);
        synth.render_block(&mut buffer);
    }

    let peak_after_release = buffer.iter().fold(0.0f32, |acc, &x| acc.max(x.abs()));

    // After release, amplitude should be significantly lower
    assert!(
        peak_during_note > 0.001,
        "Note should produce audible output"
    );
    assert!(
        peak_after_release < peak_during_note * 0.1,
        "After release, amplitude should drop significantly"
    );
}
