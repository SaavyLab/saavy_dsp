#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::f32::consts::PI;
use std::f32::consts::TAU;

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy)]
pub enum OscillatorWaveform {
    Sine,
    Saw,
    Square,
    Noise,
}

pub struct OscillatorBlock {
    phase: f32,
    frequency: f32,
    sample_rate: f32,
    waveform: OscillatorWaveform,
    duty: f32,
    rng: u32,
}

impl OscillatorBlock {
    pub fn new(frequency: f32, sample_rate: f32, waveform: OscillatorWaveform) -> Self {
        Self {
            phase: 0.0,
            frequency,
            sample_rate,
            waveform,
            duty: 0.5,
            rng: 0x9E3779B9
        }
    }

    pub fn set_frequency(&mut self, frequency: f32) {
        self.frequency = frequency.max(0.0);
    }

    pub fn set_waveform(&mut self, waveform: OscillatorWaveform) {
        self.waveform = waveform;
    }

    pub fn set_duty(&mut self, duty: f32) {
        self.duty = duty.clamp(0.0, 1.0);
    }

    /*
        we get handed a writable list of floats (the audio buffer)
        compute how fast the wave should progress per sample (phase_inc)
        walk through each slot, write the sample value for our current phase
        advance phase, and wrap it when we complete a 2π cycle
    */
    pub fn render(&mut self, buffer: &mut [f32], amp: f32) {
        if self.sample_rate <= 0.0 {
            buffer.fill(0.0);
            return;
        }

        let phase_inc: f32 = 2.0 * PI * self.frequency / self.sample_rate;

        for sample in buffer.iter_mut() {
            let y = match self.waveform {
                OscillatorWaveform::Sine => self.phase.sin(),
                // normalized phase in [0,1]: phi = phase / TAU
                OscillatorWaveform::Saw => {
                    let phi = self.phase / TAU;
                    (2.0 * phi) - 1.0
                },
                OscillatorWaveform::Square => {
                    // compare normalized phase to duty
                    let phi = self.phase / TAU;
                    if phi < self.duty { 1.0 } else { -1.0 }
                },
                OscillatorWaveform::Noise => self.next_noise()
            };

            *sample = amp * y;

            self.phase = (self.phase + phase_inc).rem_euclid(TAU);
        }
    }

    #[inline]
    /*
        pseudo-random number generator xorshift32
        Original u32:  11010110 10110101 01101011 10110110 (32 bits, random)
                        ^^^^^^^^^ ← throw away top 9 bits
                
        After >> 9:    00000000 01101011 01011010 1101011 (23 bits remain)
                                 ^^^^^^^^^^^^^^^^^^^^^^^ these become your random float

        Divide by 2^23: Normalizes to 0.0 - 1.0
        Times 2.0:      Scales to 0.0 - 2.0  
        Minus 1.0:      Shifts to -1.0 - 1.0 ✅
    */
    pub fn next_noise(&mut self) -> f32 {
        let mut x = self.rng;
        // xorshift32 -> map to [-1, 1]
        x ^= x << 13; 
        x ^= x >> 17; 
        x ^= x << 5;
        self.rng = x;

        // take upper bits, scale to float in [-1, 1]
        let u = (x >> 9) as f32 / 8388608.0; // 23-bit mantissa scale
        u * 2.0 - 1.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn phase_wrapping() {
        let mut osc = OscillatorBlock::new(440.0, 48_000.0, OscillatorWaveform::Sine);
        let mut buffer = [0.0; 128];
        osc.render(&mut buffer, 1.0);
        assert!(osc.phase < TAU);
    }
}