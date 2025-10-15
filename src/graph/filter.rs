use crate::{
    dsp::filter::SVFilter,
    graph::node::{GraphNode, Modulatable},
};

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
