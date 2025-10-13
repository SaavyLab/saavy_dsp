/// Demonstrates ADSR envelope behavior
/// Shows attack, decay, sustain, and release phases

use saavy_dsp::graph::{
    envelope::EnvNode,
    extensions::NodeExt,
    node::GraphNode,
    oscillator::OscNode,
};

fn main() {
    println!("=== ADSR Envelope Demo ===\n");

    let sample_rate = 48_000.0;
    let attack = 0.1;   // 100ms
    let decay = 0.1;    // 100ms
    let sustain = 0.5;  // 50% level
    let release = 0.2;  // 200ms

    println!("Envelope parameters:");
    println!("  Attack:  {:.0}ms", attack * 1000.0);
    println!("  Decay:   {:.0}ms", decay * 1000.0);
    println!("  Sustain: {:.0}%", sustain * 100.0);
    println!("  Release: {:.0}ms\n", release * 1000.0);

    // Create synth with envelope
    let mut env = EnvNode::with_params(sample_rate, attack, decay, sustain, release);
    env.note_on();
    let mut synth = OscNode::sine(440.0, sample_rate).amplify(env);

    // Simulate phases
    let attack_samples = (attack * sample_rate) as usize;
    let decay_samples = (decay * sample_rate) as usize;
    let sustain_samples = (0.5 * sample_rate) as usize; // Hold for 500ms
    let release_samples = (release * sample_rate) as usize;

    println!("Phase timeline:");

    // Attack phase
    let mut buffer = vec![0.0; attack_samples];
    synth.render_block(&mut buffer);
    let attack_peak = buffer.iter().fold(0.0f32, |acc, &x| acc.max(x.abs()));
    println!("  Attack:  {:6} samples, peak amplitude: {:.3}", attack_samples, attack_peak);

    // Decay phase
    buffer.resize(decay_samples, 0.0);
    buffer.fill(0.0);
    synth.render_block(&mut buffer);
    let decay_end = buffer[buffer.len() - 1].abs();
    println!("  Decay:   {:6} samples, end amplitude:  {:.3}", decay_samples, decay_end);

    // Sustain phase
    buffer.resize(sustain_samples, 0.0);
    buffer.fill(0.0);
    synth.render_block(&mut buffer);
    let sustain_avg = buffer.iter().map(|&x| x.abs()).sum::<f32>() / buffer.len() as f32;
    println!("  Sustain: {:6} samples, avg amplitude:  {:.3}", sustain_samples, sustain_avg);

    // Trigger release (note_off not available on non-shared EnvNode)
    // In real usage, you'd use SharedEnvNode with EnvelopeHandle
    println!("\n⚠ Note: This demo uses EnvNode (non-shared)");
    println!("   For runtime control (note_on/off), use SharedEnvNode + EnvelopeHandle");
    println!("   See cpal_demo.rs for an example with real-time control");

    println!("\n=== Envelope Behavior ===");
    println!("• Attack:  ramps from 0.0 → 1.0");
    println!("• Decay:   ramps from 1.0 → sustain level");
    println!("• Sustain: holds at sustain level until note_off");
    println!("• Release: ramps from current level → 0.0");
    println!("\nEnvelope is re-trigger safe: can call note_on during release");
}
