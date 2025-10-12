use crate::dsp::oscillator::OscillatorBlock;
use crate::dsp::oscillator::OscillatorWaveform;
use crate::dsp_fluent::voice_node::RenderCtx;
use crate::dsp_fluent::voice_node::VoiceNode;

pub struct OscNode {
  osc: OscillatorBlock,
}

impl OscNode {
  pub fn sine(freq: f32, sr: f32) -> Self {
    let osc = OscillatorBlock::new(freq, sr, OscillatorWaveform::Sine);
    Self { osc }
  }

  pub fn saw(freq: f32, sr: f32) -> Self {
    let osc = OscillatorBlock::new(freq, sr, OscillatorWaveform::Saw);
    Self { osc }
  }
}

impl VoiceNode for OscNode {
  fn render_block(&mut self, ctx: &mut RenderCtx, out: &mut [f32]) {
    self.osc.render(out, 1.0);
    ctx.time += ctx.dt as f64 * out.len() as f64;
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::f32::consts::TAU;
  
  #[test]
  fn valid_sine() {
    let freq = 440.0;
    let sample_rate = 48_000.0;
    let block_size = 128;

    let mut ctx = RenderCtx::new(sample_rate, block_size);
    let mut synth = OscNode::sine(freq, sample_rate);
    // .amplify(EnvelopeNode::adsr(...))
    // .through(FilterNode::lowpass(...));

    let mut buffer = vec![0.0f32; block_size];
    synth.render_block(&mut ctx, &mut buffer);

    // sample n should be sin(2pi f n / sr)
    let sample_index = 12;
    let expected = (TAU * freq * sample_index as f32 / sample_rate).sin();
    let actual = buffer[sample_index];
    assert!( (actual - expected).abs() < 1e-6, "expected {expected}, got {actual}" );
  }
}