use crate::io::midi::MidiEvent;

pub trait VoiceAllocator {
    fn handle_event(&mut self, event: MidiEvent);
    fn advance(&mut self, frames: usize);
}
