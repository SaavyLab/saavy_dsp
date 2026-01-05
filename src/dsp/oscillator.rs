#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::f32::consts::TAU;

use crate::graph::node::RenderCtx;

/*
Oscillator Implementation
=========================

An oscillator generates a repeating waveform - the raw material of synthesis.
It's like a wheel spinning: as it rotates, we trace out a shape (the waveform).

Vocabulary
----------

  phase       Angular position in the cycle, measured in radians (0 to τ).
              Think of it as "where we are" on a spinning wheel.

  φ (phi)     Normalized phase (0.0 to 1.0). Same concept as phase, but scaled
              so one complete cycle = 1.0 instead of τ. Easier for waveform math.

              φ = phase / τ

  τ (tau)     One complete rotation in radians = 2π ≈ 6.283. We use TAU because
              "one cycle = TAU radians" is cleaner than "one cycle = 2*PI".

  frequency   Cycles per second, measured in Hz. A4 = 440 Hz means 440 complete
              waveform cycles every second.

  period      Duration of one cycle = 1/frequency. At 440 Hz, period ≈ 2.27ms.

  sample_rate Samples per second (e.g., 48000). How many times per second we
              ask "what's the current output value?"

  phase_inc   How much phase advances per sample. The key formula:

              phase_inc = τ × frequency / sample_rate

  waveform    The shape traced during one cycle. Different shapes = different
              timbres (tonal colors).

  duty_cycle  For square waves: fraction of the cycle spent "high" (0.0-1.0).
              duty = 0.5 gives a symmetric square wave.


The Core Loop
-------------

Every sample, we:
  1. Read the current waveform value at our phase position
  2. Advance phase by phase_inc
  3. Wrap phase back to 0 when it exceeds τ (completed a cycle)

    ┌─────────────────────────────────────────────────────┐
    │  sample = waveform(phase)                           │
    │  phase = (phase + phase_inc) mod τ                  │
    └─────────────────────────────────────────────────────┘


The Math: Frequency to Phase Increment
--------------------------------------

How much should phase advance each sample?

  phase_inc = τ × frequency / sample_rate

Example: 440 Hz at 48 kHz sample rate
  phase_inc = 6.283 × 440 / 48000
            = 2764.6 / 48000
            ≈ 0.0576 radians per sample

After 48000 samples (1 second), total phase advance:
  0.0576 × 48000 = 2764.6 radians = 440 × τ = 440 cycles ✓


Waveform Shapes
---------------

All waveforms output values in the range [-1.0, +1.0].

SINE: The "pure" tone. Only contains the fundamental frequency.

    +1 │    ╭───╮
       │   ╱     ╲
     0 │──╱───────╲──────
       │ ╱         ╲    ╱
    -1 │╱           ╰──╯
       └─────────────────→ φ
        0    0.5    1.0

    Formula: sin(phase)  // phase is in radians, so sin() works directly


SAWTOOTH: Bright, buzzy. Contains all harmonics (1st, 2nd, 3rd...).

    +1 │      ╱│      ╱│
       │    ╱  │    ╱  │
     0 │──╱────│──╱────│
       │╱      │╱      │
    -1 │       │       │
       └─────────────────→ φ
        0    0.5    1.0

    Formula: (2 × φ) - 1
    At φ=0: (2×0)-1 = -1
    At φ=1: (2×1)-1 = +1 (then wraps back to -1)


SQUARE: Hollow, woody. Contains only odd harmonics (1st, 3rd, 5th...).

    +1 │ ████      ████
       │ █  █      █  █
     0 │─█──█──────█──█─
       │    █      █
    -1 │    ██████ █
       └─────────────────→ φ
        0   0.5   1.0

    Formula: if φ < duty { +1 } else { -1 }
    With duty=0.5, output is +1 for first half, -1 for second half.


TRIANGLE: Mellow, flute-like. Contains only odd harmonics, but weaker.

    +1 │    ╱╲
       │   ╱  ╲
     0 │──╱────╲────╱──
       │ ╱      ╲  ╱
    -1 │╱        ╲╱
       └─────────────────→ φ
        0   0.5   1.0

    Formula: 2 × |sawtooth| - 1
    Take absolute value of sawtooth, scale to [-1, +1].


NOISE: Random values. No pitch, used for percussion/texture.

    +1 │ █ █   █ █  █
       │█ █ █ █ █ ██ █
     0 │──────────────
       │ █ ██ █  █ █ █
    -1 │  █    █    █
       └─────────────────→ time

    No phase relationship - each sample is pseudo-random.


Phase Wrapping
--------------

When phase exceeds τ, we wrap it back: phase = phase mod τ

We use rem_euclid() instead of % because it handles negative numbers correctly
(though phase should never go negative in normal operation).


Why Radians?
------------

We store phase in radians because sin() expects radians. Converting to/from
normalized φ is cheap (multiply/divide by τ), and keeping phase in radians
avoids a conversion every sample for sine waves.

Alternative: Store normalized φ and convert for sin(). Some implementations
do this. Both approaches work; we chose radians for sine efficiency.
*/

