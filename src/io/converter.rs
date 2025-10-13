use crate::{io::midi::MidiEvent, synth::message::SynthMessage};

pub fn midi_to_synth(midi: MidiEvent, channel_filter: u8) -> Option<SynthMessage> {
    match midi {
        MidiEvent::NoteOn {
            channel,
            key,
            velocity,
        } if channel == channel_filter => Some(SynthMessage::NoteOn {
            note: key,
            velocity,
        }),
        MidiEvent::NoteOff {
            channel,
            key,
            velocity,
        } if channel == channel_filter => Some(SynthMessage::NoteOff {
            note: key,
            velocity,
        }),
        _ => None,
    }
}

pub fn midi_note_to_freq(note: u8) -> f32 {
    // Clamp to valid MIDI range to prevent dangerous frequencies
    let note = note.min(127);
    440.0 * 2.0_f32.powf((note as f32 - 69.0) / 12.0)
}
