use super::duration::Duration;
use super::time_signature::TimeSignature;

/// A single event in a sequence (note or rest)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SequenceEvent {
    /// When this event occurs (in ticks from sequence start)
    pub tick_offset: u32,
    /// How long this event lasts (in ticks)
    pub duration_ticks: u32,
    /// MIDI note number (None = rest)
    pub note: Option<u8>,
    /// MIDI velocity (0-127)
    pub velocity: u8,
    /// Microtiming offset in ticks (for swing/humanization)
    /// Can be negative to rush, positive to drag
    pub offset_ticks: i32,
}

/// A musical sequence with time signature and events
#[derive(Debug, Clone)]
pub struct Sequence {
    /// Time signature
    pub time_signature: TimeSignature,
    /// Pulses per quarter note (timing resolution)
    pub ppq: u32,
    /// List of events in this sequence
    pub events: Vec<SequenceEvent>,
    /// Total duration in ticks
    pub total_ticks: u32,
}

impl Sequence {
    /// Create a new sequence builder with default 4/4 time signature
    pub fn new(ppq: u32) -> SequenceBuilder {
        SequenceBuilder::new(TimeSignature::FOUR_FOUR, ppq)
    }

    /// Create a new sequence builder with custom time signature
    pub fn with_time_signature(time_signature: TimeSignature, ppq: u32) -> SequenceBuilder {
        SequenceBuilder::new(time_signature, ppq)
    }

    /// Get all events that should trigger between two tick positions
    pub fn events_between(&self, start_tick: u32, end_tick: u32) -> impl Iterator<Item = &SequenceEvent> {
        self.events
            .iter()
            .filter(move |e| {
                let actual_offset = ((e.tick_offset as i32).saturating_add(e.offset_ticks)) as u32;
                actual_offset >= start_tick && actual_offset < end_tick
            })
    }

    /// Get the total duration of this sequence in ticks
    pub fn duration_ticks(&self) -> u32 {
        self.total_ticks
    }

    /// Get the duration of one bar in ticks
    pub fn bar_ticks(&self) -> u32 {
        self.time_signature.bar_ticks(self.ppq)
    }
}

/// Builder for constructing sequences with a fluent API
pub struct SequenceBuilder {
    time_signature: TimeSignature,
    ppq: u32,
    events: Vec<SequenceEvent>,
    cursor_ticks: u32, // Current position in ticks
    allow_anacrusis: bool,
    num_bars: u32,
}

impl SequenceBuilder {
    /// Create a new sequence builder
    fn new(time_signature: TimeSignature, ppq: u32) -> Self {
        Self {
            time_signature,
            ppq,
            events: Vec::new(),
            cursor_ticks: 0,
            allow_anacrusis: false,
            num_bars: 1,
        }
    }

    /// Set the number of bars for this sequence (default: 1)
    pub fn bars(mut self, num_bars: u32) -> Self {
        self.num_bars = num_bars;
        self
    }

    /// Allow anacrusis (pickup measure) - sequence can be shorter than full bars
    pub fn allow_anacrusis(mut self, allow: bool) -> Self {
        self.allow_anacrusis = allow;
        self
    }

    /// Add a note with the specified duration
    pub fn note(mut self, duration: Duration) -> Self {
        let ticks = duration.to_ticks(self.ppq);
        self.events.push(SequenceEvent {
            tick_offset: self.cursor_ticks,
            duration_ticks: ticks,
            note: Some(60), // Default to middle C
            velocity: 100,
            offset_ticks: 0,
        });
        self.cursor_ticks += ticks;
        self
    }

    /// Add a rest (silence) with the specified duration
    pub fn rest(mut self, duration: Duration) -> Self {
        let ticks = duration.to_ticks(self.ppq);
        self.cursor_ticks += ticks; // Just advance time without adding an event
        self
    }

    /// Set the MIDI note number for the last added event
    pub fn with_note(mut self, note: u8) -> Self {
        if let Some(event) = self.events.last_mut() {
            event.note = Some(note);
        }
        self
    }

    /// Set the velocity for the last added event
    pub fn with_velocity(mut self, velocity: u8) -> Self {
        if let Some(event) = self.events.last_mut() {
            event.velocity = velocity;
        }
        self
    }

    /// Set the microtiming offset for the last added event (for swing/humanization)
    pub fn with_offset(mut self, offset_ticks: i32) -> Self {
        if let Some(event) = self.events.last_mut() {
            event.offset_ticks = offset_ticks;
        }
        self
    }

    /// Build the final sequence
    /// Returns Result to handle bar validation errors
    pub fn build(self) -> Result<Sequence, SequenceError> {
        let bar_ticks = self.time_signature.bar_ticks(self.ppq);
        let expected_ticks = bar_ticks * self.num_bars;

        // Validate bar boundaries
        if self.cursor_ticks > expected_ticks {
            return Err(SequenceError::OverflowsBar {
                expected: expected_ticks,
                actual: self.cursor_ticks,
            });
        }

        if !self.allow_anacrusis && self.cursor_ticks < expected_ticks {
            return Err(SequenceError::UnderflowsBar {
                expected: expected_ticks,
                actual: self.cursor_ticks,
            });
        }

        Ok(Sequence {
            time_signature: self.time_signature,
            ppq: self.ppq,
            events: self.events,
            total_ticks: expected_ticks,
        })
    }
}

/// Errors that can occur when building a sequence
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SequenceError {
    /// Sequence overflows the specified number of bars
    OverflowsBar { expected: u32, actual: u32 },
    /// Sequence underflows the specified number of bars (and anacrusis not allowed)
    UnderflowsBar { expected: u32, actual: u32 },
}

