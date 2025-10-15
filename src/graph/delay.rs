use crate::{dsp::delay::DelayLine, graph::node::GraphNode};

pub struct DelayNode {
    delay_line: DelayLine,
    delay_ms: f32,
}

impl DelayNode {
    pub fn new(delay_ms: f32) -> Self {
        Self {
            delay_line: DelayLine::new(),
            delay_ms,
        }
    }
}

impl GraphNode for DelayNode {
    fn render_block(&mut self, out: &mut [f32], ctx: &super::node::RenderCtx) {
        let delay_samples = ((self.delay_ms / 1000.0) * ctx.sample_rate) as usize;
        self.delay_line.render(out, delay_samples);
    }

    fn note_on(&mut self, _ctx: &super::node::RenderCtx) {
        // Clear buffer here to avoid clicks
        self.delay_line.reset();
    }
}
