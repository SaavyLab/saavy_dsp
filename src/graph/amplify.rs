use crate::{
    graph::node::{GraphNode, RenderCtx},
    MAX_BLOCK_SIZE,
};

pub struct Amplify<N, M> {
    pub signal: N,
    pub modulator: M,
    temp_buffer: Vec<f32>,
}

impl<N, M> Amplify<N, M> {
    pub fn new(signal: N, modulator: M) -> Self {
        Self {
            signal,
            modulator,
            temp_buffer: vec![0.0; MAX_BLOCK_SIZE],
        }
    }
}

impl<N: GraphNode, M: GraphNode> GraphNode for Amplify<N, M> {
    fn render_block(&mut self, out: &mut [f32], ctx: &RenderCtx) {
        // Render signal into output
        self.signal.render_block(out, ctx);

        // Slice temp buffer to match output size (RT-safe, no allocation)
        let frames = &mut self.temp_buffer[..out.len()];
        frames.fill(0.0);
        self.modulator.render_block(frames, ctx);

        // Multiply signal by modulator (ring modulation / amplitude control)
        for (o, m) in out.iter_mut().zip(frames.iter()) {
            *o *= *m;
        }
    }

    fn note_on(&mut self, ctx: &RenderCtx) {
        self.signal.note_on(ctx);
        self.modulator.note_on(ctx);
    }

    fn note_off(&mut self, ctx: &RenderCtx) {
        self.signal.note_off(ctx);
        self.modulator.note_off(ctx);
    }

    fn is_active(&self) -> bool {
        self.modulator.is_active() | self.signal.is_active()
    }
}
