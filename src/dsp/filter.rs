use std::f32::consts::TAU;

use crate::graph::node::RenderCtx;

/*
| type              | constructed by       | passes          | rejects      |
| ----------------- | -------------------- | --------------- | ------------ |
| low-pass          | LPF                  | below cutoff    | above cutoff |
| high-pass         | HPF                  | above cutoff    | below cutoff |
| band-pass         | LPF ∘ HPF (series)   | between cutoffs | outside      |
| notch / band-stop | LPF + HPF (parallel) | outside         | between      |
*/

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

    pub fn bandpass(cutoff_hz: f32) -> Self {
        Self {
            ic1eq: 0.0,
            ic2eq: 0.0,
            cutoff_hz,
            resonance: 0.0,
            filter_type: FilterType::BandPass,
        }
    }

    pub fn notch(cutoff_hz: f32) -> Self {
        Self {
            ic1eq: 0.0,
            ic2eq: 0.0,
            cutoff_hz,
            resonance: 0.0,
            filter_type: FilterType::Notch,
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
    use crate::graph::node::GraphNode;
    use crate::graph::oscillator::OscNode;

    fn peak_after_transient(buffer: &[f32]) -> f32 {
        let skip = buffer.len().min(32);
        buffer
            .get(skip..)
            .unwrap_or(buffer)
            .iter()
            .fold(0.0f32, |acc, &x| acc.max(x.abs()))
    }

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

    #[test]
    fn test_lowpass_filters_high_freq() {
        let mut filter = SVFilter::lowpass(500.0);
        let sample_rate = 48_000.0;
        let freq = 5_000.0;
        let ctx = RenderCtx::from_freq(sample_rate, freq, 100.0); // 10x cutoff

        // Generate sine wave via OscNode to match runtime usage
        let mut osc = OscNode::sine();
        let mut buffer = vec![0.0f32; 128];
        osc.render_block(&mut buffer, &ctx);

        filter.render(&mut buffer, &ctx);

        // After filtering, high freq should be attenuated by ~12dB/octave (≈3.3x reduction)
        let peak = buffer.iter().fold(0.0f32, |acc, &x| acc.max(x.abs()));
        assert!(
            peak < 0.3,
            "Expected high freq attenuation, got peak: {}",
            peak
        );
    }

    #[test]
    fn test_bandpass_emphasizes_cutoff_frequency() {
        let sample_rate = 48_000.0;
        let cutoff = 1_000.0;

        let mut filter = SVFilter::new(FilterType::BandPass);
        filter.cutoff_hz = cutoff;
        filter.resonance = 0.5;

        let mut osc_pass = OscNode::sine();
        let mut pass_buffer = vec![0.0f32; 512];
        let ctx_pass = RenderCtx::from_freq(sample_rate, cutoff, 100.0);
        osc_pass.render_block(&mut pass_buffer, &ctx_pass);
        filter.render(&mut pass_buffer, &ctx_pass);
        let pass_peak = peak_after_transient(&pass_buffer);

        filter.reset();
        let mut osc_off = OscNode::sine();
        let mut off_buffer = vec![0.0f32; 512];
        let ctx_off = RenderCtx::from_freq(sample_rate, 200.0, 100.0);
        osc_off.render_block(&mut off_buffer, &ctx_off);
        filter.render(&mut off_buffer, &ctx_off);
        let off_peak = peak_after_transient(&off_buffer);

        assert!(
            pass_peak > off_peak * 2.0,
            "expected bandpass to emphasize cutoff freq, got pass_peak={}, off_peak={}",
            pass_peak,
            off_peak
        );
    }

    #[test]
    fn test_notch_rejects_cutoff_frequency() {
        let sample_rate = 48_000.0;
        let cutoff = 1_000.0;

        let mut filter = SVFilter::new(FilterType::Notch);
        filter.cutoff_hz = cutoff;
        filter.resonance = 0.5;

        let mut osc_center = OscNode::sine();
        let mut center_buffer = vec![0.0f32; 512];
        let ctx_center = RenderCtx::from_freq(sample_rate, cutoff, 100.0);
        osc_center.render_block(&mut center_buffer, &ctx_center);
        filter.render(&mut center_buffer, &ctx_center);
        let center_peak = peak_after_transient(&center_buffer);

        filter.reset();
        let mut osc_off = OscNode::sine();
        let mut off_buffer = vec![0.0f32; 512];
        let ctx_off = RenderCtx::from_freq(sample_rate, 200.0, 100.0);
        osc_off.render_block(&mut off_buffer, &ctx_off);
        filter.render(&mut off_buffer, &ctx_off);
        let off_peak = peak_after_transient(&off_buffer);

        assert!(
            center_peak * 2.0 < off_peak,
            "expected notch to reject center freq, got center_peak={}, off_peak={}",
            center_peak,
            off_peak
        );
    }
}
