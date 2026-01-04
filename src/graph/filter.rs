use crate::{
    dsp::filter::SVFilter,
    graph::node::{GraphNode, Modulatable},
};

/*
State-Variable Filter (SVF)
===========================

A filter removes or attenuates certain frequencies from a signal. In subtractive
synthesis, you start with a harmonically rich waveform (like a sawtooth) and
filter out frequencies to sculpt the timbre. This is why it's called
"subtractive" - you're subtracting harmonics.

Filter Types:
-------------

Lowpass (LP): Passes frequencies BELOW the cutoff, attenuates above.
  - The most common synth filter
  - Higher cutoff = brighter sound
  - Lower cutoff = darker, muffled sound
  - Use: Basses, pads, "underwater" effects

Highpass (HP): Passes frequencies ABOVE the cutoff, attenuates below.
  - Removes low-end rumble and muddiness
  - Creates thin, airy sounds
  - Use: Hi-hats, leads, clearing mix space

Bandpass (BP): Passes frequencies AROUND the cutoff, attenuates both sides.
  - Creates a focused, "telephone" quality
  - Sweeping bandpass = classic wah effect
  - Use: Vocal-like sounds, wah effects, isolating frequencies

Notch: Attenuates frequencies AT the cutoff, passes everything else.
  - Creates a "hollow" sound at the notch frequency
  - Opposite of bandpass
  - Use: Phaser effects, removing resonant frequencies

Parameters:
-----------

Cutoff (Hz): The frequency where the filter takes effect.
  - 20 Hz:     Barely open (very dark)
  - 200 Hz:    Muffled, like through a wall
  - 1000 Hz:   Warm, round bass
  - 5000 Hz:   Present, clear
  - 20000 Hz:  Fully open (no filtering)

Resonance (Q): Emphasis at the cutoff frequency.
  - 0.0:  No emphasis (gentle rolloff)
  - 0.5:  Slight peak (adds character)
  - 1.0+: Strong peak (aggressive, "squelchy")
  - High: Self-oscillation (filter becomes a sine oscillator!)

Why "State-Variable"?
---------------------
The SVF is a specific filter topology that's popular in synthesizers because:
1. It provides LP, HP, BP, and Notch outputs simultaneously
2. It's stable and well-behaved at high resonance
3. Cutoff and resonance are independently controllable
4. It uses a "TPT" (topology-preserving transform) for digital accuracy

Example usage:
  // Basic lowpass
  let dark = OscNode::sawtooth().through(FilterNode::lowpass(800.0));

  // Modulated filter (auto-wah)
  let lfo = LfoNode::sine(2.0);
  let wah = FilterNode::lowpass(1000.0)
      .modulate(lfo, FilterParam::Cutoff, 800.0);

  // Envelope-controlled filter (classic synth sound)
  let filter = OscNode::sawtooth()
      .through(FilterNode::lowpass(500.0));
  // (Apply envelope modulation to sweep the cutoff)
*/

#[derive(Clone, Copy, Debug)]
pub enum FilterParam {
    Cutoff,
    Resonance,
}

pub struct FilterNode {
    filter: SVFilter,
    base_cutoff: f32,
    base_resonance: f32,
}

impl FilterNode {
    pub fn lowpass(cutoff_hz: f32) -> Self {
        let filter = SVFilter::lowpass(cutoff_hz);

        FilterNode {
            filter,
            base_cutoff: cutoff_hz,
            base_resonance: 0.0,
        }
    }

    pub fn highpass(cutoff_hz: f32) -> Self {
        let filter = SVFilter::highpass(cutoff_hz);

        FilterNode {
            filter,
            base_cutoff: cutoff_hz,
            base_resonance: 0.0,
        }
    }

    pub fn bandpass(cutoff_hz: f32) -> Self {
        let filter = SVFilter::bandpass(cutoff_hz);

        FilterNode {
            filter,
            base_cutoff: cutoff_hz,
            base_resonance: 0.0,
        }
    }

    pub fn notch(cutoff_hz: f32) -> Self {
        let filter = SVFilter::notch(cutoff_hz);

        FilterNode {
            filter,
            base_cutoff: cutoff_hz,
            base_resonance: 0.0,
        }
    }

    // Test-only getters to verify modulation behavior
    #[cfg(test)]
    pub fn get_base_cutoff(&self) -> f32 {
        self.base_cutoff
    }

    #[cfg(test)]
    pub fn get_base_resonance(&self) -> f32 {
        self.base_resonance
    }
}

impl Modulatable for FilterNode {
    type Param = FilterParam;

    fn get_param(&self, param: Self::Param) -> f32 {
        match param {
            FilterParam::Cutoff => self.base_cutoff,
            FilterParam::Resonance => self.base_resonance,
        }
    }

    fn apply_modulation(&mut self, param: Self::Param, base: f32, modulation: f32) {
        let final_value = base + modulation;
        match param {
            FilterParam::Cutoff => {
                self.base_cutoff = base;
                self.filter.set_cutoff(final_value.clamp(20.0, 20_000.0));
            }
            FilterParam::Resonance => {
                self.base_resonance = base;
                self.filter.set_resonance(final_value.clamp(0.0, 10.0));
            }
        }
    }
}

impl GraphNode for FilterNode {
    fn render_block(&mut self, out: &mut [f32], ctx: &super::node::RenderCtx) {
        self.filter.render(out, ctx);
    }
}
