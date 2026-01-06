//! Reverb - Room Simulation via Delay Networks
//!
//! Reverb simulates the sound of a space by creating many delayed, filtered
//! reflections of the input signal. This implementation uses the classic
//! Schroeder reverb algorithm.
//!
//! # Schroeder Reverb Architecture
//!
//! ```text
//! Input ──┬──→ [Comb 1] ──┐
//!         ├──→ [Comb 2] ──┤
//!         ├──→ [Comb 3] ──┼──→ (+) ──→ [Allpass 1] ──→ [Allpass 2] ──→ Output
//!         └──→ [Comb 4] ──┘
//! ```
//!
//! ## Comb Filters
//!
//! A comb filter creates a series of equally-spaced echoes that decay over time.
//! Multiple comb filters with different delay times create a dense reverb tail.
//!
//! ```text
//! y[n] = x[n] + feedback * y[n - delay]
//! ```
//!
//! The delay times are chosen to be mutually prime (no common factors) to avoid
//! resonant buildup at specific frequencies.
//!
//! ## Allpass Filters
//!
//! Allpass filters pass all frequencies equally but shift their phase. In reverb,
//! they add density and diffusion without coloring the sound.
//!
//! ```text
//! y[n] = -g * x[n] + x[n - delay] + g * y[n - delay]
//! ```
//!
//! # Parameters
//!
//! - **Room Size**: Scales all delay times (larger = longer reverb)
//! - **Damping**: High-frequency absorption (higher = darker sound)
//! - **Feedback**: Controls reverb decay time

/// Max comb filter delay: 50ms at 192kHz = 9600 samples
const MAX_COMB_DELAY: usize = 9600;
/// Max allpass filter delay: 10ms at 192kHz = 1920 samples
const MAX_ALLPASS_DELAY: usize = 1920;

/// A simple comb filter for reverb (pre-allocated, RT-safe)
pub struct CombFilter {
    buffer: [f32; MAX_COMB_DELAY],
    delay_samples: usize,
    write_pos: usize,
    feedback: f32,
    damp: f32,
    filter_state: f32,
}

impl CombFilter {
    pub fn new(delay_samples: usize) -> Self {
        Self {
            buffer: [0.0; MAX_COMB_DELAY],
            delay_samples: delay_samples.min(MAX_COMB_DELAY).max(1),
            write_pos: 0,
            feedback: 0.5,
            damp: 0.5,
            filter_state: 0.0,
        }
    }

    pub fn set_feedback(&mut self, feedback: f32) {
        self.feedback = feedback.clamp(0.0, 0.99);
    }

    pub fn set_damp(&mut self, damp: f32) {
        self.damp = damp.clamp(0.0, 1.0);
    }

    /// Set delay length (RT-safe, no allocation)
    pub fn set_delay(&mut self, delay_samples: usize) {
        self.delay_samples = delay_samples.min(MAX_COMB_DELAY).max(1);
        self.write_pos = self.write_pos % self.delay_samples;
    }

    pub fn process(&mut self, input: f32) -> f32 {
        let output = self.buffer[self.write_pos];

        // One-pole lowpass filter for damping (absorbs high frequencies)
        self.filter_state = output * (1.0 - self.damp) + self.filter_state * self.damp;

        // Write new sample: input + filtered feedback
        self.buffer[self.write_pos] = input + self.filter_state * self.feedback;

        // Advance write position (wrap at actual delay length)
        self.write_pos = (self.write_pos + 1) % self.delay_samples;

        output
    }

    pub fn reset(&mut self) {
        self.buffer.fill(0.0);
        self.filter_state = 0.0;
        self.write_pos = 0;
    }
}

/// An allpass filter for reverb diffusion (pre-allocated, RT-safe)
pub struct AllpassFilter {
    buffer: [f32; MAX_ALLPASS_DELAY],
    delay_samples: usize,
    write_pos: usize,
    feedback: f32,
}

impl AllpassFilter {
    pub fn new(delay_samples: usize) -> Self {
        Self {
            buffer: [0.0; MAX_ALLPASS_DELAY],
            delay_samples: delay_samples.min(MAX_ALLPASS_DELAY).max(1),
            write_pos: 0,
            feedback: 0.5,
        }
    }

    pub fn set_feedback(&mut self, feedback: f32) {
        self.feedback = feedback.clamp(0.0, 0.9);
    }

    /// Set delay length (RT-safe, no allocation)
    pub fn set_delay(&mut self, delay_samples: usize) {
        self.delay_samples = delay_samples.min(MAX_ALLPASS_DELAY).max(1);
        self.write_pos = self.write_pos % self.delay_samples;
    }

    pub fn process(&mut self, input: f32) -> f32 {
        let delayed = self.buffer[self.write_pos];

        // Allpass: output = -g*input + delayed + g*delayed_output
        let output = -self.feedback * input + delayed;

        // Write: input + feedback * output
        self.buffer[self.write_pos] = input + self.feedback * output;

        // Advance write position (wrap at actual delay length)
        self.write_pos = (self.write_pos + 1) % self.delay_samples;

        output
    }

    pub fn reset(&mut self) {
        self.buffer.fill(0.0);
        self.write_pos = 0;
    }
}

/// Schroeder reverb with 4 comb filters and 2 allpass filters
pub struct SchroederReverb {
    combs: [CombFilter; 4],
    allpasses: [AllpassFilter; 2],
}

