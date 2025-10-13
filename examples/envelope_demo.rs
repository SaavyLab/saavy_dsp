/// Demonstrates ADSR envelope behavior
/// Shows attack, decay, sustain, and release phases
use saavy_dsp::{
    graph::{envelope::EnvNode, extensions::NodeExt, node::GraphNode, oscillator::OscNode},
    MAX_BLOCK_SIZE,
};

fn main() {
    println!("=== ADSR Envelope Demo ===\n");

    let sample_rate = 48_000.0;
    let attack = 0.1; // 100ms
    let decay = 0.1; // 100ms
    let sustain = 0.5; // 50% level
    let release = 0.2; // 200ms

    println!("Envelope parameters:");
    println!("  Attack:  {:.0}ms", attack * 1000.0);
    println!("  Decay:   {:.0}ms", decay * 1000.0);
    println!("  Sustain: {:.0}%", sustain * 100.0);
    println!("  Release: {:.0}ms\n", release * 1000.0);

    // Create synth with envelope
    let mut env = EnvNode::with_params(sample_rate, attack, decay, sustain, release);
    env.note_on();
    let mut synth = OscNode::sine(440.0, sample_rate).amplify(env);

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

    println!("\n⚠ Note: This demo uses EnvNode (non-shared)");
    println!("   For runtime control (note_on/off), use SharedEnvNode + EnvelopeHandle");
    println!("   See cpal_demo.rs for an example with real-time control");

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
