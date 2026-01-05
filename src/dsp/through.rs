//! Serial signal chain primitive.

/*
Serial Signal Processing
========================

Serial processing passes audio through a chain of processors, where each
stage transforms the output of the previous stage. This is the fundamental
pattern for building audio effects chains.

Vocabulary
----------

  signal chain    A sequence of processors connected output-to-input.
                  Source → Effect A → Effect B → Output

  in-place        Processing that modifies the input buffer directly rather
                  than writing to a separate output. Most filters and effects
                  work this way for efficiency.

  wet signal      Audio that has been processed by an effect.
  dry signal      The original, unprocessed audio.

  insert          An effect placed in the main signal path (serial).
                  ALL audio passes through it.

  send/return     An effect in parallel (use Mix for this pattern).
                  Original audio is preserved, effect is blended in.


How It Works
------------

Serial processing is deceptively simple:

    1. Render source into buffer
    2. Pass buffer to processor (modifies in-place)
    3. Pass buffer to next processor (modifies in-place)
    ...repeat...

    ┌──────────┐     ┌──────────┐     ┌──────────┐
    │  Source  │ ──→ │ Effect A │ ──→ │ Effect B │ ──→ output
    └──────────┘     └──────────┘     └──────────┘
         buffer ────────────────────────────→
         (same buffer passed through, modified at each stage)


In-Place Processing
-------------------

Most DSP effects process buffers IN-PLACE, meaning they read from and write
to the same buffer:

    // Before: buffer contains source signal
    buffer = [0.5, 0.8, -0.3, 0.2, ...]

    effect.process(buffer);  // Filter reads AND writes to same buffer

    // After: buffer contains filtered signal
    buffer = [0.4, 0.6, -0.2, 0.15, ...]

Why in-place?
- No extra memory allocation
- Better cache locality
- Common pattern in audio APIs

The tradeoff: you lose the original signal. If you need both dry and wet,
use Mix to blend a copy of the original with the processed version.


Order Matters!
--------------

Signal chain order dramatically affects the sound:

    // Option A: Filter then Distortion
    osc.through(filter).through(distortion)
    // Distortion acts on filtered (smoother) signal
    // Result: cleaner, more controlled distortion

    // Option B: Distortion then Filter
    osc.through(distortion).through(filter)
    // Filter removes some distortion harmonics
    // Result: warmer, less harsh

    // Option C: Envelope before Filter
    osc.amplify(env).through(filter)
    // Filter processes envelope-shaped signal
    // Quieter parts get filtered differently

    // Option D: Filter before Envelope
    osc.through(filter).amplify(env)
    // Filter processes full-amplitude signal
    // Envelope shapes the filtered result

Classic synth order: Oscillator → Filter → Amplifier (VCA)
This is "subtractive synthesis" - start bright, filter away harmonics.


Feedback (Not Implemented Here)
-------------------------------

Some effects (reverb, delay) feed their output back into their input:

    ┌─────────────────────────┐
    │     ┌──────────────┐    │
    └──→  │    Delay     │ ───┴──→ output
          └──────────────┘
               ↑     │
               └─────┘ (feedback)

This requires special handling (delay buffers) and isn't just simple
serial chaining. See `dsp/delay.rs` for feedback implementation.


Implementation
--------------

The actual "through" operation is trivial - just call render twice:

    source.render(buffer);
    effect.render(buffer);

The educational value is understanding WHY this works and what it means
for signal flow. The complexity lives in the individual processors
(filters, delays, etc.), not in the chaining itself.
*/

/// Process a buffer through an effect in-place.
///
/// This is the conceptual primitive, though in practice you'd just call
/// the effect's render method directly. It's here for documentation.
///
/// # Arguments
/// * `buffer` - Audio buffer to process (modified in-place)
/// * `process` - Function that processes the buffer
#[inline]
pub fn process_in_place<F>(buffer: &mut [f32], mut process: F)
where
    F: FnMut(&mut [f32]),
{
    process(buffer);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_in_place() {
        let mut buffer = [1.0, 2.0, 3.0, 4.0];

        // Simple "effect": halve all values
        process_in_place(&mut buffer, |buf| {
            for sample in buf.iter_mut() {
                *sample *= 0.5;
            }
        });

        assert_eq!(buffer, [0.5, 1.0, 1.5, 2.0]);
    }

    #[test]
    fn test_chained_processing() {
        let mut buffer = [1.0, 2.0, 3.0, 4.0];

        // Chain two "effects"
        process_in_place(&mut buffer, |buf| {
            for sample in buf.iter_mut() {
                *sample *= 2.0; // Double
            }
        });
        process_in_place(&mut buffer, |buf| {
            for sample in buf.iter_mut() {
                *sample -= 1.0; // Subtract 1
            }
        });

        // 1*2-1=1, 2*2-1=3, 3*2-1=5, 4*2-1=7
        assert_eq!(buffer, [1.0, 3.0, 5.0, 7.0]);
    }
}
