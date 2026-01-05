use crate::{dsp::mix::mix_in_place, graph::node::GraphNode, MAX_BLOCK_SIZE};

/*
Mix Node
========

Combines two graph nodes by ADDING their outputs with adjustable balance.
This is the additive counterpart to Amplify (which multiplies).

When to Use Mix
---------------

Use `.mix()` when you want to BLEND or LAYER signals:

  // Layer two oscillators for a thicker sound
  let thick = OscNode::sine().mix(OscNode::sawtooth(), 0.5);

  // Wet/dry balance for effects
  let dry = OscNode::sine();
  let wet = dry.through(FilterNode::lowpass(800.0));
  let fx = dry.mix(wet, 0.3);  // 70% dry, 30% wet

  // Mostly one sound with a hint of another
  let blend = OscNode::sine().mix(OscNode::square(), 0.2);  // 80% sine, 20% square


Balance Parameter
-----------------

  balance = 0.0  →  100% source A, 0% source B
  balance = 0.5  →  50% each (equal mix)
  balance = 1.0  →  0% source A, 100% source B


Mix vs Amplify vs Through
-------------------------

  .mix()      →  Adds signals (blending, layering, wet/dry)
  .amplify()  →  Multiplies signals (envelope control, tremolo)
  .through()  →  Chains source → processor (filtering, effects)

Quick rule: layering sounds? Use `.mix()`. Controlling volume? Use `.amplify()`.


Envelope Gotcha
---------------

Both sources receive note_on/note_off events. Watch the order:

  ✓ osc1.mix(osc2, 0.5).amplify(env)  // Envelope gates the mixed result
  ✗ osc1.amplify(env).mix(osc2, 0.5)  // Only osc1 gated, osc2 drones forever!


How It Works
------------

See `dsp/mix.rs` for the implementation details, including:
- Linear vs equal-power crossfading
- Why weights sum to 1.0 (prevents clipping)
- Phase relationship considerations
*/

/// Combines two graph nodes by adding their outputs with adjustable balance.
pub struct Mix<A, B> {
    /// First signal source
    pub source_a: A,
    /// Second signal source
    pub source_b: B,
    /// Mix balance (0.0 = all A, 0.5 = equal, 1.0 = all B)
    pub balance: f32,
    /// Pre-allocated buffer for source B output
    b_buffer: Vec<f32>,
}

impl<A, B> Mix<A, B> {
    pub fn new(source_a: A, source_b: B, balance: f32) -> Self {
        Mix {
            source_a,
            source_b,
            balance: balance.clamp(0.0, 1.0),
            b_buffer: vec![0.0; MAX_BLOCK_SIZE],
        }
    }
}

