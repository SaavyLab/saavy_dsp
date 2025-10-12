use crate::dsp_fluent::voice_node::{RenderCtx, VoiceNode};

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
      carrier: Vec::new(),
      gain: Vec::new(),
    }
  }
}

impl<N, M> VoiceNode for Amplify<N, M> where N: VoiceNode, M: VoiceNode {
  fn render_block(&mut self, ctx: &mut RenderCtx, out: &mut [f32]) {
    self.carrier.resize(out.len(), 0.0);
    self.gain.resize(out.len(), 0.0);

    self.signal.render_block(ctx, &mut self.carrier);
    self.modulator.render_block(ctx, &mut self.gain);

    for (o, (c, g)) in out.iter_mut().zip(self.carrier.iter().zip(&self.gain)) {
      *o = c * g;
    }
  }
}