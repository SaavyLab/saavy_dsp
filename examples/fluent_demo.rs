use saavy_dsp::dsp_fluent::{
    envelope_node::AdsrEnvNode,
    node_extension::NodeExt,
    oscillator_node::OscNode,
    voice_node::{RenderCtx, VoiceNode},
};

fn main() {
    let freq = 440.0;
    let sample_rate = 48_000.0;
    let block_size = 128;

    let mut ctx = RenderCtx::new(sample_rate, block_size);
    let mut envelope = AdsrEnvNode::with_params(sample_rate, 0.01, 0.7, 0.6, 0.1);
    envelope.note_on();
    let mut synth = OscNode::sine(freq, sample_rate).amplify(envelope);
    // .through(FilterNode::lowpass(...));

    let mut buffer = vec![0.0f32; block_size];
    synth.render_block(&mut ctx, &mut buffer);

    println!(
        "first sixty-four samples: {:?}",
        &buffer[..64.min(buffer.len())]
    );
    println!("playhead time after render: {:.6} s", ctx.time);
}
