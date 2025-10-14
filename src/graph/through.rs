use crate::graph::node::{GraphNode, RenderCtx};

pub struct Through<S, F> {
  source: S,
  filter: F, 
}

impl<S, F> Through<S, F> {
  pub fn new(source: S, filter: F) -> Self {
    Self {
      source,
      filter
    }
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
      self.source.is_active() | self.filter.is_active()
  }
}
