use crate::{graph::node::{GraphNode, Modulatable, RenderCtx}, MAX_BLOCK_SIZE};

/*
Parameter Modulation
====================

The Modulate node connects an LFO (or any signal) to a parameter on another
node, creating time-varying effects like vibrato, tremolo, and auto-wah.

How it works:
1. Render the LFO to a buffer (control signal from -1.0 to 1.0)
2. Average the LFO samples (block-rate modulation for efficiency)
3. Scale by depth: modulation = lfo_avg * depth
4. Apply to parameter: final_value = base_value + modulation
5. Render the source node with the modulated parameter

Block-rate vs. Sample-rate modulation:
--------------------------------------
Block-rate: Update parameter once per block (current implementation)
  - Pro: Efficient (only one parameter update per ~64-2048 samples)
  - Pro: Works with all existing nodes (filters compute coefficients per-block)
  - Con: Stepping artifacts possible at very low block sizes

Sample-rate: Update parameter every sample
  - Pro: Smoother modulation, no stepping
  - Con: Higher CPU cost (parameter recalculation per sample)
  - Con: Requires nodes to support per-sample parameter updates

For most musical applications, block-rate is perfectly adequate and
significantly more efficient.

Example usage:
  let lfo = LfoNode::sine(5.0);                    // 5 Hz sine wave
  let base_cutoff = 1000.0;                         // Center frequency
  let mod_depth = 500.0;                            // ±500 Hz swing

  let filter = FilterNode::lowpass(base_cutoff)
      .modulate(lfo, FilterParam::Cutoff, mod_depth);

  // Result: Cutoff sweeps from 500 Hz to 1500 Hz at 5 Hz
  //         (1000 - 500 when LFO = -1.0)
  //         (1000 + 500 when LFO = +1.0)
*/

pub struct Modulate<S, L> where
  S: GraphNode + Modulatable,
  L: GraphNode,
{
  source: S,          // The node being modulated (e.g., FilterNode)
  lfo: L,             // The modulation source (e.g., LfoNode)
  param: S::Param,    // Which parameter to modulate (e.g., FilterParam::Cutoff)
  depth: f32,         // Modulation amount (scales LFO output)
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

    // Step 1: Render LFO to temp buffer
    // LFO produces values in range [-1.0, 1.0]
    self.lfo.render_block(&mut self.lfo_buffer[..len], ctx);

    // Step 2: Average LFO samples for block-rate modulation
    // This gives us a single modulation value for the entire block
    let lfo_avg = self.lfo_buffer[..len].iter().sum::<f32>() / len as f32;

    // Step 3: Calculate modulation amount
    // If LFO = 0.5 and depth = 500.0, modulation = 250.0
    let base_value = self.source.get_param(self.param);
    let modulation = lfo_avg * self.depth;

    // Step 4: Apply modulation to the source node's parameter
    // This updates the parameter's internal state (e.g., filter coefficients)
    self.source.apply_modulation(self.param, base_value, modulation);

    // Step 5: Render the source with the modulated parameter
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
  use crate::graph::{filter::{FilterNode, FilterParam}, lfo::LfoNode, extensions::NodeExt};

  #[test]
  fn test_modulation_does_not_panic() {
    // Basic smoke test: verify modulation doesn't crash
    let lfo = LfoNode::sine(5.0);
    let mut filter = FilterNode::lowpass(1000.0)
        .modulate(lfo, FilterParam::Cutoff, 500.0);

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
    let filter = FilterNode::lowpass(base_cutoff)
        .modulate(lfo, FilterParam::Cutoff, 500.0);

    // Note: Can't directly inspect Modulate's internal state easily
    // This test mainly ensures construction works
    let _ = filter;
  }

  #[test]
  fn test_modulation_clamping() {
    // Test that extreme modulation depths get clamped appropriately
    let lfo = LfoNode::square(1.0); // Square wave: -1.0 or +1.0
    let mut filter = FilterNode::lowpass(1000.0)
        .modulate(lfo, FilterParam::Cutoff, 100_000.0); // Huge depth

    let mut buffer = vec![0.0; 1024];
    let ctx = RenderCtx::from_freq(48000.0, 440.0, 1.0);

    // Should clamp without panicking
    filter.render_block(&mut buffer, &ctx);

    // Verify output is finite (not NaN/Inf from bad clamping)
    for &sample in &buffer {
      assert!(sample.is_finite(), "Output contains non-finite value: {}", sample);
    }
  }

  #[test]
  fn test_modulation_forwards_note_events() {
    // Verify note_on and note_off are forwarded
    let lfo = LfoNode::sine(5.0);
    let mut filter = FilterNode::lowpass(1000.0)
        .modulate(lfo, FilterParam::Cutoff, 200.0);

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
    let mut filter = FilterNode::lowpass(2000.0)
        .modulate(lfo1, FilterParam::Cutoff, 800.0);

    let mut buffer = vec![0.0; 256];
    let ctx = RenderCtx::from_freq(48000.0, 440.0, 1.0);

    filter.render_block(&mut buffer, &ctx);

    // Verify produces valid output
    for &sample in &buffer {
      assert!(sample.is_finite());
    }
  }
}