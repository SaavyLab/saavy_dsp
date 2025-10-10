#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy)]
pub enum FilterType {
    LowPass,
    HighPass,
    BandPass,
}

pub struct FilterBlock;

impl FilterBlock {
    pub fn new() -> Self {
        Self
    }

    pub fn process(&mut self, _buffer: &mut [f32]) {
        todo!("apply filter to buffer")
    }
}
