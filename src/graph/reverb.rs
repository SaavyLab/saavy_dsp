use crate::dsp::reverb::SchroederReverb;
use crate::graph::node::{GraphNode, Modulatable, RenderCtx};

/*
Reverb Node
===========

Simulates the acoustic reflections of a physical space. Reverb adds depth,
dimension, and "glue" to sounds, making them feel like they exist in a room.

How Reverb Works
----------------

When sound bounces off walls, floor, and ceiling, you hear:
1. Direct sound (original signal)
2. Early reflections (first few bounces, give sense of room size)
3. Reverb tail (dense wash of many reflections, decays over time)

This implementation uses a Schroeder reverb algorithm:
- 4 parallel comb filters create the reverb tail
- 2 series allpass filters add density and diffusion

Parameters
----------

Room Size (0.0 - 1.0):
  How large the virtual space feels.
  0.0 = small room, 1.0 = large hall

Damping (0.0 - 1.0):
  High-frequency absorption. Higher values = darker, more natural decay.
  0.0 = bright, metallic    1.0 = dark, muffled

Mix (0.0 - 1.0):
  Dry/wet blend.
  0.0 = all dry, 0.3 = subtle, 0.5 = obvious, 1.0 = all wet

Example usage:

  // Subtle room reverb on drums
  let drums = voices::snare()
      .through(ReverbNode::new(0.3, 0.4, 0.2));

  // Lush hall reverb on pad
  let pad = voices::pad()
      .through(ReverbNode::new(0.8, 0.5, 0.4));

  // Huge ambient reverb
  let ambient = OscNode::sine()
      .through(ReverbNode::new(1.0, 0.3, 0.7));
*/

/// Parameters that can be modulated
#[derive(Clone, Copy, Debug)]
pub enum ReverbParam {
    /// Room size (0.0 = small, 1.0 = large)
    RoomSize,
    /// High-frequency damping (0.0 = bright, 1.0 = dark)
    Damping,
    /// Dry/wet mix (0.0 = dry, 1.0 = wet)
    Mix,
}

/// Schroeder reverb effect
pub struct ReverbNode {
    reverb: SchroederReverb,
    room_size: f32,
    damping: f32,
    mix: f32,
    initialized: bool,
}

impl ReverbNode {
    /// Create a new reverb effect.
    ///
    /// - `room_size`: 0.0 (small room) to 1.0 (large hall)
    /// - `damping`: 0.0 (bright) to 1.0 (dark/muffled)
    /// - `mix`: 0.0 (dry) to 1.0 (wet)
    pub fn new(room_size: f32, damping: f32, mix: f32) -> Self {
        // Create with default sample rate, will be updated on first render
        let mut reverb = SchroederReverb::new(48000.0);
        reverb.set_room_size(room_size);
        reverb.set_damping(damping);

        Self {
            reverb,
            room_size: room_size.clamp(0.0, 1.0),
            damping: damping.clamp(0.0, 1.0),
            mix: mix.clamp(0.0, 1.0),
            initialized: false,
        }
    }

    /// Create a small room reverb (short, tight)
    pub fn room(mix: f32) -> Self {
        Self::new(0.3, 0.5, mix)
    }

    /// Create a medium hall reverb (balanced)
    pub fn hall(mix: f32) -> Self {
        Self::new(0.6, 0.4, mix)
    }

    /// Create a large plate reverb (long, smooth)
    pub fn plate(mix: f32) -> Self {
        Self::new(0.85, 0.3, mix)
    }
}

impl GraphNode for ReverbNode {
    fn render_block(&mut self, out: &mut [f32], ctx: &RenderCtx) {
        // Initialize reverb with correct sample rate on first render
        if !self.initialized {
            self.reverb = SchroederReverb::new(ctx.sample_rate);
            self.reverb.set_room_size(self.room_size);
            self.reverb.set_damping(self.damping);
            self.initialized = true;
        }

        for sample in out.iter_mut() {
            let dry = *sample;
            let wet = self.reverb.process(dry);
            *sample = dry * (1.0 - self.mix) + wet * self.mix;
        }
    }

    fn note_on(&mut self, _ctx: &RenderCtx) {
        // Don't reset reverb on note-on - we want the tail to continue
    }
}

impl Modulatable for ReverbNode {
    type Param = ReverbParam;

    fn get_param(&self, param: Self::Param) -> f32 {
        match param {
            ReverbParam::RoomSize => self.room_size,
            ReverbParam::Damping => self.damping,
            ReverbParam::Mix => self.mix,
        }
    }

    fn apply_modulation(&mut self, param: Self::Param, base: f32, modulation: f32) {
        match param {
            ReverbParam::RoomSize => {
                self.room_size = (base + modulation).clamp(0.0, 1.0);
                self.reverb.set_room_size(self.room_size);
            }
            ReverbParam::Damping => {
                self.damping = (base + modulation).clamp(0.0, 1.0);
                self.reverb.set_damping(self.damping);
            }
            ReverbParam::Mix => {
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
    fn test_reverb_adds_tail() {
        let mut reverb = ReverbNode::new(0.5, 0.5, 1.0); // 100% wet

        // Process an impulse
        let mut buffer = vec![1.0; 1];
        reverb.render_block(&mut buffer, &test_ctx());

        // Process silence and check for reverb tail
        let mut tail_energy = 0.0;
        for _ in 0..100 {
            let mut buf = vec![0.0; 64];
            reverb.render_block(&mut buf, &test_ctx());
            tail_energy += buf.iter().map(|x| x * x).sum::<f32>();
        }

        assert!(tail_energy > 0.01, "Reverb should produce a tail");
    }

    #[test]
    fn test_dry_reverb_preserves_signal() {
        let mut reverb = ReverbNode::new(0.5, 0.5, 0.0); // 100% dry

        let mut buffer = vec![0.5, 0.3, 0.7];
        let original = buffer.clone();

        reverb.render_block(&mut buffer, &test_ctx());

        for (a, b) in buffer.iter().zip(original.iter()) {
            assert!((a - b).abs() < 0.01, "Dry reverb should preserve signal");
        }
    }

    #[test]
    fn test_reverb_presets() {
        let room = ReverbNode::room(0.3);
        let hall = ReverbNode::hall(0.3);
        let plate = ReverbNode::plate(0.3);

        assert!(room.room_size < hall.room_size);
        assert!(hall.room_size < plate.room_size);
    }
}
