pub struct RenderCtx {
  pub sample_rate: f32,
  pub block_size: usize,
  pub time: f64, // absolute position in render context, dt * frames
  pub dt: f32, // delta time, duration of one sample step
}

impl RenderCtx {
  pub fn new(sample_rate: f32, block_size: usize) -> Self {
    Self {
      sample_rate,
      block_size,
      time: 0.0,
      dt: 1.0 / sample_rate
    }
  }
}

pub trait VoiceNode {
  fn render_block(&mut self, ctx: &mut RenderCtx, out:  &mut [f32]);
}