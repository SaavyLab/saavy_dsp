use crate::{
    graph::{
        amplify::Amplify,
        envelope::{EnvelopeHandle, SharedEnvNode},
        node::GraphNode,
        oscillator::OscNode,
    },
    io::converter::midi_note_to_freq,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VoiceState {
    Free,      // Available for allocation
    Active,    // Playing, envelope in attack/decay/sustain
    Releasing, // Key released, envelope in release phase
}

pub struct Voice {
    note: u8,
    velocity: u8,
    state: VoiceState,
    age: u64,
    chain: VoiceChain,
    env_handle: EnvelopeHandle,
}

type VoiceChain = Amplify<OscNode, SharedEnvNode>;

impl Voice {
    pub fn new(sample_rate: f32, attack: f32, decay: f32, sustain: f32, release: f32) -> Self {
        let (env_node, env_handle) =
            SharedEnvNode::adsr(sample_rate, attack, decay, sustain, release);
        let osc = OscNode::sine(440.0, sample_rate);
        let chain = Amplify::new(osc, env_node);

        Self {
            note: 0,
            velocity: 0,
            state: VoiceState::Free,
            age: 0,
            chain,
            env_handle,
        }
    }

    pub fn start(&mut self, note: u8, velocity: u8, age: u64) {
        self.note = note;
        self.velocity = velocity;
        self.state = VoiceState::Active;
        self.age = age;

        let freq = midi_note_to_freq(note);
        self.chain.signal.set_frequency(freq);
        self.env_handle.note_on();
    }

    pub fn release(&mut self) {
        if self.state == VoiceState::Active {
            self.state = VoiceState::Releasing;
            self.env_handle.note_off();
        }
    }

    pub fn render(&mut self, out: &mut [f32]) {
        self.chain.render_block(out);
    }

    pub fn is_free(&self) -> bool {
        self.state == VoiceState::Free
    }

    pub fn is_active(&self) -> bool {
        matches!(self.state, VoiceState::Active | VoiceState::Releasing)
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
