use crate::graph::{amplify::Amplify, node::GraphNode};

pub trait NodeExt: GraphNode + Sized {
    fn amplify<M>(self, modulator: M) -> Amplify<Self, M> {
        Amplify::new(self, modulator)
    }
}

impl<T: GraphNode> NodeExt for T {}
