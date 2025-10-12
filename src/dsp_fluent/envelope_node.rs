use std::sync::{Arc, Mutex};

use crate::dsp::envelope::AdsrEnvelope;
use crate::dsp_fluent::voice_node::VoiceNode;

pub struct AdsrEnvNode {
  env: AdsrEnvelope,
}

impl AdsrEnvNode {
  pub fn note_on(&mut self) {
    self.env.note_on();
  }

  pub fn note_off(&mut self) {
    self.env.note_off();
  }

  pub fn new(sample_rate: f32) -> Self {
    let env = AdsrEnvelope::new(sample_rate);
    Self { env }
  }

  pub fn with_params(sample_rate: f32, attack: f32, decay: f32, sustain: f32, release: f32) -> Self {
    let env = AdsrEnvelope::with_params(sample_rate, attack, decay, sustain, release);
    Self { env }
  }
}

impl VoiceNode for AdsrEnvNode {
  fn render_block(&mut self, ctx: &mut super::voice_node::RenderCtx, out:  &mut [f32]) {
      self.env.render(out);
      ctx.time += ctx.dt as f64 * out.len() as f64;
  }
}

#[derive(Clone)]
pub struct EnvelopeHandle {
  inner: Arc<Mutex<AdsrEnvelope>>,
}

impl EnvelopeHandle {
  pub fn note_on(&self) {
    if let Ok(mut env) = self.inner.lock() {
      env.note_on();
    }
  }

  pub fn note_off(&self) {
    if let Ok(mut env) = self.inner.lock() {
      env.note_off();
    }
  }
}

pub struct SharedAdsrEnvNode {
  env: Arc<Mutex<AdsrEnvelope>>,
}

impl SharedAdsrEnvNode {
  pub fn with_params(sample_rate: f32, attack: f32, decay: f32, sustain: f32, release: f32) -> (Self, EnvelopeHandle) {
    let env = AdsrEnvelope::with_params(sample_rate, attack, decay, sustain, release);
    let arc = Arc::new(Mutex::new(env));
    let handle = EnvelopeHandle { inner: Arc::clone(&arc) };
    (Self { env: arc }, handle)
  }
}

impl VoiceNode for SharedAdsrEnvNode {
  fn render_block(&mut self, ctx: &mut super::voice_node::RenderCtx, out:  &mut [f32]) {
      if let Ok(mut env) = self.env.lock() {
        env.render(out);
      } else {
        out.fill(0.0);
      }
      ctx.time += ctx.dt as f64 * out.len() as f64;
  }
}
