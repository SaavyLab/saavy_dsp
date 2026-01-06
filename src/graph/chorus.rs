use crate::dsp::delay::DelayLine;
use crate::graph::node::{GraphNode, Modulatable, RenderCtx};
use std::f32::consts::TAU;

/*
Chorus Effect
=============

Chorus thickens a sound by mixing the dry signal with a slightly delayed,
pitch-modulated copy. The modulation creates subtle detuning that makes
one voice sound like several playing together.

How It Works
------------

1. Input signal passes through unchanged (dry)
2. A copy is sent through a short delay (~15-25ms)
3. An LFO modulates the delay time, creating pitch variation
4. Dry and wet signals are mixed together

The modulating delay creates small pitch shifts (as delay time changes,
pitch goes up/down slightly). This mimics the natural variation when
multiple musicians play the same part.

Parameters
----------

Rate (0.1 - 5.0 Hz):
  LFO speed. Slower = subtle shimmer, faster = vibrato-like wobble.
  Classic chorus: 0.5-1.5 Hz

Depth (0.5 - 5.0 ms):
  How much the delay time varies. More depth = more dramatic effect.
  Subtle: 1-2ms, Obvious: 3-5ms

Mix (0.0 - 1.0):
  Dry/wet blend. 0.3-0.5 is typical for chorus.

Base Delay (~20ms):
  The center delay time. Keeps wet signal distinct from dry.
  Too short: comb filtering. Too long: slapback echo.

Example usage:

  // Subtle chorus on pad
  let lush_pad = OscNode::sawtooth()
      .through(ChorusNode::new(0.8, 2.0, 0.4))
      .amplify(EnvNode::adsr(0.3, 0.1, 0.8, 0.5));

  // Obvious chorus on lead
  let wide_lead = OscNode::square()
      .through(ChorusNode::new(1.5, 4.0, 0.5))
      .through(FilterNode::lowpass(3000.0));
*/

/// Parameters that can be modulated
#[derive(Clone, Copy, Debug)]
pub enum ChorusParam {
    /// LFO rate in Hz
    Rate,
    /// Modulation depth in ms
    Depth,
    /// Dry/wet mix
    Mix,
}

/// Chorus effect - thickens sound with modulated delay
pub struct ChorusNode {
    delay_line: DelayLine,
    lfo_phase: f32,
    rate: f32,        // LFO Hz
    depth_ms: f32,    // Modulation depth in ms
    mix: f32,         // Dry/wet
    base_delay_ms: f32,
}

impl ChorusNode {
    /// Create a new chorus effect.
    ///
    /// - `rate`: LFO speed in Hz (0.1-5.0 typical, 0.8-1.5 classic)
    /// - `depth_ms`: Modulation depth in milliseconds (1-5 typical)
    /// - `mix`: Dry/wet blend (0.0 = dry, 1.0 = wet, 0.3-0.5 typical)
    pub fn new(rate: f32, depth_ms: f32, mix: f32) -> Self {
        Self {
            delay_line: DelayLine::new(),
            lfo_phase: 0.0,
            rate: rate.clamp(0.1, 10.0),
            depth_ms: depth_ms.clamp(0.5, 10.0),
            mix: mix.clamp(0.0, 1.0),
            base_delay_ms: 20.0, // Classic chorus base delay
        }
    }

    /// Set the base delay time (default 20ms).
    pub fn with_base_delay(mut self, ms: f32) -> Self {
        self.base_delay_ms = ms.clamp(5.0, 50.0);
        self
    }
}

impl GraphNode for ChorusNode {
    fn render_block(&mut self, out: &mut [f32], ctx: &RenderCtx) {
        let sample_rate = ctx.sample_rate;
        let phase_inc = TAU * self.rate / sample_rate;

        for sample in out.iter_mut() {
            // Calculate modulated delay time
            let lfo_value = self.lfo_phase.sin(); // -1 to +1
            let delay_ms = self.base_delay_ms + lfo_value * self.depth_ms;
            let delay_samples = (delay_ms * sample_rate / 1000.0).max(1.0);

            // Get delayed sample (interpolated for smooth modulation)
            let delayed = self.delay_line.read_interpolated(delay_samples);

            // Write current sample to delay line
            self.delay_line.write(*sample);

            // Mix dry and wet
            let dry = *sample;
            let wet = delayed;
            *sample = dry * (1.0 - self.mix) + wet * self.mix;

            // Advance LFO phase
            self.lfo_phase += phase_inc;
            if self.lfo_phase >= TAU {
                self.lfo_phase -= TAU;
            }
        }
    }

    fn note_on(&mut self, _ctx: &RenderCtx) {
        // Optionally reset delay line for clean attack
        // self.delay_line.reset();
        // Note: Usually we don't reset chorus on note-on to maintain continuity
    }
}

impl Modulatable for ChorusNode {
    type Param = ChorusParam;

    fn get_param(&self, param: Self::Param) -> f32 {
        match param {
            ChorusParam::Rate => self.rate,
            ChorusParam::Depth => self.depth_ms,
            ChorusParam::Mix => self.mix,
        }
    }

    fn apply_modulation(&mut self, param: Self::Param, base: f32, modulation: f32) {
        match param {
            ChorusParam::Rate => {
                self.rate = (base + modulation).clamp(0.1, 10.0);
            }
            ChorusParam::Depth => {
                self.depth_ms = (base + modulation).clamp(0.5, 10.0);
            }
            ChorusParam::Mix => {
                self.mix = (base + modulation).clamp(0.0, 1.0);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_ctx() -> RenderCtx {
        RenderCtx::from_note(48000.0, 60, 100.0)
    }

    #[test]
    fn test_chorus_modifies_signal() {
        let mut node = ChorusNode::new(1.0, 3.0, 0.5);
        let mut buffer = vec![0.5; 256];
        let original_sum: f32 = buffer.iter().sum();

        node.render_block(&mut buffer, &test_ctx());

        // After chorus, the signal should be different
        let new_sum: f32 = buffer.iter().sum();
        // Due to delay and mixing, values change
        assert!((original_sum - new_sum).abs() > 0.01 || buffer.iter().any(|&x| x != 0.5));
    }

    #[test]
    fn test_dry_chorus_preserves_signal() {
        let mut node = ChorusNode::new(1.0, 3.0, 0.0); // 100% dry
        // Need to prime the delay line first
        let mut warmup = vec![0.0; 1024];
        node.render_block(&mut warmup, &test_ctx());

        let mut buffer = vec![0.3; 64];
        let original = buffer.clone();

        node.render_block(&mut buffer, &test_ctx());

        // At 0% mix, output should be mostly the input
        for (a, b) in buffer.iter().zip(original.iter()) {
            assert!((a - b).abs() < 0.01);
        }
    }

    #[test]
    fn test_chorus_output_bounded() {
        let mut node = ChorusNode::new(2.0, 5.0, 0.5);
        let mut buffer: Vec<f32> = (0..256).map(|i| (i as f32 * 0.1).sin()).collect();

        node.render_block(&mut buffer, &test_ctx());

        // Output should stay reasonable
        for sample in &buffer {
            assert!(sample.abs() < 2.0);
        }
    }
}
