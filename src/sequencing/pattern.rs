/*
Pattern API
===========

A higher-level API for creating musical sequences. Patterns represent a single
cycle (usually one bar) of music where timing is implicit based on position.

The key insight: in most music, notes divide time equally. A 4/4 bar with 4 notes
means each note is a quarter note. Subdivisions (brackets in the macro syntax)
create faster notes within a slot.

Example mental model:
    [C4, E4, G4, C5]     = 4 quarter notes
    [C4, [E4, G4], _, _] = quarter, 2 eighths, 2 quarter rests
    [[C4, E4, G4], ...]  = triplet (3 notes in 1 beat)

This module provides:
- `PatternSlot` - A single slot that can be a note, rest, or subdivision
- `Pattern` - A collection of slots with a time signature
- Conversion to the low-level `Sequence` type for playback
*/

use super::time_signature::TimeSignature;
use super::{Sequence, SequenceEvent};

/// A slot in a pattern - can be a note, rest, or subdivision
#[derive(Debug, Clone, PartialEq)]
pub enum PatternSlot {
    /// A single note (MIDI note number)
    Note(NoteSlot),
    /// Silence for this slot
    Rest,
    /// Subdivide this slot into smaller parts
    Subdivision(Vec<PatternSlot>),
}

/// A note with optional weight for swing/uneven subdivisions
#[derive(Debug, Clone, PartialEq)]
pub struct NoteSlot {
    /// MIDI note number
    pub note: u8,
    /// Velocity (0-127), defaults to 100
    pub velocity: u8,
    /// Weight for uneven subdivisions (default 1)
    /// In a subdivision like [C4@2, E4], C4 gets 2/3 of the time
    pub weight: u8,
}

impl NoteSlot {
    pub fn new(note: u8) -> Self {
        Self {
            note,
            velocity: 100,
            weight: 1,
        }
    }

    pub fn with_velocity(mut self, velocity: u8) -> Self {
        self.velocity = velocity;
        self
    }

    pub fn with_weight(mut self, weight: u8) -> Self {
        self.weight = weight;
        self
    }
}

/// Convenient conversion from u8 (MIDI note) to PatternSlot
impl From<u8> for PatternSlot {
    fn from(note: u8) -> Self {
        PatternSlot::Note(NoteSlot::new(note))
    }
}

/// A pattern is one cycle of music with a time signature
#[derive(Debug, Clone)]
pub struct Pattern {
    /// Time signature for this pattern
    pub time_signature: TimeSignature,
    /// The slots in this pattern (one per beat in simple meter)
    pub slots: Vec<PatternSlot>,
}

impl Pattern {
    /// Create a new pattern with a time signature and slots
    pub fn new(time_signature: TimeSignature, slots: Vec<PatternSlot>) -> Self {
        Self {
            time_signature,
            slots,
        }
    }

    /// Create a 4/4 pattern (most common)
    pub fn four_four(slots: Vec<PatternSlot>) -> Self {
        Self::new(TimeSignature::FOUR_FOUR, slots)
    }

    /// Create a 3/4 pattern
    pub fn three_four(slots: Vec<PatternSlot>) -> Self {
        Self::new(TimeSignature::THREE_FOUR, slots)
    }

    /// Create a 6/8 pattern
    pub fn six_eight(slots: Vec<PatternSlot>) -> Self {
        Self::new(TimeSignature::SIX_EIGHT, slots)
    }

    /// Chain this pattern with another (play one after the other)
    pub fn then(self, other: Pattern) -> PatternChain {
        PatternChain {
            patterns: vec![self, other],
        }
    }

    /// Repeat this pattern n times
    pub fn repeat(self, n: usize) -> PatternChain {
        PatternChain {
            patterns: vec![self; n],
        }
    }

    /// Convert to a low-level Sequence for playback
    pub fn to_sequence(&self, ppq: u32) -> Sequence {
        let bar_ticks = self.time_signature.bar_ticks(ppq);
        let slot_count = self.slots.len() as u32;

        // Handle empty pattern - return empty sequence
        if slot_count == 0 {
            return Sequence {
                time_signature: self.time_signature.clone(),
                ppq,
                events: Vec::new(),
                total_ticks: bar_ticks,
            };
        }

        // Each top-level slot gets an equal portion of the bar
        let ticks_per_slot = bar_ticks / slot_count;

        let mut events = Vec::new();
        let mut cursor = 0u32;

        for slot in &self.slots {
            Self::expand_slot(slot, cursor, ticks_per_slot, &mut events);
            cursor += ticks_per_slot;
        }

        Sequence {
            time_signature: self.time_signature.clone(),
            ppq,
            events,
            total_ticks: bar_ticks,
        }
    }

