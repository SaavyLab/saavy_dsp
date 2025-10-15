use crate::{dsp::delay::DelayLine, graph::node::{GraphNode, Modulatable}};

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
