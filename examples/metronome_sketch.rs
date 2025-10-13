/// Sketch showing how metronome would use direct frequency (not MIDI notes)
use saavy_dsp::graph::{
    envelope::EnvNode,
    extensions::NodeExt,
    node::{GraphNode, RenderCtx},
    oscillator::OscNode,
};

fn main() {
    println!("=== Metronome Frequency Demo ===\n");

    let sample_rate = 48_000.0;

    // Create a clave-style sound (bright click)
    let mut clave = OscNode::sine().amplify(EnvNode::adsr(0.001, 0.01, 0.0, 0.02));

    // Metronome uses direct frequency, not MIDI notes!
    // 2500Hz = bright click sound
    let ctx = RenderCtx::from_freq(sample_rate, 2500.0, 1.0);

    // Trigger the sound
    clave.note_on(&ctx);

    // Render
    let mut buffer = vec![0.0; 256];
    clave.render_block(&mut buffer, &ctx);

    let peak = buffer.iter().fold(0.0f32, |acc, &x| acc.max(x.abs()));
    println!("Rendered clave click at 2500Hz");
    println!("Peak amplitude: {:.3}", peak);

    println!("\n✅ No MIDI note needed!");
    println!("   - Direct frequency: 2500Hz");
    println!("   - Perfect for metronomes, drum machines, sound effects");
    println!("   - Still can use MIDI notes with RenderCtx::from_note()");
}
