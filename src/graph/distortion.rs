use crate::dsp::distortion::{foldback_buffer, hard_clip_buffer, soft_clip_buffer};
use crate::graph::node::{GraphNode, Modulatable, RenderCtx};

/*
Distortion Node
===============

Adds harmonics and grit to a signal by applying waveshaping. Use this to add
warmth, aggression, or character to sounds.

Distortion Modes
----------------

Soft:     Warm, tube-like saturation. Gradually compresses peaks.
          Best for: Warming up basses, subtle saturation on leads

Hard:     Harsh, buzzy clipping. Creates odd harmonics like a fuzz pedal.
          Best for: Aggressive leads, lo-fi drums, guitar-style distortion

Foldback: Complex, metallic character. Signal folds back at threshold.
          Best for: Sound design, synth textures, extreme effects

Parameters
----------

Drive (1.0 - 10.0+):
  How hard the signal is pushed into distortion.
  1.0 = clean, 3-4 = warm, 5-10 = heavy

Mix (0.0 - 1.0):
  Dry/wet blend. 0.0 = all dry, 1.0 = all wet, 0.5 = 50/50

Example usage:

  // Warm bass saturation
  let bass = OscNode::sawtooth()
      .through(DistortionNode::soft(3.0, 0.5))
      .through(FilterNode::lowpass(800.0));

  // Aggressive lead
  let lead = OscNode::square()
      .through(DistortionNode::hard(6.0, 0.8));

  // Weird sound design
  let texture = OscNode::sine()
      .through(DistortionNode::foldback(8.0, 0.7));
*/

/// Type of distortion/waveshaping
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DistortionMode {
    /// Soft clipping - warm, tube-like saturation
    Soft,
    /// Hard clipping - harsh, buzzy distortion
    Hard,
    /// Foldback - complex, metallic harmonics
    Foldback,
}

/// Parameters that can be modulated
#[derive(Clone, Copy, Debug)]
pub enum DistortionParam {
    /// Drive amount (1.0 = clean, higher = more distortion)
    Drive,
    /// Dry/wet mix (0.0 = dry, 1.0 = wet)
    Mix,
}

/// Waveshaping distortion effect
pub struct DistortionNode {
    mode: DistortionMode,
    drive: f32,
    mix: f32,
    threshold: f32, // For hard clip and foldback
}

impl DistortionNode {
    /// Create a soft clipping distortion (warm, tube-like)
    pub fn soft(drive: f32, mix: f32) -> Self {
        Self {
            mode: DistortionMode::Soft,
            drive: drive.max(1.0),
            mix: mix.clamp(0.0, 1.0),
            threshold: 1.0,
        }
    }

    /// Create a hard clipping distortion (harsh, buzzy)
    pub fn hard(drive: f32, mix: f32) -> Self {
        Self {
            mode: DistortionMode::Hard,
            drive: drive.max(1.0),
            mix: mix.clamp(0.0, 1.0),
            threshold: 1.0,
        }
    }

    /// Create a foldback distortion (complex, metallic)
    pub fn foldback(drive: f32, mix: f32) -> Self {
        Self {
            mode: DistortionMode::Foldback,
            drive: drive.max(1.0),
            mix: mix.clamp(0.0, 1.0),
            threshold: 1.0,
        }
    }

    /// Set a custom threshold for hard clip and foldback modes.
    /// Lower threshold = more extreme distortion at same drive.
    pub fn with_threshold(mut self, threshold: f32) -> Self {
        self.threshold = threshold.max(0.1);
        self
    }
}

impl GraphNode for DistortionNode {
    fn render_block(&mut self, out: &mut [f32], _ctx: &RenderCtx) {
        // Store dry signal for mixing
        let dry: Vec<f32> = out.to_vec();

        // Apply distortion based on mode
        match self.mode {
            DistortionMode::Soft => {
                soft_clip_buffer(out, self.drive);
            }
            DistortionMode::Hard => {
                hard_clip_buffer(out, self.drive, self.threshold);
            }
            DistortionMode::Foldback => {
                foldback_buffer(out, self.drive, self.threshold);
            }
        }

        // Mix dry/wet
        if self.mix < 1.0 {
            let dry_amount = 1.0 - self.mix;
            for (wet, dry_sample) in out.iter_mut().zip(dry.iter()) {
                *wet = *wet * self.mix + dry_sample * dry_amount;
            }
        }
    }
}

impl Modulatable for DistortionNode {
    type Param = DistortionParam;

    fn get_param(&self, param: Self::Param) -> f32 {
        match param {
            DistortionParam::Drive => self.drive,
            DistortionParam::Mix => self.mix,
        }
    }

    fn apply_modulation(&mut self, param: Self::Param, base: f32, modulation: f32) {
        match param {
            DistortionParam::Drive => {
                self.drive = (base + modulation).max(1.0);
            }
            DistortionParam::Mix => {
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
    fn test_soft_distortion_modifies_signal() {
        let mut node = DistortionNode::soft(4.0, 1.0);
        let mut buffer = vec![0.5, -0.5, 0.8, -0.8];
        let original = buffer.clone();

        node.render_block(&mut buffer, &test_ctx());

        // Signal should be different (distorted)
        assert!(buffer.iter().zip(original.iter()).any(|(a, b)| (a - b).abs() > 0.01));
    }

    #[test]
    fn test_dry_mix_preserves_signal() {
        let mut node = DistortionNode::soft(4.0, 0.0); // 100% dry
        let mut buffer = vec![0.5, -0.5, 0.3, -0.3];
        let original = buffer.clone();

        node.render_block(&mut buffer, &test_ctx());

        // Signal should be unchanged
        for (a, b) in buffer.iter().zip(original.iter()) {
            assert!((a - b).abs() < 1e-6);
        }
    }

    #[test]
    fn test_hard_clipping_limits_amplitude() {
        let mut node = DistortionNode::hard(5.0, 1.0).with_threshold(0.5);
        let mut buffer = vec![0.5, -0.5, 0.8, -0.8];

        node.render_block(&mut buffer, &test_ctx());

        // All values should be within threshold
        for sample in &buffer {
            assert!(sample.abs() <= 0.5 + 1e-6);
        }
    }
}
