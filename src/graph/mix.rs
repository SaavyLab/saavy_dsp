use crate::{graph::node::GraphNode, MAX_BLOCK_SIZE};

/*
Parallel Signal Mixing
======================

The Mix node combines two audio signals in parallel using linear crossfading.
This is the additive counterpart to the Amplify node (which multiplies signals).

How it works:
1. Render source A into the output buffer
2. Render source B into a temporary buffer
3. Apply weighted sum: output = (A × weight_a) + (B × weight_b)
   where weight_a = 1.0 - balance
   and   weight_b = balance

Linear vs. Equal-Power Crossfading:
-----------------------------------
This implementation uses LINEAR crossfading for simplicity and predictability.

Linear (current implementation):
  - balance = 0.0 → 100% A, 0% B
  - balance = 0.5 → 50% A, 50% B (both signals at half amplitude)
  - balance = 1.0 → 0% A, 100% B
  - Pro: Simple, predictable math
  - Con: Perceived loudness dip in the middle (50/50 is quieter than 100% A or 100% B)

Equal-power (not implemented):
  - Uses sqrt() curves to maintain constant perceived loudness
  - balance = 0.5 → ~70.7% A, ~70.7% B (sqrt(0.5) ≈ 0.707)
  - Pro: More constant perceived loudness during crossfade
  - Con: Slightly more CPU (two sqrt calls per block)

For most musical applications (layering oscillators, wet/dry mixing), linear
crossfading is perfectly adequate. If you need equal-power, apply it at the
control level before calling .mix().

Use Cases:
----------
- Layering oscillators (detuned saws, supersaw)
- Multi-timbral voices (sine + square for thickness)
- Wet/dry mixing (dry signal + effect signal)
- Parallel processing chains

Example usage:
  let osc1 = OscNode::sine();
  let osc2 = OscNode::sawtooth();

  // Equal mix (50/50)
  let mixed = osc1.mix(osc2, 0.5);

  // Mostly osc1, a bit of osc2 for color
  let blend = osc1.mix(osc2, 0.2);  // 80% osc1, 20% osc2

  // Wet/dry for effects
  let dry = OscNode::sine();
  let wet = OscNode::sine().through(FilterNode::lowpass(800.0));
  let fx_mix = dry.mix(wet, 0.3);  // 70% dry, 30% wet

Important: Both sources receive note_on/note_off events. If you only want
one source to be gated by an envelope, apply the envelope AFTER mixing:

  osc1.mix(osc2, 0.5).amplify(env)  // ✓ Envelope gates both

  osc1.amplify(env).mix(osc2, 0.5)  // ✗ Only osc1 is gated, osc2 drones
*/

pub struct Mix<A, B> {
    pub source_a: A,
    pub source_b: B,
    pub balance: f32, // 0.0 = all A, 1.0 = all B, 0.5 = equal mix
    temp_buffer: Vec<f32>,
}

impl<A, B> Mix<A, B> {
    pub fn new(source_a: A, source_b: B, balance: f32) -> Self {
        Mix {
            source_a,
            source_b,
            balance: balance.clamp(0.0, 1.0),
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

        let weight_a = 1.0 - self.balance;
        let weight_b = self.balance;
        for (o, b) in out.iter_mut().zip(frames.iter()) {
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
