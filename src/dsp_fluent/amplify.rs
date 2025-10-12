use crate::{dsp_fluent::voice_node::{RenderCtx, VoiceNode}, MAX_BLOCK_SIZE};

pub struct Amplify<N, M> {
    pub signal: N,
    pub modulator: M,
    carrier: Vec<f32>,
    gain: Vec<f32>,
}

impl<N, M> Amplify<N, M> {
    pub fn new(signal: N, modulator: M) -> Self {
        Self {
            signal,
            modulator,
            carrier: vec![0.0; MAX_BLOCK_SIZE],
            gain: vec![0.0; MAX_BLOCK_SIZE],
        }
    }
}

impl<N, M> VoiceNode for Amplify<N, M>
where
    N: VoiceNode,
    M: VoiceNode,
{
    fn render_block(&mut self, ctx: &mut RenderCtx, out: &mut [f32]) {
        let carrier = &mut self.carrier[..out.len()];
        let gain = &mut self.gain[..out.len()];

        self.signal.render_block(ctx, carrier);
        self.modulator.render_block(ctx, gain);

        for (o, (c, g)) in out.iter_mut().zip(carrier.iter().zip(gain.iter())) {
            *o = c * g;
        }
    }
}