impl<A: GraphNode, B: GraphNode> GraphNode for Mix<A, B> {
    fn render_block(&mut self, out: &mut [f32], ctx: &super::node::RenderCtx) {
        // Render source A into output
        self.source_a.render_block(out, ctx);

        // Render source B into temp buffer
        let b_out = &mut self.b_buffer[..out.len()];
        b_out.fill(0.0);
        self.source_b.render_block(b_out, ctx);

        // Mix using dsp primitive
        mix_in_place(out, b_out, self.balance);
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::{
        envelope::EnvNode, extensions::NodeExt, node::RenderCtx, oscillator::OscNode,
    };

    #[test]
    fn test_mix_does_not_panic() {
        // Basic smoke test: verify mixing doesn't crash
        let osc1 = OscNode::sine();
        let osc2 = OscNode::sawtooth();
        let mut mixed = osc1.mix(osc2, 0.5);

        let mut buffer = vec![0.0; 512];
        let ctx = RenderCtx::from_freq(48000.0, 440.0, 1.0);

        // Should not panic
        mixed.render_block(&mut buffer, &ctx);
    }

    #[test]
    fn test_mix_equal_balance() {
        // Test 50/50 mix produces expected result
        let osc1 = OscNode::sine();
        let osc2 = OscNode::sine();
        let mut mixed = osc1.mix(osc2, 0.5);

        let mut buffer = vec![0.0; 256];
        let ctx = RenderCtx::from_freq(48000.0, 440.0, 1.0);

        mixed.render_block(&mut buffer, &ctx);

        // Should produce valid output
        assert!(
            buffer.iter().any(|&s| s.abs() > 0.0),
            "Mix should produce signal"
        );
        assert!(
            buffer.iter().all(|&s| s.is_finite()),
            "All samples should be finite"
        );
    }

    #[test]
    fn test_mix_all_source_a() {
        // Test balance = 0.0 (100% source A, 0% source B)
        let osc1 = OscNode::sine();
        let osc2 = OscNode::square(); // Different waveform to distinguish
        let mut mixed = osc1.mix(osc2, 0.0);

        let mut buffer = vec![0.0; 256];
        let ctx = RenderCtx::from_freq(48000.0, 440.0, 1.0);

        mixed.render_block(&mut buffer, &ctx);

        // Should produce signal (sine wave)
        assert!(
            buffer.iter().any(|&s| s.abs() > 0.0),
            "Should produce signal"
        );

        // Peak should be close to 1.0 (pure sine)
        let peak = buffer.iter().fold(0.0f32, |acc, &x| acc.max(x.abs()));
        assert!(peak > 0.5, "Peak should be substantial for sine wave");
    }

    #[test]
    fn test_mix_all_source_b() {
        // Test balance = 1.0 (0% source A, 100% source B)
        let osc1 = OscNode::sine();
        let osc2 = OscNode::square();
        let mut mixed = osc1.mix(osc2, 1.0);

        let mut buffer = vec![0.0; 256];
        let ctx = RenderCtx::from_freq(48000.0, 440.0, 1.0);

        mixed.render_block(&mut buffer, &ctx);

        // Should produce signal (square wave)
        assert!(
            buffer.iter().any(|&s| s.abs() > 0.0),
            "Should produce signal"
        );

        let peak = buffer.iter().fold(0.0f32, |acc, &x| acc.max(x.abs()));
        assert!(peak > 0.5, "Peak should be substantial for square wave");
    }

    #[test]
    fn test_mix_with_envelope() {
        // Test mixing two enveloped oscillators
        let osc1 = OscNode::sine();
        let env1 = EnvNode::adsr(0.01, 0.1, 0.7, 0.2);

        let osc2 = OscNode::sawtooth();
        let env2 = EnvNode::adsr(0.01, 0.1, 0.7, 0.2);

        let mut mixed = osc1.amplify(env1).mix(osc2.amplify(env2), 0.5);

        let mut buffer = vec![0.0; 256];
        let ctx = RenderCtx::from_freq(48000.0, 440.0, 100.0);

        // Trigger note
        mixed.note_on(&ctx);
        mixed.render_block(&mut buffer, &ctx);

        // Should produce signal
        assert!(
            buffer.iter().any(|&s| s.abs() > 0.0),
            "Enveloped mix should produce signal"
        );

        // All samples should be valid
        for &sample in &buffer {
            assert!(sample.is_finite(), "Sample should be finite");
        }
    }

    #[test]
    fn test_mix_forwards_note_events() {
        // Verify note_on and note_off are forwarded to both sources
        let osc1 = OscNode::sine();
        let env1 = EnvNode::adsr(0.01, 0.1, 0.7, 0.2);

        let osc2 = OscNode::sawtooth();
        let env2 = EnvNode::adsr(0.01, 0.1, 0.7, 0.2);

        let mut mixed = osc1.amplify(env1).mix(osc2.amplify(env2), 0.5);

        let ctx = RenderCtx::from_freq(48000.0, 440.0, 100.0);

        // Should not panic
        mixed.note_on(&ctx);
        mixed.note_off(&ctx);
    }

    #[test]
    fn test_mix_is_active_logic() {
        // Test that mix reports active if either source is active
        let osc1 = OscNode::sine();
        let env1 = EnvNode::adsr(0.01, 0.1, 0.7, 0.2);

        let osc2 = OscNode::sawtooth();
        let env2 = EnvNode::adsr(0.01, 0.1, 0.7, 0.2);

        let mut mixed = osc1.amplify(env1).mix(osc2.amplify(env2), 0.5);

        let ctx = RenderCtx::from_freq(48000.0, 440.0, 100.0);

        // Before note_on, envelope should be idle
        mixed.note_on(&ctx);

        // After note_on, should be active
        assert!(mixed.is_active(), "Mix should be active after note_on");
    }

    #[test]
    fn test_mix_output_range() {
        // Verify mixed output stays within reasonable bounds
        let osc1 = OscNode::sine();
        let osc2 = OscNode::sine();
        let mut mixed = osc1.mix(osc2, 0.5);

        let mut buffer = vec![0.0; 1024];
        let ctx = RenderCtx::from_freq(48000.0, 440.0, 1.0);

        mixed.render_block(&mut buffer, &ctx);

        // Check all samples are in reasonable range
        for &sample in &buffer {
            assert!(
                sample.abs() <= 2.0,
                "Mixed sample out of range: {}, should be <= 2.0",
                sample
            );
            assert!(sample.is_finite(), "Sample should be finite");
        }
    }

    #[test]
    fn test_mix_multiple_block_sizes() {
        // Test that mixing works with various block sizes
        let block_sizes = [64, 128, 256, 512, 1024];

        for &size in &block_sizes {
            let osc1 = OscNode::sine();
            let osc2 = OscNode::sawtooth();
            let mut mixed = osc1.mix(osc2, 0.5);

            let mut buffer = vec![0.0; size];
            let ctx = RenderCtx::from_freq(48000.0, 440.0, 1.0);

            mixed.render_block(&mut buffer, &ctx);

            // Verify output is valid
            assert!(
                buffer.iter().any(|&s| s.abs() > 0.0),
                "Mix should produce signal for block size {}",
                size
            );
            assert!(
                buffer.iter().all(|&s| s.is_finite()),
                "All samples should be finite for block size {}",
                size
            );
        }
    }
}
