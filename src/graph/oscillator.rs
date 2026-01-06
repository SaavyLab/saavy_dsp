use crate::dsp::oscillator::OscillatorBlock;
use crate::graph::node::{GraphNode, Modulatable, RenderCtx};

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

Noise: Random samples - no pitch.
  - Sound: Hiss, static, breath
  - Harmonics: All frequencies equally (white noise)
  - Use: Percussion (snare, hi-hat), wind, texture

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
  let osc = OscNode::noise();     // Random (percussion)

  // Typical subtractive synth: saw → filter → envelope
  let voice = OscNode::sawtooth()
      .through(FilterNode::lowpass(2000.0))
      .amplify(EnvNode::adsr(0.01, 0.1, 0.7, 0.3));
*/

pub struct OscNode {
    osc: OscillatorBlock,
    /// Fixed frequency (Hz). If Some, ignores ctx.frequency and uses this instead.
    /// Used for drums and other sounds that shouldn't track note pitch.
    base_frequency: Option<f32>,
    /// Current frequency after modulation (only used when base_frequency is Some)
    current_frequency: f32,
    /// Detune in cents (-100 to +100 typical). 100 cents = 1 semitone.
    /// Used for supersaw, reese bass, and other "thick" sounds.
    detune_cents: f32,
}

/// Parameters that can be modulated on an oscillator
#[derive(Clone, Copy, Debug)]
pub enum OscParam {
    /// Oscillator frequency in Hz
    Frequency,
    /// Detune in cents (100 cents = 1 semitone)
    Detune,
}

impl OscNode {
    fn new(osc: OscillatorBlock) -> Self {
        Self {
            osc,
            base_frequency: None,
            current_frequency: 440.0,
            detune_cents: 0.0,
        }
    }

    pub fn sine() -> Self {
        Self::new(OscillatorBlock::sine())
    }

    pub fn sawtooth() -> Self {
        Self::new(OscillatorBlock::sawtooth())
    }

    pub fn square() -> Self {
        Self::new(OscillatorBlock::square())
    }

    pub fn triangle() -> Self {
        Self::new(OscillatorBlock::triangle())
    }

    pub fn noise() -> Self {
        Self::new(OscillatorBlock::noise())
    }

    /// Set a fixed frequency, ignoring the note pitch from RenderCtx.
    ///
    /// Use this for drums and other sounds that shouldn't track keyboard pitch.
    /// The frequency can then be modulated with `.modulate()`.
    ///
    /// # Example
    /// ```ignore
    /// // Kick drum with pitch envelope: 150Hz -> 50Hz
    /// OscNode::sine()
    ///     .with_frequency(50.0)  // Base frequency (what it settles to)
    ///     .modulate(EnvNode::adsr(0.001, 0.08, 0.0, 0.0), OscParam::Frequency, 100.0)
    /// ```
    pub fn with_frequency(mut self, freq: f32) -> Self {
        self.base_frequency = Some(freq);
        self.current_frequency = freq;
        self
    }

    /// Set detune in cents (100 cents = 1 semitone).
    ///
    /// Use this to create "thick" sounds by layering detuned oscillators.
    /// Typical values: ±5-15 cents for subtle width, ±20-50 for obvious detune.
    ///
    /// # Example
    /// ```ignore
    /// // Reese bass: two saws slightly detuned
    /// OscNode::sawtooth()
    ///     .mix(OscNode::sawtooth().with_detune(12.0), 0.5)
    ///     .through(FilterNode::lowpass(300.0))
    ///
    /// // Supersaw: multiple detuned saws
    /// OscNode::sawtooth()
    ///     .mix(OscNode::sawtooth().with_detune(-15.0), 0.33)
    ///     .mix(OscNode::sawtooth().with_detune(15.0), 0.33)
    /// ```
    pub fn with_detune(mut self, cents: f32) -> Self {
        self.detune_cents = cents;
        self
    }
}

impl GraphNode for OscNode {
    fn render_block(&mut self, out: &mut [f32], ctx: &RenderCtx) {
        // Determine base frequency: fixed or from note
        let base_freq = if self.base_frequency.is_some() {
            self.current_frequency
        } else {
            ctx.frequency
        };

        // Apply detune: frequency * 2^(cents/1200)
        let final_freq = if self.detune_cents != 0.0 {
            base_freq * 2.0_f32.powf(self.detune_cents / 1200.0)
        } else {
            base_freq
        };

        let modified_ctx = RenderCtx {
            frequency: final_freq,
            ..*ctx
        };
        self.osc.render(out, &modified_ctx);
    }

    fn note_on(&mut self, _ctx: &RenderCtx) {
        // Reset current_frequency to base on note-on (important for modulation)
        if let Some(base) = self.base_frequency {
            self.current_frequency = base;
        }
    }
}

impl Modulatable for OscNode {
    type Param = OscParam;

    fn get_param(&self, param: Self::Param) -> f32 {
        match param {
            OscParam::Frequency => self.base_frequency.unwrap_or(440.0),
            OscParam::Detune => self.detune_cents,
        }
    }

    fn apply_modulation(&mut self, param: Self::Param, base: f32, modulation: f32) {
        match param {
            OscParam::Frequency => {
                // Clamp to audible range (20 Hz - 20 kHz)
                self.current_frequency = (base + modulation).clamp(20.0, 20_000.0);
            }
            OscParam::Detune => {
                // Clamp to reasonable range (±2 semitones)
                self.detune_cents = (base + modulation).clamp(-200.0, 200.0);
            }
        }
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
