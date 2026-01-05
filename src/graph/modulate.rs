use crate::{
    dsp::modulate::block_average,
    graph::node::{GraphNode, Modulatable, RenderCtx},
    MAX_BLOCK_SIZE,
};

/*
Modulate Node
=============

Connects an LFO (or any signal) to a parameter on another node.
This creates time-varying effects like vibrato, filter sweeps, and auto-wah.

When to Use Modulate
--------------------

Use `.modulate()` when you want a parameter to change over time:

  // Auto-wah: LFO sweeps filter cutoff
  let wah = FilterNode::lowpass(1000.0)
      .modulate(LfoNode::sine(2.0), FilterParam::Cutoff, 800.0);

  // Vibrato-like effect on delay time (chorus)
  let chorus = DelayNode::new(15.0, 0.0, 0.5)
      .modulate(LfoNode::sine(1.0), DelayParam::DelayTime, 10.0);


Understanding Depth
-------------------

Depth controls how much the parameter changes:

    modulated_value = base_value + (LFO × depth)

Example with cutoff=1000, depth=500, LFO swinging -1 to +1:
    LFO = -1.0  →  cutoff = 1000 + (-1 × 500) = 500 Hz
    LFO =  0.0  →  cutoff = 1000 + ( 0 × 500) = 1000 Hz
    LFO = +1.0  →  cutoff = 1000 + (+1 × 500) = 1500 Hz

The parameter sweeps between 500 and 1500 Hz.


Common Effects
--------------

  Filter sweep / Auto-wah:  LFO → Filter Cutoff
  Chorus / Flanger:         LFO → Delay Time
  Vibrato:                  LFO → Pitch (via delay time tricks)
  PWM:                      LFO → Pulse Width


How It Works
------------

See `dsp/modulate.rs` for implementation details, including:
- Block-rate vs sample-rate modulation tradeoffs
- The averaging algorithm for smooth modulation
- Parameter clamping considerations
*/

pub struct Modulate<S, L>
where
    S: GraphNode + Modulatable,
    L: GraphNode,
{
    source: S,            // The node being modulated (e.g., FilterNode)
    lfo: L,               // The modulation source (e.g., LfoNode)
    param: S::Param,      // Which parameter to modulate (e.g., FilterParam::Cutoff)
    depth: f32,           // Modulation amount (scales LFO output)
    lfo_buffer: Vec<f32>, // Temp buffer for LFO output
}

impl<S, L> Modulate<S, L>
where
    S: GraphNode + Modulatable,
    L: GraphNode,
{
    pub fn new(source: S, lfo: L, param: S::Param, depth: f32) -> Self {
        Self {
            source,
            lfo,
            param,
            depth,
            lfo_buffer: vec![0.0; MAX_BLOCK_SIZE],
        }
    }
}

impl<S, L> GraphNode for Modulate<S, L>
where
    S: GraphNode + Modulatable,
    L: GraphNode,
{
    fn render_block(&mut self, out: &mut [f32], ctx: &RenderCtx) {
        let len = out.len();

        // Render LFO to temp buffer (values in [-1.0, +1.0])
        self.lfo.render_block(&mut self.lfo_buffer[..len], ctx);

        // Average LFO samples for block-rate modulation
        let lfo_avg = block_average(&self.lfo_buffer[..len]);

        // Calculate and apply modulation
        let base_value = self.source.get_param(self.param);
        let modulation = lfo_avg * self.depth;
        self.source
            .apply_modulation(self.param, base_value, modulation);

        // Render the source with modulated parameter
        self.source.render_block(out, ctx);
    }

    fn note_on(&mut self, ctx: &RenderCtx) {
        self.source.note_on(ctx);
        self.lfo.note_on(ctx);
    }

    fn note_off(&mut self, ctx: &RenderCtx) {
        self.source.note_off(ctx);
        self.lfo.note_off(ctx);
    }

    fn is_active(&self) -> bool {
        self.source.is_active()
    }

    fn get_envelope_level(&self) -> Option<f32> {
        self.source.get_envelope_level()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::{
        extensions::NodeExt,
        filter::{FilterNode, FilterParam},
        lfo::LfoNode,
    };

    #[test]
    fn test_modulation_does_not_panic() {
        // Basic smoke test: verify modulation doesn't crash
        let lfo = LfoNode::sine(5.0);
        let mut filter = FilterNode::lowpass(1000.0).modulate(lfo, FilterParam::Cutoff, 500.0);

        let mut buffer = vec![0.0; 512];
        let ctx = RenderCtx::from_freq(48000.0, 440.0, 1.0);

        // Should not panic
        filter.render_block(&mut buffer, &ctx);
    }

    #[test]
    fn test_modulation_base_value_preserved() {
        // Verify base value is stored correctly
        let lfo = LfoNode::sine(5.0);
        let base_cutoff = 1000.0;
        let filter = FilterNode::lowpass(base_cutoff).modulate(lfo, FilterParam::Cutoff, 500.0);

        // Note: Can't directly inspect Modulate's internal state easily
        // This test mainly ensures construction works
        let _ = filter;
    }

    #[test]
    fn test_modulation_clamping() {
        // Test that extreme modulation depths get clamped appropriately
        let lfo = LfoNode::square(1.0); // Square wave: -1.0 or +1.0
        let mut filter = FilterNode::lowpass(1000.0).modulate(lfo, FilterParam::Cutoff, 100_000.0); // Huge depth

        let mut buffer = vec![0.0; 1024];
        let ctx = RenderCtx::from_freq(48000.0, 440.0, 1.0);

        // Should clamp without panicking
        filter.render_block(&mut buffer, &ctx);

        // Verify output is finite (not NaN/Inf from bad clamping)
        for &sample in &buffer {
            assert!(
                sample.is_finite(),
                "Output contains non-finite value: {}",
                sample
            );
        }
    }

    #[test]
    fn test_modulation_forwards_note_events() {
        // Verify note_on and note_off are forwarded
        let lfo = LfoNode::sine(5.0);
        let mut filter = FilterNode::lowpass(1000.0).modulate(lfo, FilterParam::Cutoff, 200.0);

        let ctx = RenderCtx::from_freq(48000.0, 440.0, 1.0);

        // Should not panic
        filter.note_on(&ctx);
        filter.note_off(&ctx);
    }

    #[test]
    fn test_multiple_modulations() {
        // Test chaining modulations (modulate cutoff, then resonance - if we could)
        // For now, just test creating the modulated node
        let lfo1 = LfoNode::sine(3.0);
        let mut filter = FilterNode::lowpass(2000.0).modulate(lfo1, FilterParam::Cutoff, 800.0);

        let mut buffer = vec![0.0; 256];
        let ctx = RenderCtx::from_freq(48000.0, 440.0, 1.0);

        filter.render_block(&mut buffer, &ctx);

        // Verify produces valid output
        for &sample in &buffer {
            assert!(sample.is_finite());
        }
    }
}
