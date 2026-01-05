use crate::graph::node::{GraphNode, RenderCtx};

/*
Through Node
============

Connects two nodes in series: source output flows into effect input.
This is how you build signal processing chains.

When to Use Through
-------------------

Use `.through()` to chain processors - audio flows from one to the next:

  // Classic subtractive synth: oscillator → filter → envelope
  let voice = OscNode::sawtooth()
      .through(FilterNode::lowpass(1000.0))
      .amplify(EnvNode::adsr(0.01, 0.1, 0.7, 0.3));

  // Stack multiple effects
  let processed = source
      .through(FilterNode::highpass(200.0))
      .through(DelayNode::new(0.3, 0.4));

  // Steeper filter (two 12dB = 24dB/octave)
  let steep = source
      .through(FilterNode::lowpass(2000.0))
      .through(FilterNode::lowpass(2000.0));


Through vs Amplify vs Mix
-------------------------

  .through()  →  Serial chain (source → effect → output)
  .amplify()  →  Multiplies signals (envelope control)
  .mix()      →  Adds signals in parallel (layering)

Signal flow:

  Through: [Source] ──→ [Effect] ──→ output
  Amplify: [Signal] ──┬──→ (×) ──→ output
           [Mod]    ──┘
  Mix:     [A] ────┬──→ (+) ──→ output
           [B] ────┘

Quick rule: passing audio through a filter or effect? Use `.through()`.


Order Matters!
--------------

  osc.through(filter).through(distortion)  // Distort filtered signal
  osc.through(distortion).through(filter)  // Filter distorted signal
  // These sound VERY different!


How It Works
------------

See `dsp/through.rs` for details on:
- In-place processing (why effects modify buffers directly)
- Signal chain ordering and its effect on sound
- Insert vs send/return effect patterns
*/

/// Chains two graph nodes in series (source → effect).
pub struct Through<S, E> {
    /// The signal source
    source: S,
    /// The effect/processor
    effect: E,
}

impl<S, E> Through<S, E> {
    pub fn new(source: S, effect: E) -> Self {
        Self { source, effect }
    }
}

impl<S: GraphNode, E: GraphNode> GraphNode for Through<S, E> {
    fn render_block(&mut self, out: &mut [f32], ctx: &RenderCtx) {
        // Render source into buffer
        self.source.render_block(out, ctx);
        // Pass buffer through effect (in-place processing)
        self.effect.render_block(out, ctx);
    }

    fn note_on(&mut self, ctx: &RenderCtx) {
        self.source.note_on(ctx);
        self.effect.note_on(ctx);
    }

    fn note_off(&mut self, ctx: &RenderCtx) {
        self.source.note_off(ctx);
        self.effect.note_off(ctx);
    }

    fn is_active(&self) -> bool {
        self.source.is_active() || self.effect.is_active()
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
