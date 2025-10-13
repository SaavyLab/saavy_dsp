use crate::{dsp_fluent::voice_node::{VoiceNode}, MAX_BLOCK_SIZE};

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
    fn render_block(&mut self, out: &mut [f32]) {
        let frames = out.len();
        debug_assert!(frames <= MAX_BLOCK_SIZE);

        let carrier = &mut self.carrier[..frames];
        let gain = &mut self.gain[..frames];
        carrier.fill(0.0);
        gain.fill(0.0);

        self.signal.render_block(carrier);
        self.modulator.render_block(gain);

        for (o, (c, g)) in out.iter_mut().zip(carrier.iter().zip(gain.iter())) {
            *o = c * g;
        }
    }
}
