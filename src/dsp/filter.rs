use std::f32::consts::TAU;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::graph::node::RenderCtx;

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy)]
pub enum FilterType {
    LowPass,
    HighPass,
    BandPass,
    Notch,
}

pub struct FilterOutputs {
    pub lowpass: f32,
    pub bandpass: f32,
    pub highpass: f32,
    pub notch: f32,
}

pub struct SVFilter {
    ic1eq: f32, // First integrator's memory
    ic2eq: f32, // Second integrator's memory

    cutoff_hz: f32,
    resonance: f32,
    filter_type: FilterType,
}

impl SVFilter {
    pub fn new(filter_type: FilterType) -> Self {
        Self {
            ic1eq: 0.0,
            ic2eq: 0.0,
            cutoff_hz: 1000.0,
            resonance: 0.0,
            filter_type,
        }
    }

    pub fn lowpass(cutoff_hz: f32) -> Self {
        Self {
            ic1eq: 0.0,
            ic2eq: 0.0,
            cutoff_hz,
            resonance: 0.0,
            filter_type: FilterType::LowPass,
        }
    }

    pub fn highpass(cutoff_hz: f32) -> Self {
        Self {
            ic1eq: 0.0,
            ic2eq: 0.0,
            cutoff_hz,
            resonance: 0.0,
            filter_type: FilterType::HighPass,
        }
    }

    #[inline]
    fn compute_g(&self, ctx: &RenderCtx) -> f32 {
        let wd = TAU * self.cutoff_hz;
        let wa = (2.0 * ctx.sample_rate) * (wd / (2.0 * ctx.sample_rate)).tan();
        wa / (2.0 * ctx.sample_rate)
    }

    pub fn next_sample(&mut self, sample: f32, k: f32, g: f32) -> FilterOutputs {
        let h = 1.0 / (1.0 + g * (g + k));
        let v3 = sample - self.ic2eq;
        let v1 = h * (self.ic1eq + g * v3);
        let v2 = self.ic2eq + g * v1;

        self.ic1eq = 2.0 * v1 - self.ic1eq;
        self.ic2eq = 2.0 * v2 - self.ic2eq;

        FilterOutputs {
            lowpass: v2,
            bandpass: v1,
            highpass: sample - k * v1 - v2,
            notch: sample - k * v1,
        }
    }

    pub fn render(&mut self, buffer: &mut [f32], ctx: &RenderCtx) {
        let g = self.compute_g(ctx);
        let k = 2.0 - (2.0 * self.resonance);

        for sample in buffer.iter_mut() {
            let outputs = self.next_sample(*sample, k, g);

            *sample = match self.filter_type {
                FilterType::LowPass => outputs.lowpass,
                FilterType::HighPass => outputs.highpass,
                FilterType::BandPass => outputs.bandpass,
                FilterType::Notch => outputs.notch,
            }
        }
    }

    pub fn reset(&mut self) {
        self.ic1eq = 0.0;
        self.ic2eq = 0.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lowpass_basic() {
        let mut filter = SVFilter::lowpass(500.0);
        let mut buffer = vec![1.0; 128];
        let ctx = RenderCtx::from_freq(48_000.0, 440.0, 100.0);

        filter.render(&mut buffer, &ctx);

        println!("Buffer at 127: {}", buffer[127]);
        assert!(buffer[127] > 0.99);
    }

    #[test]
    fn test_highpass_basic() {
        let mut filter = SVFilter::highpass(500.0);
        let mut buffer = vec![1.0; 128];
        let ctx = RenderCtx::from_freq(48_000.0, 440.0, 100.0);

        filter.render(&mut buffer, &ctx);

        println!("Buffer at 127: {}", buffer[127]);
        assert!(buffer[127] < 0.001);
    }
}
