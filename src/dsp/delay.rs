use crate::MAX_DELAY_SAMPLES;

/*
Delay Line - Circular Buffer for Audio Delay
=============================================

A delay line is a ring buffer that stores audio samples and allows reading
from the past. This is the building block for echo, reverb, chorus, and
other time-based effects.

The buffer wraps around (circular/ring buffer) so we can continuously write
without running out of space.
*/

pub struct DelayLine {
    buffer: Vec<f32>,
    write_pos: usize,
}

impl DelayLine {
    pub fn new() -> Self {
        Self {
            buffer: vec![0.0; MAX_DELAY_SAMPLES],
            write_pos: 0,
        }
    }

    /// Read a delayed sample without advancing write position
    /// Uses linear interpolation for smooth modulation
    pub fn read(&self, delay_samples: usize) -> f32 {
        let delay_samples = delay_samples.min(MAX_DELAY_SAMPLES - 1);
        let read_pos = (self.write_pos + MAX_DELAY_SAMPLES - delay_samples) % MAX_DELAY_SAMPLES;
        self.buffer[read_pos]
    }

    /// Read a delayed sample with fractional delay (interpolated)
    /// This is critical for smooth delay time modulation (chorus/flanger effects)
    pub fn read_interpolated(&self, delay_samples_float: f32) -> f32 {
        // Clamp delay to valid range
        let delay_clamped = delay_samples_float.clamp(1.0, (MAX_DELAY_SAMPLES - 2) as f32);

        // Split into integer and fractional parts
        let delay_int = delay_clamped.floor() as usize;
        let frac = delay_clamped - delay_int as f32;

        // Calculate read positions for interpolation
        let read_pos1 = (self.write_pos + MAX_DELAY_SAMPLES - delay_int) % MAX_DELAY_SAMPLES;
        let read_pos2 = (read_pos1 + MAX_DELAY_SAMPLES - 1) % MAX_DELAY_SAMPLES;

        // Linear interpolation between two samples
        let sample1 = self.buffer[read_pos1];
        let sample2 = self.buffer[read_pos2];

        sample1 * (1.0 - frac) + sample2 * frac
    }

    /// Write a sample and advance write position
    pub fn write(&mut self, sample: f32) {
        self.buffer[self.write_pos] = sample;
        self.write_pos = (self.write_pos + 1) % MAX_DELAY_SAMPLES;
    }

    /// Convenience: write sample, then read delayed sample
    /// This is the most common operation for simple delays
    pub fn next_sample(&mut self, sample: f32, delay_samples: usize) -> f32 {
        let delayed = self.read(delay_samples);
        self.write(sample);
        delayed
    }

    /// Process a block of samples through the delay line
    /// This only delays - mixing and feedback happen at a higher level
    pub fn render(&mut self, buffer: &mut [f32], delay_samples: usize) {
        for sample in buffer.iter_mut() {
            *sample = self.next_sample(*sample, delay_samples);
        }
    }

    pub fn reset(&mut self) {
        self.buffer.fill(0.0);
        self.write_pos = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_delay_line_basic() {
        let mut delay = DelayLine::new();

        // Write a few samples
        delay.write(1.0);
        delay.write(2.0);
        delay.write(3.0);

        // Read with 3 sample delay - should get first sample we wrote
        // (write_pos is now at 3, so read(3) goes back to position 0)
        let delayed = delay.read(3);
        assert!((delayed - 1.0).abs() < 0.001, "Expected 1.0, got {}", delayed);
    }

    #[test]
    fn test_delay_line_wraps_around() {
        let mut delay = DelayLine::new();

        // Fill more than a few samples
        for i in 0..100 {
            delay.write(i as f32);
        }

        // After writing 100 samples, write_pos is at 100
        // read(10) should give us sample at position 90 (100 - 10)
        let delayed = delay.read(10);
        assert!((delayed - 90.0).abs() < 0.001, "Expected 90.0, got {}", delayed);
    }

    #[test]
    fn test_delay_line_interpolation() {
        let mut delay = DelayLine::new();

        // Write known pattern
        delay.write(0.0);
        delay.write(10.0);
        delay.write(20.0);

        // Read with fractional delay (1.5 samples)
        // Should interpolate between sample at pos 1 (10.0) and pos 2 (20.0)
        // With frac=0.5: 10.0 * 0.5 + 20.0 * 0.5 = 15.0
        let delayed = delay.read_interpolated(1.5);
        assert!((delayed - 15.0).abs() < 0.1, "Expected ~15.0, got {}", delayed);
    }

    #[test]
    fn test_delay_line_interpolation_edges() {
        let mut delay = DelayLine::new();

        // Write pattern
        delay.write(100.0);
        delay.write(200.0);

        // After writing 2 samples, write_pos is at 2
        // read_interpolated(2.0) should give us sample at position 0 (100.0)
        let delayed_int = delay.read_interpolated(2.0);
        assert!((delayed_int - 100.0).abs() < 0.1, "Integer delay should be exact, got {}", delayed_int);

        // Fractional delay 1.5 should interpolate between pos 0 and pos 1
        let delayed_frac = delay.read_interpolated(1.5);
        assert!(delayed_frac > 100.0 && delayed_frac < 200.0,
                "Fractional delay should be between samples, got {}", delayed_frac);
    }

    #[test]
    fn test_delay_line_next_sample() {
        let mut delay = DelayLine::new();

        // next_sample writes then reads
        let delayed1 = delay.next_sample(1.0, 1);
        assert!(delayed1.abs() < 0.001, "First read should be ~0 (empty buffer)");

        let delayed2 = delay.next_sample(2.0, 1);
        assert!((delayed2 - 1.0).abs() < 0.001, "Should read what was written 1 sample ago");

        let delayed3 = delay.next_sample(3.0, 1);
        assert!((delayed3 - 2.0).abs() < 0.001, "Should read previous sample");
    }

    #[test]
    fn test_delay_line_reset() {
        let mut delay = DelayLine::new();

        // Fill with data
        for i in 0..10 {
            delay.write(i as f32);
        }

        // Reset
        delay.reset();

        // Read should return zeros
        let delayed = delay.read(5);
        assert!(delayed.abs() < 0.001, "After reset, buffer should be zeroed");
    }

    #[test]
    fn test_delay_line_max_delay_clamping() {
        let mut delay = DelayLine::new();

        delay.write(1.0);
        delay.write(2.0);

        // Request delay longer than max - should clamp
        let delayed = delay.read(MAX_DELAY_SAMPLES + 1000);
        // Should not panic and should return a valid sample
        assert!(delayed.is_finite(), "Should clamp and return finite value");
    }

    #[test]
    fn test_delay_line_zero_delay() {
        let mut delay = DelayLine::new();

        delay.write(1.0);
        delay.write(2.0);

        // Zero delay should give very recent sample
        let delayed = delay.read(0);
        assert!(delayed.is_finite(), "Zero delay should work");
    }
}
