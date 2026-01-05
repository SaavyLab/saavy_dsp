use crate::graph::{
    amplify::{Amplify, Gain},
    mix::Mix,
    modulate::Modulate,
    node::{GraphNode, Modulatable},
    through::Through,
};

pub trait NodeExt: GraphNode + Sized {
    /// Multiply by another signal (envelope, LFO, ring mod)
    fn amplify<M>(self, modulator: M) -> Amplify<Self, M> {
        Amplify::new(self, modulator)
    }

    /// Apply constant gain (volume control)
    ///
    /// Common values: 0.5 = -6dB, 1.0 = unity, 2.0 = +6dB
    fn gain(self, gain: f32) -> Gain<Self> {
        Gain::new(self, gain)
    }

    fn through<F: GraphNode>(self, filter: F) -> Through<Self, F> {
        Through::new(self, filter)
    }

    fn modulate<M: GraphNode>(self, lfo: M, param: Self::Param, depth: f32) -> Modulate<Self, M>
    where
        Self: Modulatable,
    {
        Modulate::new(self, lfo, param, depth)
    }

    fn mix<M: GraphNode>(self, source: M, balance: f32) -> Mix<Self, M> {
        Mix::new(self, source, balance)
    }
}

impl<T: GraphNode> NodeExt for T {}
