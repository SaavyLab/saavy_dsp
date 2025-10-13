use rtrb::Consumer;

use crate::{
    synth::{message::SynthMessage, voice::Voice},
    MAX_BLOCK_SIZE,
};

pub struct PolySynth {
    voices: Vec<Voice>,
    rx: Consumer<SynthMessage>,
    temp_buffer: Vec<f32>,
    frame_counter: u64,
}

impl PolySynth {
    pub fn new(
        sample_rate: f32,
        max_voices: usize,
        rx: Consumer<SynthMessage>,
        attack: f32,
        decay: f32,
        sustain: f32,
        release: f32,
    ) -> Self {
        let voices = (0..max_voices)
            .map(|_| Voice::new(sample_rate, attack, decay, sustain, release))
            .collect();

        Self {
            voices,
            rx,
            temp_buffer: vec![0.0; MAX_BLOCK_SIZE],
            frame_counter: 0,
        }
    }

    pub fn render_block(&mut self, out: &mut [f32]) {
        // Process control messages
        while let Ok(msg) = self.rx.pop() {
            match msg {
                SynthMessage::NoteOn { note, velocity } => {
                    let age = self.frame_counter;
                    if let Some(voice) = self.allocate_voice() {
                        voice.start(note, velocity, age);
                    }
                }
                SynthMessage::NoteOff { note, .. } => {
                    if let Some(voice) = self.find_voice(note) {
                        voice.release();
                    }
                }
                SynthMessage::AllNotesOff => {
                    for voice in &mut self.voices {
                        if voice.is_active() {
                            voice.release();
                        }
                    }
                }
                _ => {}
            }
        }

        // Mix voices
        out.fill(0.0);
        for voice in &mut self.voices {
            if voice.is_active() {
                self.temp_buffer.fill(0.0);
                voice.render(&mut self.temp_buffer[..out.len()]);

                for (o, v) in out.iter_mut().zip(&self.temp_buffer) {
                    *o += v;
                }
            }
        }

        self.frame_counter += out.len() as u64;
    }

    fn allocate_voice(&mut self) -> Option<&mut Voice> {
        use crate::synth::voice::VoiceState;

        // First pass: find free voice index
        let free_idx = self.voices.iter().position(|v| v.is_free());
        if let Some(idx) = free_idx {
            return Some(&mut self.voices[idx]);
        }

        // Second pass: steal oldest releasing voice
        let steal_idx = self
            .voices
            .iter()
            .enumerate()
            .filter(|(_, v)| v.state() == VoiceState::Releasing)
            .min_by_key(|(_, v)| v.age())
            .map(|(idx, _)| idx);

        steal_idx.map(|idx| &mut self.voices[idx])
    }

    fn find_voice(&mut self, note: u8) -> Option<&mut Voice> {
        self.voices
            .iter_mut()
            .find(|v| v.note() == note && v.is_active())
    }
}
