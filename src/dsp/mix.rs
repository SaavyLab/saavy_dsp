//! Signal mixing and crossfading primitives.

/*
Signal Mixing
=============

Mixing combines two signals by ADDING them together, optionally with weights.
This is the additive counterpart to multiplication (amplify).

Vocabulary
----------

  mixing        Combining signals by addition. The result contains both signals
                superimposed.

  crossfade     Transitioning between two signals using complementary weights.
                As one fades out, the other fades in.

  balance       A control value (0.0 to 1.0) that determines the mix ratio.
                  balance = 0.0  →  100% signal A, 0% signal B
                  balance = 0.5  →  50% A, 50% B
                  balance = 1.0  →  0% A, 100% B

  wet/dry       Common terminology for effect mixing.
                  dry = original signal (unprocessed)
                  wet = effect signal (processed)
                A "30% wet" mix means balance = 0.3.

  summing       Adding signals at equal levels (no weighting). Can cause
                clipping if signals are already near full scale.


The Math: Linear Crossfade
--------------------------

For each sample:

    output = (A × weight_a) + (B × weight_b)

    where weight_a = 1.0 - balance
          weight_b = balance

Example with balance = 0.3:
    weight_a = 0.7, weight_b = 0.3
    output = (A × 0.7) + (B × 0.3)

The weights always sum to 1.0, preventing overall level boost.


Linear vs Equal-Power Crossfade
-------------------------------

LINEAR (what we implement):

    weight_a = 1.0 - balance
    weight_b = balance

    At balance = 0.5: both signals at 50% amplitude

    Problem: Perceived loudness DIPS in the middle of a crossfade.
    Why? Two uncorrelated signals at 50% each don't sound as loud as
    one signal at 100%. (Power adds, not amplitude.)

    Level
      1.0 ──────╲      ╱──────
                 ╲    ╱
      0.5         ╲  ╱  ← loudness dip here
                   ╲╱
      0.0 ─────────────────────
          0.0     0.5     1.0
                balance

EQUAL-POWER (not implemented here):

    weight_a = cos(balance × π/2)  or  sqrt(1.0 - balance)
    weight_b = sin(balance × π/2)  or  sqrt(balance)

    At balance = 0.5: both signals at ~70.7% amplitude (√0.5)

    Result: Constant perceived loudness through the crossfade.
    Cost: Slightly more CPU (trig or sqrt).

For most synth applications, linear crossfade is fine. The loudness dip
is subtle and often masked by other sounds. Use equal-power for DJ-style
crossfades or when smoothness is critical.


Clipping Risk
-------------

When mixing at equal levels (balance = 0.5), two signals that each peak
at 1.0 could sum to 2.0 - beyond the normal [-1.0, +1.0] range.

    Signal A:  [ 1.0,  0.5, -0.5, -1.0]
    Signal B:  [ 1.0,  0.8,  0.2, -0.5]
    Sum:       [ 2.0,  1.3, -0.3, -1.5]  ← exceeds ±1.0!

Solutions:
1. Attenuate before mixing (apply gain < 1.0 to inputs)
2. Use weighted mix (our approach - weights sum to 1.0)
3. Apply limiting/clipping after mixing

Our crossfade approach (weights sum to 1.0) prevents this naturally:
    (A × 0.5) + (B × 0.5) = max 1.0 if A and B both peak at 1.0


Phase Relationships
-------------------

When mixing similar signals (e.g., two sine waves at the same frequency):

  IN PHASE:      Signals add constructively → louder (up to 2×)
  OUT OF PHASE:  Signals cancel → quieter (potentially silent!)

This is why detuned oscillators sound "fat" - the phase relationship
constantly shifts, creating movement. And why mixing an inverted copy
of a signal with itself produces silence.

For typical synth mixing (different waveforms, different frequencies),
phase relationships average out and aren't a concern.
*/

/// Mix two signals using linear crossfade.
///
/// output = (A × (1-balance)) + (B × balance)
///
/// # Arguments
/// * `a` - First signal buffer
/// * `b` - Second signal buffer
/// * `balance` - Mix ratio (0.0 = all A, 0.5 = equal, 1.0 = all B)
/// * `out` - Output buffer
#[inline]
pub fn mix(a: &[f32], b: &[f32], balance: f32, out: &mut [f32]) {
    debug_assert_eq!(a.len(), b.len());
    debug_assert_eq!(a.len(), out.len());

    let balance = balance.clamp(0.0, 1.0);
    let weight_a = 1.0 - balance;
    let weight_b = balance;

    for ((&sa, &sb), o) in a.iter().zip(b.iter()).zip(out.iter_mut()) {
        *o = (sa * weight_a) + (sb * weight_b);
    }
}

/// Mix signal B into signal A in-place using linear crossfade.
///
/// a = (A × (1-balance)) + (B × balance)
#[inline]
pub fn mix_in_place(a: &mut [f32], b: &[f32], balance: f32) {
    debug_assert_eq!(a.len(), b.len());

    let balance = balance.clamp(0.0, 1.0);
    let weight_a = 1.0 - balance;
    let weight_b = balance;

    for (sa, &sb) in a.iter_mut().zip(b.iter()) {
        *sa = (*sa * weight_a) + (sb * weight_b);
    }
}

/// Sum two signals together without weighting.
///
/// ⚠️ WARNING: Can exceed [-1.0, +1.0] range! Apply gain before or limiting after.
#[inline]
pub fn sum(a: &[f32], b: &[f32], out: &mut [f32]) {
    debug_assert_eq!(a.len(), b.len());
    debug_assert_eq!(a.len(), out.len());

    for ((&sa, &sb), o) in a.iter().zip(b.iter()).zip(out.iter_mut()) {
        *o = sa + sb;
    }
}