impl std::fmt::Display for SequenceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SequenceError::OverflowsBar { expected, actual } => {
                write!(
                    f,
                    "Sequence overflows bar: expected {} ticks, got {} ticks",
                    expected, actual
                )
            }
            SequenceError::UnderflowsBar { expected, actual } => {
                write!(
                    f,
                    "Sequence underflows bar: expected {} ticks, got {} ticks (use .allow_anacrusis(true) for pickup measures)",
                    expected, actual
                )
            }
        }
    }
}

impl std::error::Error for SequenceError {}

#[cfg(test)]
mod tests {
    use super::*;

    const PPQ: u32 = 480;

    #[test]
    fn test_sequence_builder_basic() {
        let seq = Sequence::new(PPQ)
            .note(Duration::QUARTER)
            .note(Duration::QUARTER)
            .note(Duration::QUARTER)
            .note(Duration::QUARTER)
            .build()
            .unwrap();

        assert_eq!(seq.events.len(), 4);
        assert_eq!(seq.total_ticks, 1920); // 4 bars * 480 ticks

        // Check that events are at correct tick positions
        assert_eq!(seq.events[0].tick_offset, 0);
        assert_eq!(seq.events[1].tick_offset, 480);
        assert_eq!(seq.events[2].tick_offset, 960);
        assert_eq!(seq.events[3].tick_offset, 1440);
    }

    #[test]
    fn test_sequence_with_rests() {
        // Pattern: "1, and of 2, 4"
        let seq = Sequence::new(PPQ)
            .note(Duration::EIGHTH) // 1
            .rest(Duration::EIGHTH) // and
            .rest(Duration::EIGHTH) // 2
            .note(Duration::EIGHTH) // and
            .rest(Duration::QUARTER) // 3
            .note(Duration::QUARTER) // 4
            .build()
            .unwrap();

        assert_eq!(seq.events.len(), 3); // Only 3 notes (rests don't create events)

        // Check note positions (in ticks)
        assert_eq!(seq.events[0].tick_offset, 0); // Beat 1 = 0 ticks
        assert_eq!(seq.events[1].tick_offset, 720); // And of 2 = 720 ticks (1.5 beats)
        assert_eq!(seq.events[2].tick_offset, 1440); // Beat 4 = 1440 ticks (3 beats)
    }

    #[test]
    fn test_with_note_and_velocity() {
        let seq = Sequence::new(PPQ)
            .note(Duration::QUARTER)
            .with_note(36) // Kick drum
            .with_velocity(127)
            .rest(Duration::QUARTER)
            .rest(Duration::QUARTER)
            .rest(Duration::QUARTER)
            .build()
            .unwrap();

        assert_eq!(seq.events[0].note, Some(36));
        assert_eq!(seq.events[0].velocity, 127);
    }

    #[test]
    fn test_events_between() {
        let seq = Sequence::new(PPQ)
            .note(Duration::QUARTER) // Tick 0
            .note(Duration::QUARTER) // Tick 480
            .note(Duration::QUARTER) // Tick 960
            .note(Duration::QUARTER) // Tick 1440
            .build()
            .unwrap();

        let events: Vec<_> = seq.events_between(480, 1440).collect();
        assert_eq!(events.len(), 2); // Events at ticks 480 and 960
        assert_eq!(events[0].tick_offset, 480);
        assert_eq!(events[1].tick_offset, 960);
    }

    #[test]
    fn test_bar_overflow_error() {
        let result = Sequence::new(PPQ)
            .note(Duration::WHOLE) // 4 beats
            .note(Duration::QUARTER) // +1 beat = 5 beats total (overflows 1 bar)
            .build();

        assert!(matches!(result, Err(SequenceError::OverflowsBar { .. })));
    }

    #[test]
    fn test_bar_underflow_error() {
        let result = Sequence::new(PPQ)
            .note(Duration::QUARTER) // Only 1 beat (should be 4)
            .build();

        assert!(matches!(result, Err(SequenceError::UnderflowsBar { .. })));
    }

    #[test]
    fn test_anacrusis_allowed() {
        let result = Sequence::new(PPQ)
            .allow_anacrusis(true)
            .note(Duration::QUARTER) // Pickup note
            .build();

        assert!(result.is_ok());
    }

    #[test]
    fn test_multiple_bars() {
        let seq = Sequence::new(PPQ)
            .bars(2)
            .note(Duration::WHOLE) // Bar 1
            .note(Duration::WHOLE) // Bar 2
            .build()
            .unwrap();

        assert_eq!(seq.total_ticks, 3840); // 2 bars * 1920 ticks
    }

    #[test]
    fn test_compound_meter() {
        let seq = Sequence::with_time_signature(TimeSignature::SIX_EIGHT, PPQ)
            .note(Duration::DOTTED_QUARTER) // 1 tactus beat
            .note(Duration::DOTTED_QUARTER) // 1 tactus beat
            .build()
            .unwrap();

        assert_eq!(seq.events.len(), 2);
        // 6/8 bar = 1440 ticks
        assert_eq!(seq.total_ticks, 1440);
    }

    #[test]
    fn test_microtiming_offset() {
        let seq = Sequence::new(PPQ)
            .note(Duration::EIGHTH)
            .with_offset(20) // Rush by 20 ticks
            .note(Duration::EIGHTH)
            .with_offset(-10) // Drag by 10 ticks
            .rest(Duration::HALF)
            .note(Duration::QUARTER)
            .build()
            .unwrap();

        assert_eq!(seq.events[0].offset_ticks, 20);
        assert_eq!(seq.events[1].offset_ticks, -10);
    }
}
