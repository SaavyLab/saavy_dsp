use crate::graph::node::{GraphNode, RenderCtx};

/*
Serial Signal Chain (Through)
=============================

Through connects two nodes in series, passing the output of the first (source)
into the second (effect). This is the fundamental building block for creating
signal processing chains like: oscillator → filter → delay.

How It Works:
-------------
1. Render the source into the output buffer
2. Pass that buffer through the effect (in-place processing)

  Source renders:  [0.5, 0.8, -0.3, 0.9, ...]
  Effect processes in-place (e.g., filter)
  Final output:    [0.4, 0.6, -0.2, 0.7, ...]  (filtered result)

This is different from Amplify (which multiplies) or Mix (which blends).
Through passes audio through a processor that transforms it.

Common Use Cases:
-----------------

1. Subtractive Synthesis Chain:
   The classic synth signal path.

     let voice = OscNode::sawtooth()
         .through(FilterNode::lowpass(1000.0))
         .amplify(EnvNode::adsr(0.01, 0.1, 0.7, 0.3));

   - Sawtooth → filter (shape tone) → envelope (shape volume)

2. Adding Effects:
   Chain multiple effects in series.

     let wet_signal = source
         .through(FilterNode::highpass(200.0))
         .through(DelayNode::new(0.3, 0.4));

   - Remove low frequencies, then add delay

3. Multi-stage Filtering:
   Stack filters for steeper rolloff or complex EQ.

     let steep_filter = source
         .through(FilterNode::lowpass(2000.0))
         .through(FilterNode::lowpass(2000.0));

   - Two 12dB filters = 24dB/octave rolloff

Through vs Amplify vs Mix:
--------------------------
- Through: Serial processing (source → effect → output)
- Amplify: Multiplication (signal × modulator)
- Mix:     Parallel blending (dry + wet)

Signal Flow Diagram:
--------------------
  Through: [Source] ──→ [Effect] ──→ output

  Amplify: [Signal] ──┬──→ (×) ──→ output
           [Mod]    ──┘

  Mix:     [Dry] ────┬──→ (+) ──→ output
           [Wet] ────┘

Choose Through when audio flows from one processor to the next.
*/

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