/// The shape of the waveform. Each has a distinct timbre (tonal color).
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy)]
pub enum Waveform {
    Sine,     // Pure tone, no harmonics
    Sawtooth, // Bright, buzzy - all harmonics
    Square,   // Hollow, woody - odd harmonics only
    Triangle, // Mellow, flute-like - weak odd harmonics
    Noise,    // Random - no pitch, for percussion/texture
}

pub struct OscillatorBlock {
    phase: f32,        // Current position in cycle (0 to τ radians)
    waveform: Waveform,
    duty_cycle: f32,   // For square wave: fraction spent "high" (0.0-1.0)
    noise_state: u32,  // PRNG state for noise waveform
}

/// Initial seed for noise PRNG - a common choice from numerical recipes
const NOISE_SEED: u32 = 0x9E3779B9;

impl OscillatorBlock {
    pub fn new(waveform: Waveform) -> Self {
        Self {
            phase: 0.0,
            waveform,
            duty_cycle: 0.5,
            noise_state: NOISE_SEED,
        }
    }

    pub fn sine() -> Self {
        Self::new(Waveform::Sine)
    }

    pub fn sawtooth() -> Self {
        Self::new(Waveform::Sawtooth)
    }

    pub fn square() -> Self {
        Self::new(Waveform::Square)
    }

    pub fn triangle() -> Self {
        Self::new(Waveform::Triangle)
    }

    pub fn noise() -> Self {
        Self::new(Waveform::Noise)
    }

    /// Compute the waveform value at the current phase position.
    /// Returns a value in [-1.0, +1.0].
    pub fn next_sample(&mut self) -> f32 {
        match self.waveform {
            Waveform::Sine => {
                // sin() expects radians, and phase is already in radians
                self.phase.sin()
            }

            Waveform::Sawtooth => {
                // Convert to normalized phase φ ∈ [0, 1)
                let phi = self.phase / TAU;
                // Linear ramp: φ=0 → -1, φ=1 → +1
                (2.0 * phi) - 1.0
            }

            Waveform::Square => {
                // Convert to normalized phase φ ∈ [0, 1)
                let phi = self.phase / TAU;
                // Compare to duty cycle threshold
                if phi < self.duty_cycle {
                    1.0
                } else {
                    -1.0
                }
            }

            Waveform::Triangle => {
                // Convert to normalized phase φ ∈ [0, 1)
                let phi = self.phase / TAU;
                // Start with sawtooth: -1 to +1
                let saw = (2.0 * phi) - 1.0;
                // Absolute value creates a V shape (0 to 1 to 0)
                // Scale and shift to get triangle: -1 to +1 to -1
                2.0 * saw.abs() - 1.0
            }

            Waveform::Noise => self.next_noise_sample(),
        }
    }

    /// Render a block of samples into the buffer.
    ///
    /// For each sample:
    ///   1. Write waveform value at current phase
    ///   2. Advance phase by phase_inc
    ///   3. Wrap phase when it exceeds τ
    pub fn render(&mut self, buffer: &mut [f32], ctx: &RenderCtx) {
        if ctx.sample_rate <= 0.0 {
            buffer.fill(0.0);
            return;
        }

        // phase_inc = τ × frequency / sample_rate
        // This is how much phase advances per sample
        let phase_inc = TAU * ctx.frequency / ctx.sample_rate;

        for sample in buffer.iter_mut() {
            *sample = self.next_sample();
            // Advance and wrap phase using rem_euclid (handles negatives correctly)
            self.phase = (self.phase + phase_inc).rem_euclid(TAU);
        }
    }

    /// Generate the next pseudo-random noise sample using xorshift32.
    ///
    /// Xorshift32 is a fast PRNG with good statistical properties:
    ///   x ^= x << 13
    ///   x ^= x >> 17
    ///   x ^= x << 5
    ///
    /// We convert the u32 to a float in [-1, +1]:
    ///   1. Right-shift by 9 to get 23 bits (matches f32 mantissa precision)
    ///   2. Divide by 2^23 to get [0, 1)
    ///   3. Scale to [0, 2) and shift to [-1, +1)
    #[inline]
    fn next_noise_sample(&mut self) -> f32 {
        let mut x = self.noise_state;

        // Xorshift32 iteration
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;

        self.noise_state = x;

        // Convert to float in [-1, +1]
        // 8388608 = 2^23 (23-bit mantissa of f32)
        let normalized = (x >> 9) as f32 / 8388608.0;
        normalized * 2.0 - 1.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn phase_wrapping() {
        let mut osc = OscillatorBlock::sine();
        let mut buffer = [0.0; 128];
        let ctx = RenderCtx::from_note(48_000.0, 60, 100.0);
        osc.render(&mut buffer, &ctx);
        assert!(osc.phase < TAU);
    }
}
