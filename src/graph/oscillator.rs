use crate::dsp::oscillator::OscillatorBlock;
use crate::graph::node::{GraphNode, RenderCtx};

/*
Audio Oscillator
================

An oscillator is the fundamental sound source in a synthesizer. It generates
a repeating waveform at a specific frequency (pitch), producing the raw
audio material that gets shaped by filters, envelopes, and effects.

Waveform Types and Their Character:
-----------------------------------

Sine: The purest tone - a single frequency with no harmonics.
  - Sound: Smooth, hollow, flute-like
  - Harmonics: Fundamental only (no overtones)
  - Use: Sub-bass, pure tones, FM synthesis carrier

Sawtooth: The richest waveform - contains all harmonics.
  - Sound: Bright, buzzy, brassy
  - Harmonics: All harmonics (1st, 2nd, 3rd, 4th, ...)
             Amplitude falls off as 1/n
  - Use: Leads, basses, pads, brass sounds

Square: Hollow but powerful - only odd harmonics.
  - Sound: Hollow, woody, clarinet-like
  - Harmonics: Odd harmonics only (1st, 3rd, 5th, 7th, ...)
             Amplitude falls off as 1/n
  - Use: Sub-bass (filtered), chiptune, hollow leads

Triangle: Mellow and soft - weak odd harmonics.
  - Sound: Soft, flute-like, between sine and square
  - Harmonics: Odd harmonics only, but fall off as 1/n²
             (much quieter overtones than square)
  - Use: Soft leads, sub-bass, gentle pads

What are Harmonics?
-------------------
When you play a note at 440 Hz (A4), the harmonics are:
  1st (fundamental): 440 Hz  - the pitch you hear
  2nd harmonic:      880 Hz  - one octave up
  3rd harmonic:      1320 Hz - octave + fifth
  4th harmonic:      1760 Hz - two octaves up
  ... and so on

The mix of these harmonics gives each waveform its unique timbre (tonal color).
A filter can then remove some harmonics to sculpt the sound.

Example usage:
  let osc = OscNode::sine();      // Pure tone
  let osc = OscNode::sawtooth();  // Rich and bright
  let osc = OscNode::square();    // Hollow and punchy
  let osc = OscNode::triangle();  // Soft and mellow

  // Typical subtractive synth: saw → filter → envelope
  let voice = OscNode::sawtooth()
      .through(FilterNode::lowpass(2000.0))
      .amplify(EnvNode::adsr(0.01, 0.1, 0.7, 0.3));
*/

pub struct OscNode {
    osc: OscillatorBlock,
}

impl OscNode {
    pub fn sine() -> Self {
        let osc = OscillatorBlock::sine();
        Self { osc }
    }

    pub fn sawtooth() -> Self {
        let osc = OscillatorBlock::sawtooth();
        Self { osc }
    }

    pub fn square() -> Self {
        let osc = OscillatorBlock::square();
        Self { osc }
    }

    pub fn triangle() -> Self {
        let osc = OscillatorBlock::triangle();
        Self { osc }
    }
}

impl GraphNode for OscNode {
    fn render_block(&mut self, out: &mut [f32], ctx: &RenderCtx) {
        self.osc.render(out, ctx);
    }
}

#[cfg(test)]
mod tests {
    use crate::graph::node::RenderCtx;

    use super::*;
    use std::f32::consts::TAU;

    #[test]
    fn valid_sine() {
        let sample_rate = 48_000.0;
        let block_size = 128;
        let note = 69; // MIDI note 69 = A4 = 440Hz

        let ctx = RenderCtx::from_note(sample_rate, note, 100.0);
        let mut synth = OscNode::sine();

        let mut buffer = vec![0.0f32; block_size];
        synth.render_block(&mut buffer, &ctx);

        // sample n should be sin(2pi f n / sr), where f = 440Hz (MIDI 69)
        let sample_index = 12;
        let expected = (TAU * ctx.frequency * sample_index as f32 / sample_rate).sin();
        let actual = buffer[sample_index];
        assert!(
            (actual - expected).abs() < 1e-6,
            "expected {expected}, got {actual}"
        );
    }
}
