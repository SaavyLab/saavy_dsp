use crate::MAX_DELAY_SAMPLES;

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

    pub fn next_sample(&mut self, sample: f32, delay_samples: usize) -> f32 {
        let delay_samples = delay_samples.min(MAX_DELAY_SAMPLES - 1);

        self.buffer[self.write_pos] = sample;

        let read_pos = (self.write_pos + MAX_DELAY_SAMPLES - delay_samples) % MAX_DELAY_SAMPLES;

        let delayed = self.buffer[read_pos];

        self.write_pos = (self.write_pos + 1) % MAX_DELAY_SAMPLES;

        delayed
    }

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
