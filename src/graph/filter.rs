use crate::{dsp::filter::SVFilter, graph::node::GraphNode};

pub struct FilterNode {
    filter: SVFilter,
}

impl FilterNode {
    pub fn lowpass(cutoff_hz: f32) -> Self {
        let filter = SVFilter::lowpass(cutoff_hz);

        FilterNode { filter: filter }
    }

    pub fn highpass(cutoff_hz: f32) -> Self {
        let filter = SVFilter::highpass(cutoff_hz);

        FilterNode { filter: filter }
    }
}

impl GraphNode for FilterNode {
    fn render_block(&mut self, out: &mut [f32], ctx: &super::node::RenderCtx) {
        self.filter.render(out, ctx);
    }
}
