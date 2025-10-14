use crate::graph::{amplify::Amplify, node::GraphNode, through::Through};

pub trait NodeExt: GraphNode + Sized {
    fn amplify<M>(self, modulator: M) -> Amplify<Self, M> {
        Amplify::new(self, modulator)
    }

    fn through<F: GraphNode>(self, filter: F) -> Through<Self, F> {
        Through::new(self, filter)
    }
}

impl<T: GraphNode> NodeExt for T {}
