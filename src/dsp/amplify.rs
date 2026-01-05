//! Signal multiplication primitive.

/*
Signal Multiplication
=====================

Multiplying two signals sample-by-sample is one of the most fundamental DSP
operations. Despite its simplicity, it enables powerful techniques.

Vocabulary
----------

  amplitude     The "height" or strength of a signal. For audio, typically
                in the range [-1.0, +1.0].

  gain          A multiplier applied to amplitude.
                  gain > 1.0  →  louder (amplification)
                  gain = 1.0  →  unchanged (unity gain)
                  gain < 1.0  →  quieter (attenuation)
                  gain = 0.0  →  silence

  attenuation   Reducing a signal's amplitude by multiplying by a value < 1.0.
                This is how envelopes control volume - they output 0.0 to 1.0,
                and multiplying by that value attenuates the signal.

  modulation    Using one signal to control a parameter of another.
                Amplitude modulation (AM) = controlling volume with a signal.


The Math
--------

For each sample index i:

    output[i] = signal[i] × modulator[i]

That's it. The power comes from what you choose as the modulator.


Attenuation in Decibels
-----------------------

Audio engineers often measure level changes in decibels (dB) because human
hearing is logarithmic - we perceive loudness ratios, not differences.

    dB = 20 × log₁₀(amplitude_ratio)

Common reference points:
    ×1.0   =   0 dB  (unity, no change)
    ×0.5   =  -6 dB  (half amplitude, noticeably quieter)
    ×0.1   = -20 dB  (one-tenth amplitude, much quieter)
    ×0.01  = -40 dB  (barely audible)
    ×2.0   =  +6 dB  (double amplitude, noticeably louder)

Every halving of amplitude ≈ -6 dB. Every doubling ≈ +6 dB.


Use Case 1: Envelope Control
----------------------------

The most common use. An envelope outputs a control signal (0.0 to 1.0) that
shapes the amplitude of an oscillator over time.

    Oscillator: [ 0.8, -0.6,  0.9, -0.7, ...]  (audio, full amplitude)
    Envelope:   [ 0.2,  0.5,  0.8,  1.0, ...]  (control, ramping up)
    Output:     [0.16, -0.3, 0.72, -0.7, ...]  (audio, shaped by envelope)

The envelope attenuates the oscillator, creating the attack-decay-sustain-release
shape we hear as a note.


Use Case 2: Tremolo
-------------------

Multiply by a slow LFO (< 20 Hz) to create rhythmic volume pulsing.

    Oscillator: [audio signal at 440 Hz]
    LFO:        [0.5, 0.7, 0.9, 1.0, 0.9, 0.7, 0.5, ...]  (slow sine, ~5 Hz)
    Output:     [audio with pulsing volume]

The LFO is too slow to hear as a pitch - we perceive it as the volume going
up and down. Classic guitar/organ effect.


Use Case 3: Ring Modulation
---------------------------

Multiply two audio-rate signals (both > 20 Hz) for metallic, bell-like, or
"robot voice" effects.

    Carrier:   [sine wave at 440 Hz]
    Modulator: [sine wave at 110 Hz]
    Output:    [contains 550 Hz and 330 Hz, but NOT 440 Hz or 110 Hz!]

The math: multiplying two sine waves produces their sum and difference
frequencies, while removing the originals:

    sin(A) × sin(B) = ½[cos(A-B) - cos(A+B)]

This creates inharmonic (non-musical-interval) relationships, which is why
ring mod sounds metallic or alien.

⚠️  ALIASING WARNING: If (f_carrier + f_modulator) exceeds the Nyquist
frequency (sample_rate / 2), those frequencies will "fold back" into the
audible range as artifacts. At 48 kHz, Nyquist is 24 kHz - usually safe for
typical audio, but watch out with very high frequencies.
Read more about nyquist here - https://en.wikipedia.org/wiki/Nyquist_frequency


Implementation Notes
--------------------

This operation is stateless - no memory of previous samples needed. Each output
sample depends only on the corresponding input samples at that instant.

The only "state" in the graph node wrapper is a temporary buffer to hold the
modulator's output while we compute the product.
*/

/// Multiply two signal buffers sample-by-sample.
///
/// This is the core operation for amplitude control (envelope × oscillator),
/// tremolo (LFO × audio), and ring modulation (audio × audio).
///
/// # Arguments
/// * `signal` - The primary signal (e.g., oscillator output)
/// * `modulator` - The control signal (e.g., envelope output)
/// * `out` - Output buffer, will contain signal × modulator
///
/// # Panics
/// Panics if the slices have different lengths.
#[inline]
pub fn multiply(signal: &[f32], modulator: &[f32], out: &mut [f32]) {
    debug_assert_eq!(signal.len(), modulator.len());
    debug_assert_eq!(signal.len(), out.len());

    for ((o, &s), &m) in out.iter_mut().zip(signal.iter()).zip(modulator.iter()) {
        *o = s * m;
    }
}

/// Multiply a signal by a constant gain factor (in-place).
///
/// Use this for simple volume control without a modulator signal.
///
/// # Arguments
/// * `signal` - The signal buffer to modify in-place
/// * `gain` - The gain factor (0.0 = silence, 1.0 = unchanged, 2.0 = double)
#[inline]
pub fn apply_gain(signal: &mut [f32], gain: f32) {
    for sample in signal.iter_mut() {
        *sample *= gain;
    }
}

/// Multiply a signal by a modulator, writing result into signal buffer (in-place).
///
/// More efficient when you don't need to preserve the original signal.
#[inline]
pub fn multiply_in_place(signal: &mut [f32], modulator: &[f32]) {
    debug_assert_eq!(signal.len(), modulator.len());

    for (s, &m) in signal.iter_mut().zip(modulator.iter()) {
        *s *= m;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_multiply_basic() {
        let signal = [1.0, 0.5, -0.5, -1.0];
        let modulator = [1.0, 0.5, 0.5, 0.0];
        let mut out = [0.0; 4];

        multiply(&signal, &modulator, &mut out);

        assert_eq!(out, [1.0, 0.25, -0.25, 0.0]);
    }

    #[test]
    fn test_apply_gain() {
        let mut signal = [1.0, 0.5, -0.5, -1.0];
        apply_gain(&mut signal, 0.5);
        assert_eq!(signal, [0.5, 0.25, -0.25, -0.5]);
    }

    #[test]
    fn test_multiply_in_place() {
        let mut signal = [1.0, 0.5, -0.5, -1.0];
        let modulator = [0.5, 0.5, 0.5, 0.5];
        multiply_in_place(&mut signal, &modulator);
        assert_eq!(signal, [0.5, 0.25, -0.25, -0.5]);
    }

    #[test]
    fn test_unity_gain_unchanged() {
        let signal = [0.3, -0.7, 0.5];
        let modulator = [1.0, 1.0, 1.0];
        let mut out = [0.0; 3];

        multiply(&signal, &modulator, &mut out);

        assert_eq!(out, signal);
    }

    #[test]
    fn test_zero_gain_silences() {
        let signal = [0.3, -0.7, 0.5];
        let modulator = [0.0, 0.0, 0.0];
        let mut out = [0.0; 3];

        multiply(&signal, &modulator, &mut out);

        assert_eq!(out, [0.0, 0.0, 0.0]);
    }
}