impl SchroederReverb {
    /// Create a new Schroeder reverb at the given sample rate.
    ///
    /// Delay times are tuned for a natural room sound.
    pub fn new(sample_rate: f32) -> Self {
        // Comb filter delay times in ms (mutually prime ratios)
        // These create a dense, natural-sounding reverb tail
        let comb_delays_ms = [29.7, 37.1, 41.1, 43.7];

        // Allpass delay times in ms
        let allpass_delays_ms = [5.0, 1.7];

        let combs = [
            CombFilter::new((comb_delays_ms[0] * sample_rate / 1000.0) as usize),
            CombFilter::new((comb_delays_ms[1] * sample_rate / 1000.0) as usize),
            CombFilter::new((comb_delays_ms[2] * sample_rate / 1000.0) as usize),
            CombFilter::new((comb_delays_ms[3] * sample_rate / 1000.0) as usize),
        ];

        let allpasses = [
            AllpassFilter::new((allpass_delays_ms[0] * sample_rate / 1000.0) as usize),
            AllpassFilter::new((allpass_delays_ms[1] * sample_rate / 1000.0) as usize),
        ];

        Self { combs, allpasses }
    }

    /// Configure delay times for a specific sample rate (RT-safe, no allocation).
    ///
    /// Call this when sample rate changes or on first render.
    pub fn configure(&mut self, sample_rate: f32) {
        let comb_delays_ms = [29.7, 37.1, 41.1, 43.7];
        let allpass_delays_ms = [5.0, 1.7];

        for (comb, &delay_ms) in self.combs.iter_mut().zip(comb_delays_ms.iter()) {
            comb.set_delay((delay_ms * sample_rate / 1000.0) as usize);
        }
        for (allpass, &delay_ms) in self.allpasses.iter_mut().zip(allpass_delays_ms.iter()) {
            allpass.set_delay((delay_ms * sample_rate / 1000.0) as usize);
        }
    }

    /// Set the room size (scales feedback for longer/shorter decay)
    pub fn set_room_size(&mut self, size: f32) {
        let feedback = 0.7 + size.clamp(0.0, 1.0) * 0.28; // 0.7 to 0.98
        for comb in &mut self.combs {
            comb.set_feedback(feedback);
        }
    }

    /// Set damping (high frequency absorption)
    pub fn set_damping(&mut self, damp: f32) {
        for comb in &mut self.combs {
            comb.set_damp(damp.clamp(0.0, 1.0));
        }
    }

    /// Process a single sample through the reverb
    pub fn process(&mut self, input: f32) -> f32 {
        // Sum outputs of all comb filters (parallel)
        let mut output = 0.0;
        for comb in &mut self.combs {
            output += comb.process(input);
        }
        output *= 0.25; // Normalize for 4 combs

        // Pass through allpass filters (series)
        for allpass in &mut self.allpasses {
            output = allpass.process(output);
        }

        output
    }

    /// Reset all filter states
    pub fn reset(&mut self) {
        for comb in &mut self.combs {
            comb.reset();
        }
        for allpass in &mut self.allpasses {
            allpass.reset();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_comb_filter_creates_echo() {
        let mut comb = CombFilter::new(10);
        comb.set_feedback(0.5);
        comb.set_damp(0.0);

        // Feed an impulse
        let out1 = comb.process(1.0);
        assert!(out1.abs() < 0.01); // No output yet (delayed)

        // Process 9 more samples of silence
        for _ in 0..9 {
            comb.process(0.0);
        }

        // Now we should see the echo
        let echo = comb.process(0.0);
        assert!(echo.abs() > 0.4); // Should have significant output
    }

    #[test]
    fn test_allpass_preserves_energy() {
        let mut allpass = AllpassFilter::new(5);
        allpass.set_feedback(0.5);

        // Process some samples
        let mut energy_in = 0.0;
        let mut energy_out = 0.0;

        for i in 0..100 {
            let input = if i < 10 { 1.0 } else { 0.0 };
            let output = allpass.process(input);
            energy_in += input * input;
            energy_out += output * output;
        }

        // Allpass should preserve most energy (within reason)
        assert!(energy_out > energy_in * 0.8);
    }

    #[test]
    fn test_schroeder_reverb_produces_output() {
        let mut reverb = SchroederReverb::new(48000.0);
        reverb.set_room_size(0.5);
        reverb.set_damping(0.5);

        // Feed an impulse
        let _ = reverb.process(1.0);

        // Process silence - need enough samples for delay lines to output
        // Longest comb delay is ~43ms = ~2100 samples at 48kHz
        let mut has_tail = false;
        for _ in 0..5000 {
            let out = reverb.process(0.0);
            if out.abs() > 0.001 {
                has_tail = true;
                break;
            }
        }

        assert!(has_tail, "Reverb should produce a tail after impulse");
    }

    #[test]
    fn test_reverb_stability() {
        let mut reverb = SchroederReverb::new(48000.0);
        reverb.set_room_size(1.0); // Maximum feedback

        // Process many samples and ensure no explosion
        for _ in 0..10000 {
            let out = reverb.process(0.1);
            assert!(out.is_finite(), "Reverb output should be finite");
            assert!(out.abs() < 10.0, "Reverb output unstable: {}", out);
        }
    }
}
