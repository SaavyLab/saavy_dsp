#[derive(Debug, Copy, Clone)]
pub enum SynthMessage {
    NoteOn { note: u8, velocity: u8 },
    NoteOff { note: u8, velocity: u8 },
    PitchBend { cents: f32 },
    AllNotesOff,
}
