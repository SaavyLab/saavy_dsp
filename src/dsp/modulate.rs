//! Parameter modulation primitives.

/*
Parameter Modulation
====================

Modulation is using one signal to continuously vary a parameter of another.
It's what makes synthesizers sound alive - without modulation, everything
sounds static and artificial.

Vocabulary
----------

  modulation    Continuously varying a parameter over time using a control signal.

  modulator     The signal doing the controlling (usually an LFO or envelope).
                Typically outputs values in [-1.0, +1.0] or [0.0, 1.0].

  target        The parameter being modulated (e.g., filter cutoff, pitch).

  depth         How much the parameter changes. Scales the modulator's output.
                  final_value = base_value + (modulator × depth)

  base value    The parameter's "center" value when modulator = 0.

  bipolar       Modulator swings positive AND negative (LFO sine: -1 to +1).
                Parameter moves above AND below the base value.

  unipolar      Modulator is only positive (envelope: 0 to 1).
                Parameter only moves in one direction from base.


The Math
--------

For each sample (or block):

    modulated_value = base_value + (modulator_output × depth)

Example: Filter cutoff with LFO
    base_value = 1000 Hz
    depth = 500 Hz
    LFO output swings from -1.0 to +1.0

    When LFO = -1.0:  cutoff = 1000 + (-1.0 × 500) = 500 Hz
    When LFO =  0.0:  cutoff = 1000 + ( 0.0 × 500) = 1000 Hz
    When LFO = +1.0:  cutoff = 1000 + (+1.0 × 500) = 1500 Hz

The parameter sweeps between 500 Hz and 1500 Hz, centered on 1000 Hz.


Common Modulation Effects
-------------------------

VIBRATO: LFO → Pitch
    Pitch wobble. LFO at 5-7 Hz, small depth.
    Makes notes sound more human/expressive.

TREMOLO: LFO → Amplitude
    Volume wobble. LFO at 2-10 Hz.
    Classic guitar/organ effect. (Use Amplify node for this.)

WAH / FILTER SWEEP: LFO → Filter Cutoff
    Tone wobble. LFO at 0.5-5 Hz, large depth.
    The "wah-wah" or "auto-wah" effect.

PWM (Pulse Width Modulation): LFO → Square Wave Duty Cycle
    Timbre animation. Makes square waves sound "fatter."

SIREN: LFO → Pitch with large depth
    Pitch slides up and down dramatically.


Block-Rate vs Sample-Rate Modulation
------------------------------------

SAMPLE-RATE: Update the parameter every single sample.
    Pro: Perfectly smooth modulation
    Con: Expensive (recalculate filter coefficients 48000 times/sec!)
    When: High-frequency modulation, critical smoothness

BLOCK-RATE (what we use): Update once per audio block (~every 64-2048 samples).
    Pro: Much more efficient
    Pro: Works with existing node architectures
    Con: Slight "stepping" if block size is large and LFO is fast
    When: Most typical use cases (LFO at < 20 Hz)

For an LFO at 5 Hz with 512-sample blocks at 48kHz:
    - Block duration: 512/48000 = ~10.7ms
    - LFO period: 200ms
    - Updates per LFO cycle: ~19

That's plenty smooth for most musical applications.


Averaging vs Sampling
---------------------

For block-rate modulation, we need ONE value from the LFO for the whole block.
Two approaches:

AVERAGE: Sum all LFO samples in the block, divide by count.
    Pro: Smooth, represents the "middle" of the block
    Con: Slightly more CPU (sum N values)
    This is what we implement.

SAMPLE: Just take the first (or last) LFO sample.
    Pro: Simpler, faster
    Con: Ignores LFO movement within the block


Parameter Clamping
------------------

Modulation can push parameters outside valid ranges:
    base = 1000 Hz, depth = 2000 Hz, LFO = +1.0
    result = 3000 Hz  (might be fine)

    base = 500 Hz, depth = 1000 Hz, LFO = -1.0
    result = -500 Hz  (invalid! frequency can't be negative)

The target node is responsible for clamping. FilterNode clamps cutoff to
[20, 20000] Hz. This prevents crashes but can cause "flat spots" in the
modulation where the parameter hits its limit.

Design your base value and depth so the modulated range stays valid:
    cutoff range: [500, 4500] with base=2500, depth=2000 ✓
    cutoff range: [-500, 1500] with base=500, depth=1000 ✗ (hits negative)
*/

/// Calculate the modulated parameter value.
///
/// # Arguments
/// * `base_value` - The parameter's center/default value
/// * `modulator` - The control signal value (typically -1.0 to +1.0)
/// * `depth` - How much the parameter should vary
///
/// # Returns
/// The modulated value: base + (modulator × depth)
#[inline]
pub fn apply_modulation(base_value: f32, modulator: f32, depth: f32) -> f32 {
    base_value + (modulator * depth)
}

/// Calculate the average of a modulator signal over a block.
///
/// Used for block-rate modulation: we need one value to represent
/// the entire block's worth of modulator samples.
#[inline]
pub fn block_average(samples: &[f32]) -> f32 {
    if samples.is_empty() {
        return 0.0;
    }
    samples.iter().sum::<f32>() / samples.len() as f32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_apply_modulation_center() {
        // Modulator at 0 should give base value
        assert_eq!(apply_modulation(1000.0, 0.0, 500.0), 1000.0);
    }

    #[test]
    fn test_apply_modulation_positive() {
        // Modulator at +1 should add depth
        assert_eq!(apply_modulation(1000.0, 1.0, 500.0), 1500.0);
    }

    #[test]
    fn test_apply_modulation_negative() {
        // Modulator at -1 should subtract depth
        assert_eq!(apply_modulation(1000.0, -1.0, 500.0), 500.0);
    }

    #[test]
    fn test_apply_modulation_partial() {
        // Modulator at 0.5 should add half depth
        assert_eq!(apply_modulation(1000.0, 0.5, 500.0), 1250.0);
    }

    #[test]
    fn test_block_average() {
        let samples = [1.0, 2.0, 3.0, 4.0];
        assert_eq!(block_average(&samples), 2.5);
    }

    #[test]
    fn test_block_average_empty() {
        let samples: [f32; 0] = [];
        assert_eq!(block_average(&samples), 0.0);
    }

    #[test]
    fn test_block_average_single() {
        let samples = [0.5];
        assert_eq!(block_average(&samples), 0.5);
    }
}
