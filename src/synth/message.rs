#[cfg(feature = "rtrb")]
use rtrb::Consumer;

#[derive(Debug, Copy, Clone)]
pub enum SynthMessage {
    NoteOn { note: u8, velocity: u8 },
    NoteOff { note: u8, velocity: u8 },
    PitchBend { cents: f32 },
    AllNotesOff,
}

pub trait MessageReceiver {
    fn pop(&mut self) -> Option<SynthMessage>;
}

#[cfg(feature = "rtrb")]
impl MessageReceiver for Consumer<SynthMessage> {
    fn pop(&mut self) -> Option<SynthMessage> {
        Consumer::pop(self).ok()
    }
}