    /// Recursively expand a slot into sequence events
    fn expand_slot(slot: &PatternSlot, start_tick: u32, duration: u32, events: &mut Vec<SequenceEvent>) {
        match slot {
            PatternSlot::Note(note_slot) => {
                events.push(SequenceEvent {
                    tick_offset: start_tick,
                    duration_ticks: duration,
                    note: Some(note_slot.note),
                    velocity: note_slot.velocity,
                    offset_ticks: 0,
                });
            }
            PatternSlot::Rest => {
                // Rests don't create events, just consume time
            }
            PatternSlot::Subdivision(sub_slots) => {
                assert!(
                    !sub_slots.is_empty(),
                    "Empty subdivision is not allowed - use PatternSlot::Rest for silence"
                );

                // Calculate total weight
                let total_weight: u32 = sub_slots
                    .iter()
                    .map(|s| match s {
                        PatternSlot::Note(n) => n.weight as u32,
                        _ => 1,
                    })
                    .sum();

                // Distribute time according to weights
                let mut sub_cursor = start_tick;
                for sub_slot in sub_slots {
                    let weight = match sub_slot {
                        PatternSlot::Note(n) => n.weight as u32,
                        _ => 1,
                    };
                    let sub_duration = (duration * weight) / total_weight;
                    Self::expand_slot(sub_slot, sub_cursor, sub_duration, events);
                    sub_cursor += sub_duration;
                }
            }
        }
    }
}

/// A chain of patterns to be played in sequence
#[derive(Debug, Clone)]
pub struct PatternChain {
    patterns: Vec<Pattern>,
}

impl PatternChain {
    /// Add another pattern to the chain
    pub fn then(mut self, pattern: Pattern) -> Self {
        self.patterns.push(pattern);
        self
    }

    /// Concatenate another chain onto this one
    ///
    /// This is useful when you have two chains you want to play in sequence,
    /// for example an intro followed by a main section.
    ///
    /// # Example
    /// ```ignore
    /// let intro = pattern!(4/4 => [C4, _, _, _]).repeat(4);
    /// let main = pattern!(4/4 => [C4, E4, G4, C5]).repeat(8);
    /// let full = intro.concat(main);
    /// ```
    pub fn concat(mut self, other: PatternChain) -> Self {
        self.patterns.extend(other.patterns);
        self
    }

    /// Repeat the entire chain n times
    pub fn repeat(mut self, n: usize) -> Self {
        let original = self.patterns.clone();
        for _ in 1..n {
            self.patterns.extend(original.clone());
        }
        self
    }

    /// Convert to a low-level Sequence for playback
    pub fn to_sequence(&self, ppq: u32) -> Sequence {
        let mut all_events = Vec::new();
        let mut cursor = 0u32;

        // Use the first pattern's time signature (could be smarter about this)
        let time_signature = self
            .patterns
            .first()
            .map(|p| p.time_signature.clone())
            .unwrap_or(TimeSignature::FOUR_FOUR);

        for pattern in &self.patterns {
            let bar_ticks = pattern.time_signature.bar_ticks(ppq);
            let seq = pattern.to_sequence(ppq);

            // Offset all events by the current cursor position
            for mut event in seq.events {
                event.tick_offset += cursor;
                all_events.push(event);
            }

            cursor += bar_ticks;
        }

        Sequence {
            time_signature,
            ppq,
            events: all_events,
            total_ticks: cursor,
        }
    }
}

