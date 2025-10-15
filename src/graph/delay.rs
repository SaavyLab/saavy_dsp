use crate::{
    dsp::delay::DelayLine,
    graph::node::{GraphNode, Modulatable},
};

/*
Delay Effect
============

A delay effect stores audio in a buffer and plays it back after a specified time.
Combined with feedback and mixing, this creates echo, slapback, and other time-based effects.

Parameters:
- delay_ms: Delay time in milliseconds (e.g., 500ms for half-second delay)
- feedback: How much delayed signal feeds back into itself (0.0-0.95)
  - 0.0 = single echo
  - 0.5 = multiple fading echoes
  - 0.95 = long sustaining echoes (careful - too high causes runaway!)
- mix: Dry/wet balance (0.0-1.0)
  - 0.0 = dry only (no delay)
  - 0.5 = equal mix
  - 1.0 = wet only (delayed signal only)

How it works:
1. Store incoming audio in a circular buffer
2. Read from buffer at (delay_time) ago
3. Mix delayed signal back into input (feedback)
4. Blend dry and wet signals (mix)

Example:
  let delay = DelayNode::new(250.0, 0.4, 0.3);
  // 250ms delay, 40% feedback (few echoes), 30% wet mix
*/

pub struct DelayNode {
    delay_line: DelayLine,
    delay_ms: f32,
    feedback: f32, // 0.0 - 0.95 (amount of delayed signal fed back)
    mix: f32,      // 0.0 - 1.0 (dry/wet balance)
    // For smooth, click-free modulation we ramp delay time across the block.
    prev_delay_samples: f32,
    first_block: bool,
}

impl DelayNode {
    /// Create a delay effect
    ///
    /// # Arguments
    /// * `delay_ms` - Delay time in milliseconds
    /// * `feedback` - Feedback amount (0.0-0.95, higher = more echoes)
    /// * `mix` - Dry/wet mix (0.0 = dry only, 1.0 = wet only)
    pub fn new(delay_ms: f32, feedback: f32, mix: f32) -> Self {
        Self {
            delay_line: DelayLine::new(),
            delay_ms,
            feedback: feedback.clamp(0.0, 0.95), // Prevent runaway
            mix: mix.clamp(0.0, 1.0),
            prev_delay_samples: 0.0,
            first_block: true,
        }
    }
}

impl GraphNode for DelayNode {
    fn render_block(&mut self, out: &mut [f32], ctx: &super::node::RenderCtx) {
        // Convert delay time from milliseconds to samples (as float for interpolation)
        let target_delay_samples = (self.delay_ms / 1000.0) * ctx.sample_rate;

        // Initialize ramp start on first block to avoid a big jump from 0
        if self.first_block {
            self.prev_delay_samples = target_delay_samples;
            self.first_block = false;
        }

        let len = out.len() as f32;
        let step = if len > 0.0 {
            (target_delay_samples - self.prev_delay_samples) / len
        } else {
            0.0
        };

        let mut delay_s = self.prev_delay_samples;

        for sample in out.iter_mut() {
            let dry = *sample;

            // Read with per-sample fractional delay for smooth modulation
            let wet = self.delay_line.read_interpolated(delay_s);

            // Feedback: write dry + (wet * feedback)
            let input_with_feedback = dry + (wet * self.feedback);
            self.delay_line.write(input_with_feedback);

            // Mix
            *sample = dry * (1.0 - self.mix) + wet * self.mix;

            // Advance delay towards target across the block
            delay_s += step;
        }

        // Store end-of-block delay for continuity next block
        self.prev_delay_samples = target_delay_samples;
    }

    fn note_on(&mut self, _ctx: &super::node::RenderCtx) {
        // Clear buffer to avoid clicks from previous notes
        self.delay_line.reset();
    }
}

#[derive(Clone, Copy, Debug)]
pub enum DelayParam {
    DelayTime,
    Feedback,
    Mix,
}

impl Modulatable for DelayNode {
    type Param = DelayParam;

    fn get_param(&self, param: Self::Param) -> f32 {
        match param {
            DelayParam::DelayTime => self.delay_ms,
            DelayParam::Feedback => self.feedback,
            DelayParam::Mix => self.mix,
        }
    }

