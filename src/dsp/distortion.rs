//! Distortion / Waveshaping
//!
//! Distortion adds harmonics by reshaping the waveform. The "drive" parameter
//! controls how aggressively the signal is pushed into the nonlinear region.
//!
//! # How Waveshaping Works
//!
//! A waveshaper applies a transfer function to each sample:
//!   output = f(input * drive)
//!
//! When drive is low (1.0), the signal stays in the linear region of f()
//! and passes through mostly unchanged. As drive increases, the signal hits
//! the nonlinear parts of f(), creating harmonic distortion.
//!
//! # Common Waveshaping Functions
//!
//! Soft Clip (tanh-style):
//!   f(x) = x / (1 + |x|)
//!   - Smooth, warm saturation
//!   - Gradually compresses peaks
//!   - Common in tube amp simulations
//!
//! Hard Clip:
//!   f(x) = clamp(x, -threshold, threshold)
//!   - Harsh, buzzy distortion
//!   - Creates odd harmonics (like square wave)
//!   - Think: guitar fuzz pedal
//!
//! Foldback:
//!   When x exceeds threshold, it "folds" back on itself
//!   - Creates complex, metallic harmonics
//!   - Popular in modular synthesis
//!   - More extreme than clipping
//!
//! # Drive Values
//!
//!   1.0  = Clean (no distortion)
//!   2-4  = Warm saturation
//!   5-10 = Obvious distortion
//!   10+  = Heavy, aggressive

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
    let mut x = sample * drive;

    // Fold the signal when it exceeds threshold
    while x > threshold || x < -threshold {
        if x > threshold {
            x = 2.0 * threshold - x;
        } else if x < -threshold {
            x = -2.0 * threshold - x;
        }
    }

    x
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
}
