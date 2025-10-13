use rtrb::{Consumer, Producer, RingBuffer};

use crate::{dsp::envelope::Envelope, graph::node::GraphNode};

pub struct EnvNode {
    env: Envelope,
}

impl EnvNode {
    pub fn note_on(&mut self) {
        self.env.note_on();
    }

    pub fn note_off(&mut self) {
        self.env.note_off();
    }

    pub fn new(sample_rate: f32) -> Self {
        let env = Envelope::new(sample_rate);
        Self { env }
    }

    pub fn with_params(
        sample_rate: f32,
        attack: f32,
        decay: f32,
        sustain: f32,
        release: f32,
    ) -> Self {
        let env = Envelope::adsr(sample_rate, attack, decay, sustain, release);
        Self { env }
    }
}

impl GraphNode for EnvNode {
    fn render_block(&mut self, out: &mut [f32]) {
        self.env.render(out);
    }
}

pub enum EnvelopeMessage {
    NoteOn,
    NoteOff,
}

pub struct EnvelopeHandle {
    tx: Producer<EnvelopeMessage>,
}

pub struct SharedEnvNode {
    env: Envelope,
    rx: Consumer<EnvelopeMessage>,
}

impl EnvelopeHandle {
    pub fn note_on(&mut self) {
        let _ = self.tx.push(EnvelopeMessage::NoteOn);
    }

    pub fn note_off(&mut self) {
        let _ = self.tx.push(EnvelopeMessage::NoteOff);
    }
}

const ENVELOPE_QUEUE_SIZE: usize = 64;

impl SharedEnvNode {
    pub fn adsr(
        sample_rate: f32,
        attack: f32,
        decay: f32,
        sustain: f32,
        release: f32,
    ) -> (Self, EnvelopeHandle) {
        let env = Envelope::adsr(sample_rate, attack, decay, sustain, release);
        let (tx, rx) = RingBuffer::<EnvelopeMessage>::new(ENVELOPE_QUEUE_SIZE);

        let handle = EnvelopeHandle { tx };
        let node = Self { env, rx };

        (node, handle)
    }

    /// Check if envelope is currently active (not Idle)
    pub fn is_active(&self) -> bool {
        self.env.is_active()
    }
}

impl GraphNode for SharedEnvNode {
    fn render_block(&mut self, out: &mut [f32]) {
        while let Ok(msg) = self.rx.pop() {
            match msg {
                EnvelopeMessage::NoteOff => self.env.note_off(),
                EnvelopeMessage::NoteOn => self.env.note_on(),
            }
        }

        self.env.render(out);
    }
}