    fn apply_modulation(&mut self, param: Self::Param, base: f32, modulation: f32) {
        match param {
            DelayParam::DelayTime => {
                self.delay_ms = (base + modulation).clamp(0.1, 2000.0);
            }
            DelayParam::Feedback => {
                self.feedback = (base + modulation).clamp(0.0, 0.95);
            }
            DelayParam::Mix => {
                self.mix = (base + modulation).clamp(0.0, 1.0);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::node::RenderCtx;

    #[test]
    fn test_delay_node_basic() {
        let mut delay = DelayNode::new(10.0, 0.0, 1.0); // 10ms, no feedback, wet only
        let sample_rate = 48_000.0;
        let ctx = RenderCtx::from_freq(sample_rate, 440.0, 1.0);

        // 10ms at 48kHz = 480 samples delay
        let mut buffer = vec![0.0; 1000];
        buffer[0] = 1.0; // Impulse

        delay.render_block(&mut buffer, &ctx);

        // Should see impulse delayed by ~480 samples
        assert!(buffer[0].abs() < 0.1, "First sample should be mostly dry (but wet only, so ~0)");

        // Peak should be around sample 480
        let peak_pos = buffer.iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.abs().partial_cmp(&b.abs()).unwrap())
            .map(|(i, _)| i)
            .unwrap();

        assert!((peak_pos as i32 - 480).abs() < 50,
                "Peak should be near 480 samples, got {}", peak_pos);
    }

    #[test]
    fn test_delay_node_feedback() {
        let mut delay = DelayNode::new(5.0, 0.5, 1.0); // 5ms, 50% feedback
        let sample_rate = 48_000.0;
        let ctx = RenderCtx::from_freq(sample_rate, 440.0, 1.0);

        let mut buffer = vec![0.0; 2000];
        buffer[0] = 1.0; // Impulse

        delay.render_block(&mut buffer, &ctx);

        // With feedback, we should see multiple echoes
        // Count peaks above threshold
        let peaks: Vec<_> = buffer.iter()
            .enumerate()
            .filter(|(_, &v)| v > 0.2)
            .collect();

        assert!(peaks.len() > 1, "Feedback should create multiple echoes, got {} peaks", peaks.len());
    }

    #[test]
    fn test_delay_node_dry_wet_mix() {
        let sample_rate = 48_000.0;
        let ctx = RenderCtx::from_freq(sample_rate, 440.0, 1.0);

        // Dry only (mix = 0.0)
        let mut delay_dry = DelayNode::new(10.0, 0.0, 0.0);
        let mut buffer_dry = vec![1.0; 100];
        delay_dry.render_block(&mut buffer_dry, &ctx);
        assert!((buffer_dry[0] - 1.0).abs() < 0.01, "Dry only should pass signal unchanged");

        // Wet only (mix = 1.0)
        let mut delay_wet = DelayNode::new(10.0, 0.0, 1.0);
        let mut buffer_wet = vec![1.0; 100];
        delay_wet.render_block(&mut buffer_wet, &ctx);
        assert!(buffer_wet[0].abs() < 0.1, "Wet only should be delayed (near zero initially)");

        // 50/50 mix
        let mut delay_mix = DelayNode::new(10.0, 0.0, 0.5);
        let mut buffer_mix = vec![1.0; 100];
        delay_mix.render_block(&mut buffer_mix, &ctx);
        assert!(buffer_mix[0] > 0.4 && buffer_mix[0] < 0.6,
                "50/50 mix should blend dry and wet");
    }

    #[test]
    fn test_delay_node_modulatable() {
        let mut delay = DelayNode::new(100.0, 0.3, 0.5);

        // Test get_param
        assert!((delay.get_param(DelayParam::DelayTime) - 100.0).abs() < 0.1);
        assert!((delay.get_param(DelayParam::Feedback) - 0.3).abs() < 0.01);
        assert!((delay.get_param(DelayParam::Mix) - 0.5).abs() < 0.01);

        // Test apply_modulation
        delay.apply_modulation(DelayParam::DelayTime, 100.0, 50.0);
        assert!((delay.delay_ms - 150.0).abs() < 0.1);

        delay.apply_modulation(DelayParam::Feedback, 0.3, 0.2);
        assert!((delay.feedback - 0.5).abs() < 0.01);

        delay.apply_modulation(DelayParam::Mix, 0.5, 0.3);
        assert!((delay.mix - 0.8).abs() < 0.01);
    }

    #[test]
    fn test_delay_node_clamping() {
        let mut delay = DelayNode::new(100.0, 0.0, 0.0);

        // Test feedback clamping (should not exceed 0.95)
        delay.apply_modulation(DelayParam::Feedback, 0.5, 1.0);
        assert!(delay.feedback <= 0.95, "Feedback should clamp to 0.95");

        // Test mix clamping
        delay.apply_modulation(DelayParam::Mix, 0.5, 1.0);
        assert!(delay.mix <= 1.0, "Mix should clamp to 1.0");

        delay.apply_modulation(DelayParam::Mix, 0.5, -1.0);
        assert!(delay.mix >= 0.0, "Mix should clamp to 0.0");

        // Test delay time clamping
        delay.apply_modulation(DelayParam::DelayTime, 100.0, -200.0);
        assert!(delay.delay_ms >= 0.1, "Delay time should clamp to minimum");
    }

    #[test]
    fn test_delay_node_no_runaway() {
        // High feedback should not cause runaway (exploding values)
        let mut delay = DelayNode::new(5.0, 0.9, 1.0);
        let sample_rate = 48_000.0;
        let ctx = RenderCtx::from_freq(sample_rate, 440.0, 1.0);

        let mut buffer = vec![0.0; 5000];
        buffer[0] = 1.0;

        delay.render_block(&mut buffer, &ctx);

        // All samples should be finite and reasonable
        for (i, &sample) in buffer.iter().enumerate() {
            assert!(sample.is_finite(), "Sample {} is not finite", i);
            assert!(sample.abs() < 10.0, "Sample {} is too large: {}", i, sample);
        }
    }

    #[test]
    fn test_delay_node_smoothing() {
        // Test that delay time changes are smoothed across blocks
        let mut delay = DelayNode::new(10.0, 0.0, 1.0);
        let sample_rate = 48_000.0;
        let ctx = RenderCtx::from_freq(sample_rate, 440.0, 1.0);

        // First block
        let mut buffer1 = vec![1.0; 512];
        delay.render_block(&mut buffer1, &ctx);

        // Change delay time and render second block
        delay.delay_ms = 20.0;
        let mut buffer2 = vec![1.0; 512];
        delay.render_block(&mut buffer2, &ctx);

        // Should smoothly transition without clicks
        // (Hard to test directly, but should not panic and produce finite values)
        for &sample in buffer2.iter() {
            assert!(sample.is_finite(), "Smoothing should produce finite values");
        }
    }
}
