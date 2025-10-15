use crate::graph::{amplify::Amplify, modulate::Modulate, node::{GraphNode, Modulatable}, through::Through};

pub trait NodeExt: GraphNode + Sized {
    fn amplify<M>(self, modulator: M) -> Amplify<Self, M> {
        Amplify::new(self, modulator)
    }

    fn through<F: GraphNode>(self, filter: F) -> Through<Self, F> {
        Through::new(self, filter)
    }

    fn modulate<M: GraphNode>(self, lfo: M, param: Self::Param, depth: f32) -> Modulate<Self, M> 
    where
        Self: Modulatable
    {
        Modulate::new(self, lfo, param, depth)
    }
}

impl<T: GraphNode> NodeExt for T {}
