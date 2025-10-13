use rtrb::Consumer;

use crate::{
    synth::{
        factory::VoiceFactory,
        message::SynthMessage,
        voice::{Voice, VoiceState},
    },
    MAX_BLOCK_SIZE,
};

/// Polyphonic synthesizer - manages multiple voices of any type
pub struct PolySynth<F: VoiceFactory> {
    voices: Vec<Voice<F::Voice>>,
    rx: Consumer<SynthMessage>,
    temp_buffer: Vec<f32>,
    frame_counter: u64,
}

impl<F: VoiceFactory> PolySynth<F> {
    /// Create a new polyphonic synth
    ///
    /// # Arguments
    /// * `sample_rate` - Audio sample rate (e.g., 48000.0)
    /// * `max_voices` - Maximum number of simultaneous voices
    /// * `factory` - Factory that creates voices with your sound design
    /// * `rx` - Message queue for MIDI events
    pub fn new(
        sample_rate: f32,
        max_voices: usize,
        factory: F,
        rx: Consumer<SynthMessage>,
    ) -> Self {
        let voices = (0..max_voices)
            .map(|_| Voice::new(factory.create_voice(), sample_rate))
            .collect();

        Self {
            voices,
            rx,
            temp_buffer: vec![0.0; MAX_BLOCK_SIZE],
            frame_counter: 0,
        }
    }

    pub fn render_block(&mut self, out: &mut [f32]) {
        // Safety: ensure buffer size doesn't exceed our temp buffer
        debug_assert!(
            out.len() <= MAX_BLOCK_SIZE,
            "Buffer size {} exceeds MAX_BLOCK_SIZE {}",
            out.len(),
            MAX_BLOCK_SIZE
        );

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

    fn allocate_voice(&mut self) -> Option<&mut Voice<F::Voice>> {
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

    fn find_voice(&mut self, note: u8) -> Option<&mut Voice<F::Voice>> {
        self.voices
            .iter_mut()
            .find(|v| v.note() == note && v.is_active())
    }
}
