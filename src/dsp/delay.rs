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
