/*
Distortion / Waveshaping Implementation
=======================================

Distortion adds harmonics by reshaping the waveform - it's how we add grit,
warmth, or aggression to a sound. This module provides three classic
waveshaping algorithms, each with a distinct character.


Vocabulary
----------

  waveshaping   A technique where each sample is transformed by a function.
                The function's shape determines the distortion character.

  transfer      The function f(x) that maps input to output. Plot it with
  function      input on x-axis, output on y-axis. A diagonal line = clean.

  drive         Pre-gain applied before the transfer function. Higher drive
                pushes the signal further into the nonlinear region.

  threshold     For clipping: the level beyond which the signal is affected.
                Lower threshold = more aggressive effect.

  harmonics     New frequencies created by distortion. A pure sine wave
                becomes a complex tone with overtones.

  odd harmonics Frequencies at 3×, 5×, 7×... the fundamental. Created by
                symmetric clipping. Sound "hollow" or "square-wave-like."

  even harmonics  Frequencies at 2×, 4×, 6×... the fundamental. Created by
                asymmetric distortion. Sound "warm" or "tube-like."


How Waveshaping Works
---------------------

A waveshaper applies a transfer function to each sample:

    output = f(input × drive)

When drive is low (1.0), the signal stays in the LINEAR region of f()
and passes through mostly unchanged. As drive increases, the signal
hits the NONLINEAR parts of f(), creating harmonic distortion.

    Input      Transfer Function      Output

     ╱╲            ___                 ___
    ╱  ╲     →    ╱   ╲      →        ╱   ╲
   ╱    ╲        ╱     ╲             ╱     ╲
              (soft clip)         (compressed peaks)


Transfer Functions
------------------

SOFT CLIP: f(x) = x / (1 + |x|)

    Smooth, warm saturation that gradually compresses peaks.
    Common in tube amp simulations. Asymptotically approaches ±1.

    Output
    +1 │        ____________
       │      ╱
       │    ╱
     0 │──╱─────────────────
       │╱
       │
    -1 │____________
       └─────────────────────→ Input (×drive)
         -10     0      +10

    Character: Warm, musical, forgiving. Works on everything.


HARD CLIP: f(x) = clamp(x, -threshold, +threshold)

    Abrupt limiting at the threshold. Creates a "squared off" waveform
    rich in odd harmonics. Think: guitar fuzz pedal.

    Output
    +t │    ┌───────────────
       │    │
       │   ╱│
     0 │──╱─│───────────────
       │╱   │
       │    │
    -t │────┘
       └─────────────────────→ Input (×drive)
         -t      0      +t

    Character: Harsh, buzzy, aggressive. Great for leads and bass.


FOLDBACK: When x exceeds threshold, it "folds" back on itself

    Instead of clipping flat, the waveform reflects. Creates complex,
    metallic harmonics. Popular in modular synthesis.

    Output
    +t │    ╱╲    ╱╲    ╱╲
       │   ╱  ╲  ╱  ╲  ╱
       │  ╱    ╲╱    ╲╱
     0 │─╱──────────────────
       │╱
    -t │
       └─────────────────────→ Input (×drive)
              (continues folding)

    Character: Metallic, synthy, extreme. Great for sound design.


The Math
--------

SOFT CLIP:
    y = x / (1 + |x|)

    where x = input × drive

    Example: input = 0.8, drive = 5.0
      x = 0.8 × 5.0 = 4.0
      y = 4.0 / (1 + 4.0) = 4.0 / 5.0 = 0.8

    Note: Output approaches ±1 but never exceeds it. Self-limiting.

HARD CLIP:
    y = clamp(x, -threshold, +threshold)

    Example: input = 0.8, drive = 2.0, threshold = 1.0
      x = 0.8 × 2.0 = 1.6
      y = clamp(1.6, -1.0, 1.0) = 1.0

FOLDBACK:
    While |x| > threshold:
        if x > threshold:  x = 2×threshold - x
        if x < -threshold: x = -2×threshold - x

    Example: input = 0.7, drive = 2.0, threshold = 1.0
      x = 0.7 × 2.0 = 1.4
      x > 1.0, so: x = 2×1.0 - 1.4 = 0.6
      Now |x| < threshold, done. y = 0.6


Drive Values Reference
----------------------

    1.0   = Clean (no distortion)
    2-4   = Warm saturation (subtle harmonics)
    5-10  = Obvious distortion (clearly audible effect)
    10+   = Heavy, aggressive (extreme transformation)


Implementation Notes
--------------------

Foldback uses a maximum iteration count to prevent infinite loops when
drive is extremely high. The final clamp ensures bounded output.

All functions are marked #[inline] for performance - waveshaping is
typically called once per sample in a hot loop.

The soft clip function x/(1+|x|) is cheaper than tanh() while sounding
nearly identical. It avoids the transcendental function call.
*/

