/// Demonstrates the basic audio graph architecture
/// Shows how to compose oscillators, envelopes, and modulation

use saavy_dsp::graph::{
    envelope::EnvNode,
    extensions::NodeExt,
    node::GraphNode,
    oscillator::OscNode,
};

fn main() {
    println!("=== Audio Graph Basics Demo ===\n");

    let sample_rate = 48_000.0;
    let block_size = 128;

    // Example 1: Basic oscillator
    println!("1. Basic sine wave at 440Hz");
    let mut osc = OscNode::sine(440.0, sample_rate);
    let mut buffer = vec![0.0; block_size];
    osc.render_block(&mut buffer);
    println!("   Peak amplitude: {:.3}", buffer.iter().fold(0.0f32, |acc, &x| acc.max(x.abs())));

    // Example 2: Oscillator with envelope
    println!("\n2. Sine wave with ADSR envelope");
    let mut env = EnvNode::with_params(sample_rate, 0.01, 0.1, 0.7, 0.2);
    env.note_on();
    let mut enveloped_osc = OscNode::sine(440.0, sample_rate).amplify(env);

    buffer.fill(0.0);
    enveloped_osc.render_block(&mut buffer);
    println!("   Peak amplitude: {:.3}", buffer.iter().fold(0.0f32, |acc, &x| acc.max(x.abs())));
    println!("   (Lower due to envelope attack ramp)");

    // Example 3: Ring modulation (two oscillators)
    println!("\n3. Ring modulation (sine * sine)");
    let osc_carrier = OscNode::sine(440.0, sample_rate);
    let osc_modulator = OscNode::sine(30.0, sample_rate); // 30Hz tremolo
    let mut ring_mod = osc_carrier.amplify(osc_modulator);

    buffer.fill(0.0);
    ring_mod.render_block(&mut buffer);
    println!("   Peak amplitude: {:.3}", buffer.iter().fold(0.0f32, |acc, &x| acc.max(x.abs())));
    println!("   (Creates tremolo/vibrato effect)");

    // Example 4: Show first few samples
    println!("\n4. First 8 samples of 440Hz sine:");
    let mut simple_osc = OscNode::sine(440.0, sample_rate);
    let mut small_buffer = vec![0.0; 8];
    simple_osc.render_block(&mut small_buffer);
    for (i, sample) in small_buffer.iter().enumerate() {
        println!("   sample[{}] = {:.6}", i, sample);
    }

    println!("\n=== Architectural Notes ===");
    println!("• GraphNode trait: any component that processes audio");
    println!("• .amplify(x): multiplies two signals together");
    println!("• .through(x): passes signal through a processor (future)");
    println!("• All examples use zero-allocation rendering");
}
