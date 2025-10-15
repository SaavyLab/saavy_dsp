use crate::{
    dsp::oscillator::OscillatorBlock,
    graph::node::{GraphNode, RenderCtx},
};

/*
LFO (Low Frequency Oscillator)
==============================

An LFO is an oscillator that runs at sub-audio frequencies to modulate
parameters over time. Unlike audio oscillators (20 Hz - 20 kHz), LFOs
typically operate between 0.01 Hz - 20 Hz.

Common uses:
- Vibrato:    Modulate pitch (±5-10 cents)
- Tremolo:    Modulate amplitude (volume wobble)
- Auto-wah:   Modulate filter cutoff (sweeping filter)
- Auto-pan:   Modulate stereo position (left-right movement)

Technical differences from audio oscillators:
1. Fixed frequency (ignores MIDI note input)
2. Outputs control signal (-1.0 to 1.0)
3. Used with .modulate() combinator to affect parameters

Example:
  let lfo = LfoNode::sine(5.0);  // 5 Hz sine wave
  let filter = FilterNode::lowpass(1000.0)
      .modulate(lfo, FilterParam::Cutoff, 500.0);
  // Result: Filter cutoff sweeps 500 Hz - 1500 Hz at 5 Hz
*/

pub struct LfoNode {
    osc: OscillatorBlock,
    frequency: f32, // Fixed frequency in Hz (ignores note context)
}

impl LfoNode {
    pub fn sine(frequency: f32) -> Self {
        Self {
            osc: OscillatorBlock::sine(),
            frequency,
        }
    }

    pub fn saw(frequency: f32) -> Self {
        Self {
            osc: OscillatorBlock::sawtooth(),
            frequency,
        }
    }

    pub fn square(frequency: f32) -> Self {
        Self {
            osc: OscillatorBlock::square(),
            frequency,
        }
    }

    pub fn triangle(frequency: f32) -> Self {
        Self {
            osc: OscillatorBlock::triangle(),
            frequency,
        }
    }
}

impl GraphNode for LfoNode {
    fn render_block(&mut self, out: &mut [f32], ctx: &RenderCtx) {
        // Create context with LFO's fixed frequency (not the note frequency)
        // This makes the LFO oscillate independently of the musical pitch
        let lfo_ctx = RenderCtx::from_freq(ctx.sample_rate, self.frequency, 1.0);
        self.osc.render(out, &lfo_ctx);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lfo_sine_output_range() {
        let mut lfo = LfoNode::sine(5.0);
        let mut buffer = vec![0.0; 1024];
        let ctx = RenderCtx::from_freq(48000.0, 440.0, 1.0);

        lfo.render_block(&mut buffer, &ctx);

        for &sample in &buffer {
            assert!(
                sample >= -1.0 && sample <= 1.0,
                "LFO sine sample {} out of range [-1.0, 1.0]",
                sample
            );
        }
    }

    #[test]
    #[ignore] // Triangle waveform not yet implemented
    fn test_lfo_triangle_output_range() {
        let mut lfo = LfoNode::triangle(3.0);
        let mut buffer = vec![0.0; 2048];
        let ctx = RenderCtx::from_freq(48000.0, 440.0, 1.0);

        lfo.render_block(&mut buffer, &ctx);

        for &sample in &buffer {
            assert!(
                sample >= -1.0 && sample <= 1.0,
                "LFO triangle sample {} out of range [-1.0, 1.0]",
                sample
            );
        }
    }

    #[test]
    fn test_lfo_square_output_range() {
        let mut lfo = LfoNode::square(10.0);
        let mut buffer = vec![0.0; 512];
        let ctx = RenderCtx::from_freq(48000.0, 440.0, 1.0);

        lfo.render_block(&mut buffer, &ctx);

        for &sample in &buffer {
            assert!(
                sample >= -1.0 && sample <= 1.0,
                "LFO square sample {} out of range [-1.0, 1.0]",
                sample
            );
        }
    }

    #[test]
    fn test_lfo_saw_output_range() {
        let mut lfo = LfoNode::saw(7.0);
        let mut buffer = vec![0.0; 1024];
        let ctx = RenderCtx::from_freq(48000.0, 440.0, 1.0);

        lfo.render_block(&mut buffer, &ctx);

        for &sample in &buffer {
            assert!(
                sample >= -1.0 && sample <= 1.0,
                "LFO saw sample {} out of range [-1.0, 1.0]",
                sample
            );
        }
    }

    #[test]
    fn test_lfo_ignores_note_frequency() {
        // LFO should use its own frequency, not the context frequency
        let mut lfo = LfoNode::sine(5.0);
        let mut buffer1 = vec![0.0; 512];
        let mut buffer2 = vec![0.0; 512];

        // Render with two different note frequencies
        let ctx1 = RenderCtx::from_freq(48000.0, 440.0, 1.0);
        let ctx2 = RenderCtx::from_freq(48000.0, 880.0, 1.0);

        lfo.render_block(&mut buffer1, &ctx1);

        // Reset phase by creating new LFO
        let mut lfo = LfoNode::sine(5.0);
        lfo.render_block(&mut buffer2, &ctx2);

        // Buffers should be identical (LFO ignores note frequency)
        for (i, (&s1, &s2)) in buffer1.iter().zip(&buffer2).enumerate() {
            assert!(
                (s1 - s2).abs() < 1e-6,
                "LFO output differs at sample {}: {} vs {}",
                i,
                s1,
                s2
            );
        }
    }
}
