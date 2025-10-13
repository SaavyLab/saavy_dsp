/// Demonstrates ADSR envelope behavior
/// Shows attack, decay, sustain, and release phases
use rtrb::RingBuffer;
use saavy_dsp::{
    graph::{envelope::EnvNode, extensions::NodeExt, oscillator::OscNode},
    synth::{message::SynthMessage, poly::PolySynth},
    MAX_BLOCK_SIZE,
};

fn main() {
    println!("=== ADSR Envelope Demo ===\n");

    let sample_rate = 48_000.0;
    let max_voices = 1; // Only need one voice for this demo
    let attack = 0.1; // 100ms
    let decay = 0.1; // 100ms
    let sustain = 0.5; // 50% level
    let release = 0.2; // 200ms

    println!("Envelope parameters:");
    println!("  Attack:  {:.0}ms", attack * 1000.0);
    println!("  Decay:   {:.0}ms", decay * 1000.0);
    println!("  Sustain: {:.0}%", sustain * 100.0);
    println!("  Release: {:.0}ms\n", release * 1000.0);

    // Create message queue
    let (mut tx, rx) = RingBuffer::<SynthMessage>::new(64);

    // Design patch
    let factory = || {
        let osc = OscNode::sine();
        let env = EnvNode::adsr(attack, decay, sustain, release);
        osc.amplify(env)
    };

    // Create synth
    let mut synth = PolySynth::new(sample_rate, max_voices, factory, rx);

    // Trigger note on!
    let _ = tx.push(SynthMessage::NoteOn {
        note: 60, // Middle C
        velocity: 100,
    });

    // Calculate phase durations
    let attack_samples = (attack * sample_rate) as usize;
    let decay_samples = (decay * sample_rate) as usize;
    let sustain_samples = (0.5 * sample_rate) as usize; // Hold for 500ms

    println!("Phase timeline:");
    println!("  (Rendering in chunks of {} samples)", MAX_BLOCK_SIZE);
    println!();

    // Helper function to render in chunks
    let mut render_phase = |name: &str, total_samples: usize| -> (f32, f32) {
        let mut all_samples = Vec::new();
        let mut remaining = total_samples;

        while remaining > 0 {
            let chunk_size = remaining.min(MAX_BLOCK_SIZE);
            let mut buffer = vec![0.0; chunk_size];
            synth.render_block(&mut buffer);
            all_samples.extend_from_slice(&buffer);
            remaining -= chunk_size;
        }

        let peak = all_samples.iter().fold(0.0f32, |acc, &x| acc.max(x.abs()));
        let avg = all_samples.iter().map(|&x| x.abs()).sum::<f32>() / all_samples.len() as f32;

        println!(
            "  {:8} {:6} samples, peak: {:.3}, avg: {:.3}",
            name, total_samples, peak, avg
        );

        (peak, avg)
    };

    // Render each phase
    render_phase("Attack:", attack_samples);
    render_phase("Decay:", decay_samples);
    render_phase("Sustain:", sustain_samples);

    println!("\n=== Envelope Behavior ===");
    println!("• Attack:  ramps from 0.0 → 1.0");
    println!("• Decay:   ramps from 1.0 → sustain level");
    println!("• Sustain: holds at sustain level until note_off");
    println!("• Release: ramps from current level → 0.0");
    println!(
        "\n• Graph nodes limited to MAX_BLOCK_SIZE ({}) samples per render",
        MAX_BLOCK_SIZE
    );
    println!("• For longer durations, render in multiple chunks");
}
