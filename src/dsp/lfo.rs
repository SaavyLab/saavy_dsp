//! Low Frequency Oscillator (LFO) concepts.

/*
Low Frequency Oscillators
=========================

An LFO is simply an oscillator running at sub-audio frequencies. The same
waveform math applies, but the context and usage are completely different.

Vocabulary
----------

  audio-rate      Frequencies humans can hear: ~20 Hz to ~20,000 Hz.
                  These oscillators produce the actual sound you hear.

  control-rate    Frequencies below human hearing: ~0.01 Hz to ~20 Hz.
                  These oscillators modulate parameters over time.
                  "Control" because they control other things, not make sound.

  LFO             Low Frequency Oscillator. An oscillator at control-rate.
                  Typically outputs -1.0 to +1.0 (bipolar) for modulation.

  modulator       A signal that varies another signal's parameter.
                  LFOs are the classic modulator in synthesizers.

  period          Time for one complete oscillation.
                  At 5 Hz: period = 1/5 = 0.2 seconds = 200ms

  bipolar         Output swings positive AND negative: -1.0 to +1.0
                  Used when you want the parameter to go above AND below center.

  unipolar        Output is only positive: 0.0 to 1.0
                  Used when parameter should only move in one direction.
                  Convert: unipolar = (bipolar + 1.0) / 2.0


Typical LFO Frequencies
-----------------------

Different effects work best at different speeds:

    0.01 - 0.1 Hz   Very slow sweeps, evolving textures
    0.1 - 0.5 Hz    Slow sweeps, gradual filter movement
    0.5 - 2 Hz      Classic tremolo, auto-pan
    2 - 7 Hz        Vibrato sweet spot
    7 - 15 Hz       Fast tremolo, "helicopter" effect
    > 15 Hz         Approaching audio rate (FM/AM territory)

At the upper end (~20 Hz), you cross into audio-rate territory. This creates
FM (frequency modulation) or AM (amplitude modulation) effects, which produce
sidebands and new harmonic content rather than perceived "wobble."


Common LFO Shapes
-----------------

Each waveform creates a different character of movement:

SINE
    Smooth, natural, organic sweep. The most common choice.
    Good for: vibrato, subtle filter sweeps, natural-sounding effects

TRIANGLE
    Similar to sine but with constant rate of change.
    Slightly more "linear" feel than sine.
    Good for: vibrato, smooth sweeps where you want constant motion

SAWTOOTH (Ramp)
    Gradual rise, instant reset (or vice versa).
    Creates rhythmic, one-directional sweeps.
    Good for: synced effects, sequencer-like stepping, risers

    Rising saw:  ╱╱╱╱   (gradual up, snap down)
    Falling saw: ╲╲╲╲   (gradual down, snap up)

SQUARE
    Instant switch between two values. No transition.
    Good for: gated effects, hard tremolo, on/off switching

SAMPLE & HOLD (not implemented here)
    Random value held for a fixed time, then new random value.
    Good for: "computer bleep" effects, random modulation


Sync and Phase
--------------

LFOs can be:

FREE-RUNNING: LFO runs continuously, phase unrelated to notes.
    - Different character each time you play a note
    - More "alive" but less predictable

SYNCED: LFO resets phase when note starts (note_on triggers reset).
    - Same modulation shape every time
    - More predictable, good for rhythmic effects

TEMPO-SYNCED (not implemented here): LFO frequency locked to BPM.
    - 1/4 note, 1/8 note, etc.
    - Modulation aligns with musical time


Implementation Note
-------------------

LFOs use the exact same math as audio oscillators. The only differences:

1. Frequency range: LFOs use 0.01-20 Hz instead of 20-20000 Hz
2. Note independence: LFOs ignore MIDI note input, use fixed frequency
3. Context: LFOs output control signals, not audio samples

In this crate, LfoNode wraps OscillatorBlock and overrides the frequency
from the render context with its own fixed value. The waveform generation
is identical - see `dsp/oscillator.rs` for the underlying math.


Bipolar to Unipolar Conversion
------------------------------

Many parameters want unipolar (0 to 1) rather than bipolar (-1 to +1):

    unipolar = (bipolar + 1.0) * 0.5

    bipolar   unipolar
    -1.0      0.0
     0.0      0.5
    +1.0      1.0

For envelope-style modulation (0 at rest, positive when active),
unipolar often makes more sense. For symmetric effects like vibrato
(pitch goes sharp AND flat), bipolar is natural.
*/

/// Convert bipolar signal (-1.0 to +1.0) to unipolar (0.0 to 1.0).
///
/// Useful when a parameter expects positive-only modulation.
#[inline]
pub fn bipolar_to_unipolar(bipolar: f32) -> f32 {
    (bipolar + 1.0) * 0.5
}

/// Convert unipolar signal (0.0 to 1.0) to bipolar (-1.0 to +1.0).
///
/// Useful when you have a 0-1 source but need symmetric modulation.
#[inline]
pub fn unipolar_to_bipolar(unipolar: f32) -> f32 {
    (unipolar * 2.0) - 1.0
}

/// Calculate LFO period in seconds from frequency.
///
/// # Example
/// ```
/// use saavy_dsp::dsp::lfo::period_from_frequency;
/// let period = period_from_frequency(5.0);
/// assert!((period - 0.2).abs() < 1e-6); // 5 Hz = 200ms period
/// ```
#[inline]
pub fn period_from_frequency(frequency_hz: f32) -> f32 {
    1.0 / frequency_hz
}

/// Calculate samples per LFO period.
///
/// Useful for understanding how many samples one complete LFO cycle takes.
///
/// # Example
/// ```
/// use saavy_dsp::dsp::lfo::samples_per_period;
/// let samples = samples_per_period(5.0, 48000.0);
/// assert_eq!(samples, 9600.0); // 5 Hz at 48kHz = 9600 samples
/// ```
#[inline]
pub fn samples_per_period(frequency_hz: f32, sample_rate: f32) -> f32 {
    sample_rate / frequency_hz
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bipolar_to_unipolar() {
        assert!((bipolar_to_unipolar(-1.0) - 0.0).abs() < 1e-6);
        assert!((bipolar_to_unipolar(0.0) - 0.5).abs() < 1e-6);
        assert!((bipolar_to_unipolar(1.0) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_unipolar_to_bipolar() {
        assert!((unipolar_to_bipolar(0.0) - (-1.0)).abs() < 1e-6);
        assert!((unipolar_to_bipolar(0.5) - 0.0).abs() < 1e-6);
        assert!((unipolar_to_bipolar(1.0) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_roundtrip_conversion() {
        for &val in &[-1.0, -0.5, 0.0, 0.5, 1.0] {
            let roundtrip = unipolar_to_bipolar(bipolar_to_unipolar(val));
            assert!(
                (roundtrip - val).abs() < 1e-6,
                "Roundtrip failed for {}: got {}",
                val,
                roundtrip
            );
        }
    }

    #[test]
    fn test_period_from_frequency() {
        assert!((period_from_frequency(5.0) - 0.2).abs() < 1e-6);
        assert!((period_from_frequency(1.0) - 1.0).abs() < 1e-6);
        assert!((period_from_frequency(10.0) - 0.1).abs() < 1e-6);
    }

    #[test]
    fn test_samples_per_period() {
        // 5 Hz at 48000 Hz = 9600 samples per cycle
        assert!((samples_per_period(5.0, 48000.0) - 9600.0).abs() < 1e-6);
        // 1 Hz at 48000 Hz = 48000 samples per cycle
        assert!((samples_per_period(1.0, 48000.0) - 48000.0).abs() < 1e-6);
    }
}
