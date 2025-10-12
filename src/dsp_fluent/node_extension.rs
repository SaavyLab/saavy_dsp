use crate::dsp_fluent::{amplify::Amplify, voice_node::VoiceNode};

pub trait NodeExt: VoiceNode + Sized {
  fn amplify<M>(self, modulator: M) -> Amplify<Self, M> {
    Amplify::new(self, modulator)
  }
}

impl<T: VoiceNode> NodeExt for T {}