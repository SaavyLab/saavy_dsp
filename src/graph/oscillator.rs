use crate::dsp::oscillator::OscillatorBlock;
use crate::graph::node::{GraphNode, RenderCtx};

pub struct OscNode {
    osc: OscillatorBlock,
}

impl OscNode {
    pub fn sine() -> Self {
        let osc = OscillatorBlock::sine();
        Self { osc }
    }

    pub fn sawtooth() -> Self {
        let osc = OscillatorBlock::sawtooth();
        Self { osc }
    }

    pub fn square() -> Self {
        let osc = OscillatorBlock::square();
        Self { osc }
    }
}

impl GraphNode for OscNode {
    fn render_block(&mut self, out: &mut [f32], ctx: &RenderCtx) {
        self.osc.render(out, ctx);
    }
}

#[cfg(test)]
mod tests {
    use crate::graph::node::RenderCtx;

    use super::*;
    use std::f32::consts::TAU;

    #[test]
    fn valid_sine() {
        let sample_rate = 48_000.0;
        let block_size = 128;
        let note = 69; // MIDI note 69 = A4 = 440Hz

        let ctx = RenderCtx::from_note(sample_rate, note, 100.0);
        let mut synth = OscNode::sine();

        let mut buffer = vec![0.0f32; block_size];
        synth.render_block(&mut buffer, &ctx);

        // sample n should be sin(2pi f n / sr), where f = 440Hz (MIDI 69)
        let sample_index = 12;
        let expected = (TAU * ctx.frequency * sample_index as f32 / sample_rate).sin();
        let actual = buffer[sample_index];
        assert!(
            (actual - expected).abs() < 1e-6,
            "expected {expected}, got {actual}"
        );
    }
}
