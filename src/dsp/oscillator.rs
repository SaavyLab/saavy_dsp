#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy)]
pub enum OscillatorWaveform {
    Sine,
    Saw,
    Square,
    Noise,
}

pub struct OscillatorBlock;

impl OscillatorBlock {
    pub fn new() -> Self {
        Self
    }

    pub fn render(&mut self, _destination: &mut [f32]) {
        todo!("fill buffer with oscillator output")
    }
}
