use crate::graph::node::GraphNode;

/// Factory for creating voices with a specific patch/sound design
///
/// This is the "instrument design" layer - you configure your sound once,
/// then PolySynth uses this factory to create identical voices.
pub trait VoiceFactory: Send {
    type Voice: GraphNode;

    fn create_voice(&self) -> Self::Voice;
}

impl<F, T> VoiceFactory for F
where
    F: Fn() -> T + Send,
    T: GraphNode,
{
    type Voice = T;

    fn create_voice(&self) -> Self::Voice {
        self()
    }
}
