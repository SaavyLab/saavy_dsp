/// Regression test for polyphonic synthesis
/// Ensures that:
/// - PolySynth initializes correctly
/// - Audio rendering doesn't panic
/// - Output is in valid range [-1, 1]
/// - Output contains actual signal (not silence)
use rtrb::RingBuffer;
use saavy_dsp::{
    graph::{
        delay::DelayNode, envelope::EnvNode, extensions::NodeExt, filter::FilterNode,
        oscillator::OscNode,
    },
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

/// Tests that PolySynth renders deterministically
/// Given the same input sequence, should produce identical output
/// This catches issues with:
/// - Uninitialized memory
/// - Non-deterministic state management
/// - State leakage between notes (e.g., delay buffers, LFO phase)
/// - Improper resets when voices are triggered
#[test]
fn polysynth_deterministic_renders() {
    let sample_rate = 48_000.0;
    let max_voices = 4;
    let block_size = 256;

    // Simulate a C major arpeggio sequence (like cpal_scope example)
    let note_sequence = vec![
        (
            0,
            SynthMessage::NoteOn {
                note: 60,
                velocity: 100,
            },
        ), // C
        (
            100,
            SynthMessage::NoteOff {
                note: 60,
                velocity: 0,
            },
        ),
        (
            120,
            SynthMessage::NoteOn {
                note: 64,
                velocity: 100,
            },
        ), // E
        (
            220,
            SynthMessage::NoteOff {
                note: 64,
                velocity: 0,
            },
        ),
        (
            240,
            SynthMessage::NoteOn {
                note: 67,
                velocity: 100,
            },
        ), // G
        (
            340,
            SynthMessage::NoteOff {
                note: 67,
                velocity: 0,
            },
        ),
        (
            360,
            SynthMessage::NoteOn {
                note: 72,
                velocity: 100,
            },
        ), // C (high)
        (
            460,
            SynthMessage::NoteOff {
                note: 72,
                velocity: 0,
            },
        ),
        // Repeat the sequence to test for drift
        (
            500,
            SynthMessage::NoteOn {
                note: 60,
                velocity: 100,
            },
        ),
        (
            600,
            SynthMessage::NoteOff {
                note: 60,
                velocity: 0,
            },
        ),
        (
            620,
            SynthMessage::NoteOn {
                note: 64,
                velocity: 100,
            },
        ),
        (
            720,
            SynthMessage::NoteOff {
                note: 64,
                velocity: 0,
            },
        ),
        (
            740,
            SynthMessage::NoteOn {
                note: 67,
                velocity: 100,
            },
        ),
        (
            840,
            SynthMessage::NoteOff {
                note: 67,
                velocity: 0,
            },
        ),
        (
            860,
            SynthMessage::NoteOn {
                note: 72,
                velocity: 100,
            },
        ),
        (
            960,
            SynthMessage::NoteOff {
                note: 72,
                velocity: 0,
            },
        ),
    ];

    // Helper to create a synth and render a sequence
    let render_sequence = || {
        let (mut tx, rx) = RingBuffer::<SynthMessage>::new(64);
        let factory = || {
            let osc = OscNode::sine();
            let osc_saw = OscNode::sawtooth();
            let env = EnvNode::adsr(0.01, 0.1, 0.7, 0.3);
            let lowpass = FilterNode::lowpass(1000.0);
            let highpass = FilterNode::highpass(200.0);
            let delay = DelayNode::new(0.5, sample_rate, 0.5);

            osc.mix(osc_saw, 0.5)
                .amplify(env)
                .through(lowpass)
                .through(highpass)
                .through(delay)
        };
        let mut synth = PolySynth::new(sample_rate, max_voices, factory, rx);

        let mut all_samples = Vec::new();
        let mut block_idx = 0;

        // Schedule messages
        for (target_block, msg) in note_sequence.iter() {
            // Render blocks until we reach the target block
            while block_idx < *target_block {
                let mut buffer = vec![0.0; block_size];
                synth.render_block(&mut buffer);
                all_samples.extend_from_slice(&buffer);
                block_idx += 1;
            }
            // Send the message
            let _ = tx.push(*msg);
        }

        // Render a few more blocks to capture the tail
        for _ in 0..50 {
            let mut buffer = vec![0.0; block_size];
            synth.render_block(&mut buffer);
            all_samples.extend_from_slice(&buffer);
        }

        all_samples
    };

    // Render the same sequence twice
    let output1 = render_sequence();
    let output2 = render_sequence();

    // Outputs should be identical
    assert_eq!(output1.len(), output2.len(), "Output lengths should match");

    let mut first_mismatch = None;
    for (i, (&sample1, &sample2)) in output1.iter().zip(output2.iter()).enumerate() {
        if sample1 != sample2 {
            first_mismatch = Some((i, sample1, sample2));
            break;
        }
    }

    if let Some((idx, s1, s2)) = first_mismatch {
        let block = idx / block_size;
        let sample = idx % block_size;
        panic!(
            "Non-deterministic rendering detected!\n\
             First mismatch at sample {idx} (block {block}, sample {sample}):\n\
             Run 1: {s1}\n\
             Run 2: {s2}\n\
             This indicates state leakage or uninitialized memory in the synth."
        );
    }

    // Verify we got audible signal
    let peak = output1.iter().fold(0.0f32, |acc, &x| acc.max(x.abs()));
    assert!(
        peak > 0.001,
        "Output should contain audible signal, got peak: {peak}"
    );

    println!(
        "✓ Rendered {} samples deterministically (peak: {:.4})",
        output1.len(),
        peak
    );
}