/// Macro for creating patterns with a concise syntax
///
/// # Examples
///
/// ```
/// use saavy_dsp::pattern;
/// use saavy_dsp::sequencing::*;
///
/// // Simple 4/4 pattern with four quarter notes
/// let arp = pattern!(4/4 => [C4, E4, G4, C5]);
///
/// // Pattern with rests (use _)
/// let sparse = pattern!(4/4 => [C4, _, G4, _]);
///
/// // Subdivisions with brackets
/// let eighths = pattern!(4/4 => [C4, [E4, G4], C5, _]);
///
/// // Triplets (3 notes in one beat)
/// let triplet = pattern!(4/4 => [[C4, E4, G4], _, _, _]);
///
/// // 6/8 compound meter
/// let waltz = pattern!(6/8 => [C4, G4]);
/// ```
#[macro_export]
macro_rules! pattern {
    // 4/4 time signature
    (4/4 => [$($slot:tt),* $(,)?]) => {
        $crate::sequencing::Pattern::new(
            $crate::sequencing::TimeSignature::FOUR_FOUR,
            vec![$($crate::pattern!(@slot $slot)),*]
        )
    };

    // 3/4 time signature
    (3/4 => [$($slot:tt),* $(,)?]) => {
        $crate::sequencing::Pattern::new(
            $crate::sequencing::TimeSignature::THREE_FOUR,
            vec![$($crate::pattern!(@slot $slot)),*]
        )
    };

    // 6/8 time signature
    (6/8 => [$($slot:tt),* $(,)?]) => {
        $crate::sequencing::Pattern::new(
            $crate::sequencing::TimeSignature::SIX_EIGHT,
            vec![$($crate::pattern!(@slot $slot)),*]
        )
    };

    // 2/4 time signature
    (2/4 => [$($slot:tt),* $(,)?]) => {
        $crate::sequencing::Pattern::new(
            $crate::sequencing::TimeSignature::TWO_FOUR,
            vec![$($crate::pattern!(@slot $slot)),*]
        )
    };

    // Rest slot
    (@slot _) => {
        $crate::sequencing::PatternSlot::Rest
    };

    // Subdivision slot (brackets)
    (@slot [$($inner:tt),* $(,)?]) => {
        $crate::sequencing::PatternSlot::Subdivision(
            vec![$($crate::pattern!(@slot $inner)),*]
        )
    };

    // Note slot (any other identifier/expression)
    (@slot $note:expr) => {
        $crate::sequencing::PatternSlot::from($note)
    };
}

// Re-export the macro at the crate level
pub use pattern;

/// Helper functions for building pattern slots
pub mod slot {
    use super::*;

    /// Create a note slot
    pub fn note(midi_note: u8) -> PatternSlot {
        PatternSlot::Note(NoteSlot::new(midi_note))
    }

    /// Create a note slot with velocity
    pub fn note_vel(midi_note: u8, velocity: u8) -> PatternSlot {
        PatternSlot::Note(NoteSlot::new(midi_note).with_velocity(velocity))
    }

    /// Create a note slot with weight (for swing)
    pub fn note_weight(midi_note: u8, weight: u8) -> PatternSlot {
        PatternSlot::Note(NoteSlot::new(midi_note).with_weight(weight))
    }

    /// Create a rest slot
    pub fn rest() -> PatternSlot {
        PatternSlot::Rest
    }