/// Soft clipping using x / (1 + |x|) transfer function.
///
/// This produces warm, tube-like saturation that gradually
/// compresses peaks without harsh artifacts.
#[inline]
pub fn soft_clip(sample: f32, drive: f32) -> f32 {
    let x = sample * drive;
    x / (1.0 + x.abs())
}

/// Hard clipping - simply clamps the signal at a threshold.
///
/// Creates harsh, buzzy distortion rich in odd harmonics.
/// Lower threshold = more distortion.
#[inline]
pub fn hard_clip(sample: f32, drive: f32, threshold: f32) -> f32 {
    let x = sample * drive;
    x.clamp(-threshold, threshold)
}

/// Foldback distortion - signal folds back when exceeding threshold.
///
/// Creates complex harmonics with a metallic, synthy character.
/// More extreme than clipping at high drive values.
#[inline]
pub fn foldback(sample: f32, drive: f32, threshold: f32) -> f32 {
    let threshold = threshold.max(0.01); // Prevent zero/negative threshold
    let mut x = sample * drive;

    // Fold the signal when it exceeds threshold (limited iterations for safety)
    const MAX_FOLDS: u32 = 32;
    for _ in 0..MAX_FOLDS {
        if x > threshold {
            x = 2.0 * threshold - x;
        } else if x < -threshold {
            x = -2.0 * threshold - x;
        } else {
            break;
        }
    }

    // Final clamp as safety net
    x.clamp(-threshold, threshold)
}

/// Apply soft clipping to an entire buffer in place.
pub fn soft_clip_buffer(buffer: &mut [f32], drive: f32) {
    for sample in buffer.iter_mut() {
        *sample = soft_clip(*sample, drive);
    }
}

/// Apply hard clipping to an entire buffer in place.
pub fn hard_clip_buffer(buffer: &mut [f32], drive: f32, threshold: f32) {
    for sample in buffer.iter_mut() {
        *sample = hard_clip(*sample, drive, threshold);
    }
}

/// Apply foldback distortion to an entire buffer in place.
pub fn foldback_buffer(buffer: &mut [f32], drive: f32, threshold: f32) {
    for sample in buffer.iter_mut() {
        *sample = foldback(*sample, drive, threshold);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_soft_clip_unity_drive() {
        // At drive=1, small signals pass through almost unchanged
        let input = 0.1;
        let output = soft_clip(input, 1.0);
        // f(0.1) = 0.1 / (1 + 0.1) = 0.1 / 1.1 ≈ 0.0909
        assert!((output - 0.0909).abs() < 0.01);
    }

    #[test]
    fn test_soft_clip_high_drive() {
        // At high drive, output approaches ±1 asymptotically
        let output = soft_clip(1.0, 10.0);
        // f(10) = 10 / 11 ≈ 0.909
        assert!(output > 0.9 && output < 1.0);
    }

    #[test]
    fn test_hard_clip_below_threshold() {
        let output = hard_clip(0.3, 1.0, 1.0);
        assert!((output - 0.3).abs() < 1e-6);
    }

    #[test]
    fn test_hard_clip_above_threshold() {
        let output = hard_clip(0.8, 2.0, 1.0);
        // 0.8 * 2 = 1.6, clamped to 1.0
        assert!((output - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_foldback_below_threshold() {
        let output = foldback(0.3, 1.0, 1.0);
        assert!((output - 0.3).abs() < 1e-6);
    }

    #[test]
    fn test_foldback_above_threshold() {
        // 0.7 * 2 = 1.4, folds to 2*1 - 1.4 = 0.6
        let output = foldback(0.7, 2.0, 1.0);
        assert!((output - 0.6).abs() < 1e-6);
    }

    #[test]
    fn test_foldback_extreme_drive() {
        // Test that foldback doesn't hang with very high drive
        let output = foldback(1.0, 100.0, 0.5);
        assert!(output.is_finite());
        assert!(output.abs() <= 0.5);
    }

    #[test]
    fn test_foldback_zero_threshold() {
        // Zero threshold should be clamped to minimum
        let output = foldback(1.0, 2.0, 0.0);
        assert!(output.is_finite());
        assert!(output.abs() <= 0.01); // Clamped to 0.01 threshold
    }

    #[test]
    fn test_foldback_negative_threshold() {
        // Negative threshold should be clamped to minimum
        let output = foldback(1.0, 2.0, -1.0);
        assert!(output.is_finite());
    }
}
