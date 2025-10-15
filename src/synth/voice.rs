use crate::graph::node::{GraphNode, RenderCtx};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VoiceState {
    Free,      // Available for allocation
    Active,    // Playing, envelope in attack/decay/sustain
    Releasing, // Key released, envelope in release phase
}

/// A single voice that can play any GraphNode
pub struct Voice<T: GraphNode> {
    note: u8,
    velocity: u8,
    state: VoiceState,
    age: u64,
    sample_rate: f32,
    graph: T,
}

impl<T: GraphNode> Voice<T> {
    pub fn new(graph: T, sample_rate: f32) -> Self {
        Self {
            note: 0,
            velocity: 0,
            state: VoiceState::Free,
            age: 0,
            sample_rate,
            graph,
        }
    }

    pub fn start(&mut self, note: u8, velocity: u8, age: u64) {
        self.note = note;
        self.velocity = velocity;
        self.state = VoiceState::Active;
        self.age = age;

        // Trigger note-on event in the graph
        let ctx = RenderCtx::from_note(self.sample_rate, note, velocity as f32);
        self.graph.note_on(&ctx);
    }

    pub fn release(&mut self) {
        if self.state == VoiceState::Active {
            self.state = VoiceState::Releasing;

            // Trigger note-off event in the graph
            let ctx = RenderCtx::from_note(self.sample_rate, self.note, self.velocity as f32);
            self.graph.note_off(&ctx);
        }
    }

    pub fn render(&mut self, out: &mut [f32]) {
        // Create context from voice state (MIDI note → frequency)
        let ctx = RenderCtx::from_note(self.sample_rate, self.note, self.velocity as f32);

        self.graph.render_block(out, &ctx);

        // If voice is releasing and envelope has finished, mark as free
        if self.state == VoiceState::Releasing && !self.graph.is_active() {
            self.free();
        }
    }

    pub fn is_free(&self) -> bool {
        self.state == VoiceState::Free
    }

    pub fn is_active(&self) -> bool {
        matches!(self.state, VoiceState::Active | VoiceState::Releasing)
    }

    pub fn get_envelope_level(&self) -> Option<f32> {
        self.graph.get_envelope_level()
    }

    pub fn free(&mut self) {
        self.state = VoiceState::Free;
        self.note = 0;
        self.velocity = 0;
    }

    pub fn note(&self) -> u8 {
        self.note
    }

    pub fn age(&self) -> u64 {
        self.age
    }

    pub fn state(&self) -> VoiceState {
        self.state
    }
}