    /// Create a subdivision
    pub fn sub(slots: Vec<PatternSlot>) -> PatternSlot {
        PatternSlot::Subdivision(slots)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sequencing::notes::*;

    const PPQ: u32 = 480;

    #[test]
    fn test_simple_four_note_pattern() {
        // C major arpeggio: C4, E4, G4, C5
        let pattern = Pattern::four_four(vec![
            C4.into(),
            E4.into(),
            G4.into(),
            C5.into(),
        ]);

        let seq = pattern.to_sequence(PPQ);

        assert_eq!(seq.events.len(), 4);
        assert_eq!(seq.total_ticks, 1920); // 4/4 bar at 480 PPQ

        // Each note is a quarter note (480 ticks apart)
        assert_eq!(seq.events[0].tick_offset, 0);
        assert_eq!(seq.events[0].note, Some(C4));
        assert_eq!(seq.events[1].tick_offset, 480);
        assert_eq!(seq.events[1].note, Some(E4));
        assert_eq!(seq.events[2].tick_offset, 960);
        assert_eq!(seq.events[2].note, Some(G4));
        assert_eq!(seq.events[3].tick_offset, 1440);
        assert_eq!(seq.events[3].note, Some(C5));
    }

    #[test]
    fn test_pattern_with_rests() {
        // C4, rest, G4, rest
        let pattern = Pattern::four_four(vec![
            C4.into(),
            PatternSlot::Rest,
            G4.into(),
            PatternSlot::Rest,
        ]);

        let seq = pattern.to_sequence(PPQ);

        assert_eq!(seq.events.len(), 2); // Only 2 notes
        assert_eq!(seq.events[0].tick_offset, 0);
        assert_eq!(seq.events[0].note, Some(C4));
        assert_eq!(seq.events[1].tick_offset, 960); // Beat 3
        assert_eq!(seq.events[1].note, Some(G4));
    }

    #[test]
    fn test_subdivision() {
        // Quarter, two eighths, quarter, quarter
        let pattern = Pattern::four_four(vec![
            C4.into(),
            PatternSlot::Subdivision(vec![E4.into(), G4.into()]),
            C5.into(),
            PatternSlot::Rest,
        ]);

        let seq = pattern.to_sequence(PPQ);

        assert_eq!(seq.events.len(), 4);

        // C4 at beat 1
        assert_eq!(seq.events[0].tick_offset, 0);
        assert_eq!(seq.events[0].duration_ticks, 480);

        // E4 at beat 2 (first eighth)
        assert_eq!(seq.events[1].tick_offset, 480);
        assert_eq!(seq.events[1].duration_ticks, 240);

        // G4 at beat 2.5 (second eighth)
        assert_eq!(seq.events[2].tick_offset, 720);
        assert_eq!(seq.events[2].duration_ticks, 240);

        // C5 at beat 3
        assert_eq!(seq.events[3].tick_offset, 960);
    }

    #[test]
    fn test_triplet() {
        // Triplet on beat 1, then rests
        let pattern = Pattern::four_four(vec![
            PatternSlot::Subdivision(vec![C4.into(), E4.into(), G4.into()]),
            PatternSlot::Rest,
            PatternSlot::Rest,
            PatternSlot::Rest,
        ]);

        let seq = pattern.to_sequence(PPQ);

        assert_eq!(seq.events.len(), 3);

        // Each triplet note is 480/3 = 160 ticks
        assert_eq!(seq.events[0].tick_offset, 0);
        assert_eq!(seq.events[0].duration_ticks, 160);
        assert_eq!(seq.events[1].tick_offset, 160);
        assert_eq!(seq.events[1].duration_ticks, 160);
        assert_eq!(seq.events[2].tick_offset, 320);
        assert_eq!(seq.events[2].duration_ticks, 160);
    }

    #[test]
    fn test_swing_with_weights() {
        use slot::*;

        // Swing: first note gets 2 parts, second gets 1 (2:1 ratio)
        let pattern = Pattern::four_four(vec![
            sub(vec![note_weight(C4, 2), note(E4)]),
            PatternSlot::Rest,
            PatternSlot::Rest,
            PatternSlot::Rest,
        ]);

        let seq = pattern.to_sequence(PPQ);

        assert_eq!(seq.events.len(), 2);

        // C4 gets 2/3 of the beat = 320 ticks
        assert_eq!(seq.events[0].tick_offset, 0);
        assert_eq!(seq.events[0].duration_ticks, 320);

        // E4 gets 1/3 of the beat = 160 ticks
        assert_eq!(seq.events[1].tick_offset, 320);
        assert_eq!(seq.events[1].duration_ticks, 160);
    }

    #[test]
    fn test_pattern_chain() {
        let intro = Pattern::four_four(vec![C4.into(), PatternSlot::Rest, PatternSlot::Rest, PatternSlot::Rest]);
        let main = Pattern::four_four(vec![C4.into(), E4.into(), G4.into(), C5.into()]);

        let chain = intro.then(main);
        let seq = chain.to_sequence(PPQ);

        assert_eq!(seq.events.len(), 5); // 1 from intro + 4 from main
        assert_eq!(seq.total_ticks, 3840); // 2 bars

        // First note at tick 0
        assert_eq!(seq.events[0].tick_offset, 0);
        // Second note at tick 1920 (start of bar 2)
        assert_eq!(seq.events[1].tick_offset, 1920);
    }

    #[test]
    fn test_pattern_repeat() {
        let pattern = Pattern::four_four(vec![C4.into(), PatternSlot::Rest, PatternSlot::Rest, PatternSlot::Rest]);

        let chain = pattern.repeat(4);
        let seq = chain.to_sequence(PPQ);

        assert_eq!(seq.events.len(), 4); // 1 note per bar * 4 bars
        assert_eq!(seq.total_ticks, 7680); // 4 bars

        // Notes at start of each bar
        assert_eq!(seq.events[0].tick_offset, 0);
        assert_eq!(seq.events[1].tick_offset, 1920);
        assert_eq!(seq.events[2].tick_offset, 3840);
        assert_eq!(seq.events[3].tick_offset, 5760);
    }

    #[test]
    fn test_six_eight_compound() {
        // 6/8: 2 beats, each naturally subdivides into 3
        let pattern = Pattern::six_eight(vec![
            PatternSlot::Subdivision(vec![C4.into(), E4.into(), G4.into()]),
            PatternSlot::Subdivision(vec![C5.into(), G4.into(), E4.into()]),
        ]);

        let seq = pattern.to_sequence(PPQ);

        assert_eq!(seq.events.len(), 6);
        assert_eq!(seq.total_ticks, 1440); // 6/8 bar

        // Each beat is 720 ticks, subdivided into 3 = 240 ticks each
        assert_eq!(seq.events[0].tick_offset, 0);
        assert_eq!(seq.events[0].duration_ticks, 240);
        assert_eq!(seq.events[3].tick_offset, 720); // Start of beat 2
    }

    #[test]
    fn test_slot_helpers() {
        use slot::*;

        let pattern = Pattern::four_four(vec![
            note(C4),
            note_vel(E4, 127),
            rest(),
            sub(vec![note(G4), note(C5)]),
        ]);

        let seq = pattern.to_sequence(PPQ);

        assert_eq!(seq.events.len(), 4);
        assert_eq!(seq.events[1].velocity, 127);
    }

    // Macro tests
    #[test]
    fn test_pattern_macro_basic() {
        let p = pattern!(4/4 => [C4, E4, G4, C5]);
        let seq = p.to_sequence(PPQ);

        assert_eq!(seq.events.len(), 4);
        assert_eq!(seq.events[0].note, Some(C4));
        assert_eq!(seq.events[1].note, Some(E4));
        assert_eq!(seq.events[2].note, Some(G4));
        assert_eq!(seq.events[3].note, Some(C5));
    }

    #[test]
    fn test_pattern_macro_with_rests() {
        let p = pattern!(4/4 => [C4, _, G4, _]);
        let seq = p.to_sequence(PPQ);

        assert_eq!(seq.events.len(), 2);
        assert_eq!(seq.events[0].note, Some(C4));
        assert_eq!(seq.events[1].note, Some(G4));
    }

    #[test]
    fn test_pattern_macro_subdivision() {
        let p = pattern!(4/4 => [C4, [E4, G4], C5, _]);
        let seq = p.to_sequence(PPQ);

        assert_eq!(seq.events.len(), 4);
        // E4 and G4 are subdivided (eighth notes)
        assert_eq!(seq.events[1].duration_ticks, 240);
        assert_eq!(seq.events[2].duration_ticks, 240);
    }

    #[test]
    fn test_pattern_macro_triplet() {
        let p = pattern!(4/4 => [[C4, E4, G4], _, _, _]);
        let seq = p.to_sequence(PPQ);

        assert_eq!(seq.events.len(), 3);
        // Each triplet note is 480/3 = 160 ticks
        assert_eq!(seq.events[0].duration_ticks, 160);
    }

    #[test]
    fn test_pattern_macro_nested() {
        // Quarter, then sixteenths (4 notes in one beat)
        let p = pattern!(4/4 => [C4, [[E4, F4], [G4, A4]], C5, _]);
        let seq = p.to_sequence(PPQ);

        // C4, E4, F4, G4, A4, C5 = 6 notes
        assert_eq!(seq.events.len(), 6);
        // E4 at beat 2, gets 1/4 of the beat = 120 ticks
        assert_eq!(seq.events[1].duration_ticks, 120);
    }

    #[test]
    fn test_pattern_macro_six_eight() {
        let p = pattern!(6/8 => [C4, G4]);
        let seq = p.to_sequence(PPQ);

        assert_eq!(seq.events.len(), 2);
        assert_eq!(seq.total_ticks, 1440); // 6/8 bar
    }

    #[test]
    fn test_pattern_macro_trailing_comma() {
        // Should work with trailing comma
        let p = pattern!(4/4 => [C4, E4, G4, C5,]);
        assert_eq!(p.slots.len(), 4);
    }

    // Edge case tests
    #[test]
    fn test_empty_pattern() {
        // Empty pattern should not panic, returns empty sequence
        let p = Pattern::four_four(vec![]);
        let seq = p.to_sequence(PPQ);

        assert_eq!(seq.events.len(), 0);
        assert_eq!(seq.total_ticks, 1920); // Still a full bar
    }

    #[test]
    #[should_panic(expected = "Empty subdivision is not allowed")]
    fn test_empty_subdivision_panics() {
        // Empty subdivision should panic with clear message
        let p = Pattern::four_four(vec![
            C4.into(),
            PatternSlot::Subdivision(vec![]), // empty subdivision - not allowed!
            G4.into(),
            PatternSlot::Rest,
        ]);
        let _ = p.to_sequence(PPQ); // This should panic
    }
}
