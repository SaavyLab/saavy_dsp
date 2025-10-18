use crate::graph::node::{GraphNode, RenderCtx};

pub struct Through<S, F> {
    source: S,
    filter: F,
}

impl<S, F> Through<S, F> {
    pub fn new(source: S, filter: F) -> Self {
        Self { source, filter }
    }
}

impl<S: GraphNode, F: GraphNode> GraphNode for Through<S, F> {
    fn render_block(&mut self, out: &mut [f32], ctx: &RenderCtx) {
        self.source.render_block(out, ctx);
        self.filter.render_block(out, ctx);
    }

    fn note_on(&mut self, ctx: &RenderCtx) {
        self.source.note_on(ctx);
        self.filter.note_on(ctx);
    }

    fn note_off(&mut self, ctx: &RenderCtx) {
        self.source.note_off(ctx);
        self.filter.note_off(ctx);
    }

    fn is_active(&self) -> bool {
        self.source.is_active() || self.filter.is_active()
    }

    fn get_envelope_level(&self) -> Option<f32> {
        self.source.get_envelope_level()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::{
        envelope::EnvNode,
        extensions::NodeExt,
        node::RenderCtx,
        oscillator::OscNode,
    };

    fn ctx() -> RenderCtx {
        RenderCtx::from_freq(48_000.0, 440.0, 1.0)
    }

    #[test]
    fn renders_source_then_filter() {
        let mut node = OscNode::sine().through(EnvNode::adsr(0.01, 0.05, 0.6, 0.2));
        let mut buffer = vec![1.0; 128];
        node.render_block(&mut buffer, &ctx());

        assert!(buffer.iter().any(|&sample| sample != 1.0));
        assert!(buffer.iter().all(|&sample| sample.is_finite()));
    }

    #[test]
    fn forwards_note_events() {
        let mut node = OscNode::sine().through(EnvNode::adsr(0.01, 0.05, 0.6, 0.2));
        let ctx = ctx();

        node.note_on(&ctx);
        node.note_off(&ctx);

        assert!(node.is_active(), "through node should stay active while envelope releases");
    }

    #[test]
    fn reports_envelope_level_from_source() {
        let node = OscNode::sine().amplify(EnvNode::adsr(0.01, 0.05, 0.6, 0.2));
        let through = node.through(EnvNode::adsr(0.01, 0.05, 0.6, 0.2));

        assert!(through.get_envelope_level().is_some());
    }
}
