use crate::{graph::node::GraphNode, MAX_BLOCK_SIZE};

pub struct Mix<A, B> {
    pub source_a: A,
    pub source_b: B,
    pub balance: f32, // 0.5 = equal, 0.0 = all A, 1.0 = all B
    temp_buffer: Vec<f32>,
}

impl<A, B> Mix<A, B> {
    pub fn new(source_a: A, source_b: B, balance: f32) -> Self {
        Mix {
            source_a,
            source_b,
            balance,
            temp_buffer: vec![0.0; MAX_BLOCK_SIZE],
        }
    }
}

impl<S: GraphNode, M: GraphNode> GraphNode for Mix<S, M> {
    fn render_block(&mut self, out: &mut [f32], ctx: &super::node::RenderCtx) {
        self.source_a.render_block(out, ctx);

        let frames = &mut self.temp_buffer[..out.len()];
        frames.fill(0.0);

        self.source_b.render_block(frames, ctx);

        for (o, b) in out.iter_mut().zip(frames.iter()) {
            let weight_a = 1.0 - self.balance;
            let weight_b = self.balance;
            *o = (*o * weight_a) + (*b * weight_b);
        }
    }

    fn note_on(&mut self, ctx: &super::node::RenderCtx) {
        self.source_a.note_on(ctx);
        self.source_b.note_on(ctx);
    }

    fn note_off(&mut self, ctx: &super::node::RenderCtx) {
        self.source_a.note_off(ctx);
        self.source_b.note_off(ctx);
    }

    fn is_active(&self) -> bool {
        self.source_a.is_active() || self.source_b.is_active()
    }

    fn get_envelope_level(&self) -> Option<f32> {
        match (
            self.source_a.get_envelope_level(),
            self.source_b.get_envelope_level(),
        ) {
            (Some(a), Some(b)) => Some(a.max(b)),
            (Some(a), None) => Some(a),
            (None, Some(b)) => Some(b),
            (None, None) => None,
        }
    }
}
