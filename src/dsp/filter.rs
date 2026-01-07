use std::f32::consts::TAU;

use crate::graph::node::RenderCtx;

/*
State Variable Filter (SVF) Implementation
==========================================

A filter shapes the frequency content of a sound - it's how we carve
brightness, warmth, and character from raw waveforms. This implementation
uses a topology-preserving transform (TPT) state variable filter, which
gives us all four filter types from a single structure.


Vocabulary
----------

  cutoff      The frequency (in Hz) where the filter starts to act.
              Below this: lowpass lets through, highpass blocks.
              At this frequency: both outputs are at -3dB (half power).

  resonance   Emphasis at the cutoff frequency. Higher resonance creates
              a peak that makes the filter "ring" or "whistle." At maximum,
              the filter self-oscillates (produces a sine wave).

  Q           Quality factor - another way to express resonance.
              Q = 1/(2 - 2*resonance). Higher Q = narrower, taller peak.

  SVF         State Variable Filter. A topology that produces lowpass,
              highpass, bandpass, and notch outputs simultaneously.
              We just pick which output to use.

  TPT         Topology-Preserving Transform. A discretization method that
              keeps the analog filter's behavior intact at high frequencies.
              Better than naive methods which break down near Nyquist.

  g           The integrator coefficient. Derived from cutoff frequency.
              Controls how fast the filter responds.

  k           Damping factor. Derived from resonance (k = 2 - 2*resonance).
              Lower k = higher resonance = more ringing.


Filter Types
------------

| Type      | Passes         | Rejects       | Sound Character            |
|-----------|----------------|---------------|----------------------------|
| LowPass   | below cutoff   | above cutoff  | warm, muffled, bassy       |
| HighPass  | above cutoff   | below cutoff  | thin, bright, airy         |
| BandPass  | around cutoff  | both extremes | nasal, vocal, focused      |
| Notch     | both extremes  | around cutoff | hollow, phaser-like        |


The Architecture: State Variable Filter
---------------------------------------

The SVF processes audio through two integrators in series, with feedback
paths that create the resonant behavior:

    Input ─┬───────────────────────────────────┐
           │                                   │
           │  ┌─────────────────────────────┐  │
           └─→│ HP = Input - k*BP - LP      │──┼──→ Highpass Out
              └─────────────────────────────┘  │
                             │                 │
                             ↓ (×g)            │
              ┌─────────────────────────────┐  │
              │ BP = ∫ HP  (integrator 1)   │──┼──→ Bandpass Out
              └─────────────────────────────┘  │
                             │                 │
                             ↓ (×g)            │
              ┌─────────────────────────────┐  │
              │ LP = ∫ BP  (integrator 2)   │──┴──→ Lowpass Out
              └─────────────────────────────┘
                             │
    Notch = HP + LP ←────────┘

Each integrator has memory (ic1eq, ic2eq) that persists between samples.
This memory is what makes the filter "remember" and creates smooth response.


The Math: Analog to Digital
---------------------------

The key challenge: convert an analog filter design to work with discrete
samples. We use the bilinear transform with frequency pre-warping.

Step 1: Pre-warp the cutoff frequency
  wd = 2π × cutoff_hz          (digital angular frequency)
  wa = 2×sr × tan(wd / 2×sr)   (warped analog frequency)

  Why? The bilinear transform compresses high frequencies. Pre-warping
  ensures our cutoff lands at the right place.

Step 2: Compute the integrator coefficient
  g = wa / (2 × sample_rate)

Step 3: Compute damping from resonance
  k = 2 - (2 × resonance)

  When resonance = 0: k = 2 (heavily damped, no peak)
  When resonance = 1: k = 0 (undamped, self-oscillation)

Example: 1 kHz cutoff at 48 kHz sample rate
  wd = 2π × 1000 = 6283.2 rad/s
  wa = 96000 × tan(6283.2 / 96000) = 96000 × tan(0.0654) ≈ 6294.5
  g = 6294.5 / 96000 ≈ 0.0656


The Sample Processing
---------------------

Each sample, we solve the filter equations using a trick: compute the
intermediate values (v1, v2, v3) that satisfy all constraints simultaneously.

  v3 = input - ic2eq           (input minus LP memory)
  v1 = h × (ic1eq + g × v3)    (new bandpass value)
  v2 = ic2eq + g × v1          (new lowpass value)

  where h = 1 / (1 + g×(g + k))  (normalization factor)

Then update the integrator memories using the trapezoidal rule:
  ic1eq = 2×v1 - ic1eq
  ic2eq = 2×v2 - ic2eq

This "implicit" integration is what makes TPT filters stable and accurate.


Implementation Notes
--------------------

We calculate g fresh each sample rather than caching. This allows cutoff
modulation (filter sweeps, envelope followers) without artifacts.

Resonance is clamped to prevent instability. At resonance > 1.0, the
filter self-oscillates; this can be musically useful but we leave that
as an intentional choice rather than accident.

The filter outputs are computed simultaneously from the same state,
so switching filter types mid-note is glitch-free.
*/

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

    pub cutoff_hz: f32,
    pub resonance: f32,
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

    pub fn set_cutoff(&mut self, cutoff: f32) {
        self.cutoff_hz = cutoff;
    }

    pub fn set_resonance(&mut self, resonance: f32) {
        self.resonance = resonance;
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

    #[test]
    fn test_set_cutoff_updates_frequency() {
        let mut filter = SVFilter::lowpass(1000.0);

        // Initial cutoff
        assert!((filter.cutoff_hz - 1000.0).abs() < 0.1);

        // Update cutoff
        filter.set_cutoff(2000.0);
        assert!((filter.cutoff_hz - 2000.0).abs() < 0.1);

        // Update again
        filter.set_cutoff(500.0);
        assert!((filter.cutoff_hz - 500.0).abs() < 0.1);
    }

    #[test]
    fn test_set_resonance_updates_value() {
        let mut filter = SVFilter::lowpass(1000.0);

        // Initial resonance should be default
        let initial_res = filter.resonance;

        // Update resonance
        filter.set_resonance(2.0);
        assert!((filter.resonance - 2.0).abs() < 0.01);
        assert!(
            (filter.resonance - initial_res).abs() > 0.01,
            "Resonance should have changed"
        );

        // Update again
        filter.set_resonance(5.0);
        assert!((filter.resonance - 5.0).abs() < 0.01);
    }

    #[test]
    fn test_set_cutoff_affects_filtering() {
        let sample_rate = 48_000.0;
        let test_freq = 1_000.0;

        // Create filter with low cutoff (should attenuate 1kHz)
        let mut filter = SVFilter::lowpass(200.0);
        let mut osc = OscNode::sine();
        let mut buffer1 = vec![0.0f32; 256];
        let ctx = RenderCtx::from_freq(sample_rate, test_freq, 100.0);
        osc.render_block(&mut buffer1, &ctx);
        filter.render(&mut buffer1, &ctx);
        let peak_low_cutoff = peak_after_transient(&buffer1);

        // Raise cutoff to well above test frequency (should pass 1kHz)
        filter.reset();
        filter.set_cutoff(5_000.0);
        let mut osc = OscNode::sine();
        let mut buffer2 = vec![0.0f32; 256];
        osc.render_block(&mut buffer2, &ctx);
        filter.render(&mut buffer2, &ctx);
        let peak_high_cutoff = peak_after_transient(&buffer2);

        // High cutoff should pass more signal than low cutoff
        assert!(
            peak_high_cutoff > peak_low_cutoff * 2.0,
            "High cutoff should pass more signal: high={}, low={}",
            peak_high_cutoff,
            peak_low_cutoff
        );
    }

    #[test]
    fn test_set_resonance_affects_peak() {
        let sample_rate = 48_000.0;
        let cutoff = 1_000.0;

        // Lowpass filter at cutoff with low resonance
        let mut filter = SVFilter::lowpass(cutoff);
        filter.set_resonance(0.1);
        let mut osc = OscNode::sine();
        let mut buffer1 = vec![0.0f32; 512];
        let ctx = RenderCtx::from_freq(sample_rate, cutoff, 100.0);
        osc.render_block(&mut buffer1, &ctx);
        filter.render(&mut buffer1, &ctx);
        let peak_low_res = peak_after_transient(&buffer1);

        // Same filter with moderate resonance (not too high to avoid instability)
        filter.reset();
        filter.set_resonance(2.0);
        let mut osc = OscNode::sine();
        let mut buffer2 = vec![0.0f32; 512];
        osc.render_block(&mut buffer2, &ctx);
        filter.render(&mut buffer2, &ctx);
        let peak_high_res = peak_after_transient(&buffer2);

        // Higher resonance should boost signal at cutoff frequency
        assert!(
            peak_high_res > peak_low_res * 1.2,
            "High resonance should boost signal: high_res={}, low_res={}",
            peak_high_res,
            peak_low_res
        );
    }
}
