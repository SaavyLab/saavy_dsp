use crate::{
    dsp::envelope::{Envelope, EnvelopeState},
    graph::node::{GraphNode, RenderCtx},
};

pub struct EnvNode {
    env: Envelope,
}

impl EnvNode {
    pub fn new() -> Self {
        let env = Envelope::new();
        Self { env }
    }

    pub fn adsr(attack: f32, decay: f32, sustain: f32, release: f32) -> Self {
        let env = Envelope::adsr(attack, decay, sustain, release);
        Self { env }
    }

    /// Get the current envelope level (for visualization)
    pub fn level(&self) -> f32 {
        self.env.level()
    }

    /// Get the current envelope state (for visualization)
    pub fn state(&self) -> EnvelopeState {
        self.env.state()
    }
}

impl GraphNode for EnvNode {
    fn render_block(&mut self, out: &mut [f32], ctx: &RenderCtx) {
        self.env.render(out, ctx);
    }

    fn note_on(&mut self, ctx: &RenderCtx) {
        self.env.note_on(ctx);
    }

    fn note_off(&mut self, ctx: &RenderCtx) {
        self.env.note_off(ctx);
    }

    fn is_active(&self) -> bool {
        self.env.is_active()
    }

    fn get_envelope_level(&self) -> Option<f32> {
        Some(self.env.level())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_RATE: f32 = 1_000.0;

    fn ctx() -> RenderCtx {
        RenderCtx::from_freq(SAMPLE_RATE, 440.0, 100.0)
    }

    #[test]
    fn render_outputs_envelope_levels() {
        let mut node = EnvNode::adsr(0.01, 0.02, 0.5, 0.1);
        let mut buffer = vec![0.0; 128];
        let ctx = ctx();

        node.note_on(&ctx);
        node.render_block(&mut buffer, &ctx);

        assert!(buffer.iter().any(|&sample| sample > 0.0));
        assert!(buffer.iter().all(|&sample| sample <= 1.0));
    }

    #[test]
    fn level_and_state_reflect_internal_envelope() {
        let mut node = EnvNode::adsr(0.01, 0.02, 0.4, 0.1);
        let ctx = ctx();

        node.note_on(&ctx);
        let level_after_on = node.level();
        assert!(level_after_on >= 0.0);

        node.note_off(&ctx);
        let mut buffer = vec![0.0; 256];
        node.render_block(&mut buffer, &ctx);

        assert!(node.level() <= 1.0);
        assert!(matches!(node.state(), EnvelopeState::Release | EnvelopeState::Idle));
    }
}