/// Add signal B into signal A in-place (summing).
///
/// ⚠️ WARNING: Can exceed [-1.0, +1.0] range!
#[inline]
pub fn sum_in_place(a: &mut [f32], b: &[f32]) {
    debug_assert_eq!(a.len(), b.len());

    for (sa, &sb) in a.iter_mut().zip(b.iter()) {
        *sa += sb;
    }
}

/// Blend dry and wet samples using linear crossfade (single sample version).
///
/// output = (dry × (1-mix)) + (wet × mix)
///
/// This is the common dry/wet mixing pattern used in effects.
#[inline]
pub fn blend_dry_wet(dry: f32, wet: f32, mix: f32) -> f32 {
    dry * (1.0 - mix) + wet * mix
}

/// Apply dry/wet mixing to a buffer, blending original (dry) with processed (wet).
///
/// wet[i] = (dry[i] × (1-mix)) + (wet[i] × mix)
///
/// Modifies `wet` in-place, using `dry` as the unprocessed reference.
#[inline]
pub fn apply_dry_wet(dry: &[f32], wet: &mut [f32], mix: f32) {
    debug_assert_eq!(dry.len(), wet.len());

    if mix >= 1.0 {
        return; // 100% wet, nothing to do
    }

    let dry_amount = 1.0 - mix;
    for (wet_sample, &dry_sample) in wet.iter_mut().zip(dry.iter()) {
        *wet_sample = dry_sample * dry_amount + *wet_sample * mix;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mix_all_a() {
        let a = [1.0, 0.5, -0.5, -1.0];
        let b = [0.0, 0.0, 0.0, 0.0];
        let mut out = [0.0; 4];

        mix(&a, &b, 0.0, &mut out);

        assert_eq!(out, [1.0, 0.5, -0.5, -1.0]);
    }

    #[test]
    fn test_mix_all_b() {
        let a = [0.0, 0.0, 0.0, 0.0];
        let b = [1.0, 0.5, -0.5, -1.0];
        let mut out = [0.0; 4];

        mix(&a, &b, 1.0, &mut out);

        assert_eq!(out, [1.0, 0.5, -0.5, -1.0]);
    }

    #[test]
    fn test_mix_equal() {
        let a = [1.0, 1.0, 1.0, 1.0];
        let b = [0.0, 0.0, 0.0, 0.0];
        let mut out = [0.0; 4];

        mix(&a, &b, 0.5, &mut out);

        assert_eq!(out, [0.5, 0.5, 0.5, 0.5]);
    }

    #[test]
    fn test_mix_in_place() {
        let mut a = [1.0, 1.0, 1.0, 1.0];
        let b = [0.0, 0.0, 0.0, 0.0];

        mix_in_place(&mut a, &b, 0.5);

        assert_eq!(a, [0.5, 0.5, 0.5, 0.5]);
    }

    #[test]
    fn test_sum_can_exceed_one() {
        let a = [1.0, 0.5];
        let b = [1.0, 0.8];
        let mut out = [0.0; 2];

        sum(&a, &b, &mut out);

        assert_eq!(out[0], 2.0); // Exceeds 1.0!
        assert_eq!(out[1], 1.3);
    }

    #[test]
    fn test_balance_clamped() {
        let a = [1.0];
        let b = [0.0];
        let mut out = [0.0; 1];

        // Balance > 1.0 should clamp to 1.0
        mix(&a, &b, 2.0, &mut out);
        assert_eq!(out[0], 0.0); // All B

        // Balance < 0.0 should clamp to 0.0
        mix(&a, &b, -1.0, &mut out);
        assert_eq!(out[0], 1.0); // All A
    }

    #[test]
    fn test_weights_sum_to_one() {
        // Verify that max input produces max output (no boost)
        let a = [1.0];
        let b = [1.0];
        let mut out = [0.0; 1];

        mix(&a, &b, 0.5, &mut out);

        // (1.0 × 0.5) + (1.0 × 0.5) = 1.0, not 2.0
        assert_eq!(out[0], 1.0);
    }

    #[test]
    fn test_blend_dry_wet() {
        // All dry
        assert_eq!(blend_dry_wet(1.0, 0.5, 0.0), 1.0);
        // All wet
        assert_eq!(blend_dry_wet(1.0, 0.5, 1.0), 0.5);
        // 50/50 mix
        assert_eq!(blend_dry_wet(1.0, 0.0, 0.5), 0.5);
    }

    #[test]
    fn test_apply_dry_wet_all_dry() {
        let dry = [1.0, 0.5, -0.5, -1.0];
        let mut wet = [0.0, 0.0, 0.0, 0.0];

        apply_dry_wet(&dry, &mut wet, 0.0);

        assert_eq!(wet, [1.0, 0.5, -0.5, -1.0]); // All dry
    }

    #[test]
    fn test_apply_dry_wet_all_wet() {
        let dry = [0.0, 0.0, 0.0, 0.0];
        let mut wet = [1.0, 0.5, -0.5, -1.0];
        let original = wet.clone();

        apply_dry_wet(&dry, &mut wet, 1.0);

        assert_eq!(wet, original); // All wet, unchanged
    }

    #[test]
    fn test_apply_dry_wet_half() {
        let dry = [1.0, 1.0, 1.0, 1.0];
        let mut wet = [0.0, 0.0, 0.0, 0.0];

        apply_dry_wet(&dry, &mut wet, 0.5);

        assert_eq!(wet, [0.5, 0.5, 0.5, 0.5]); // 50/50 mix
    }
}
