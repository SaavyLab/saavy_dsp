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
}
