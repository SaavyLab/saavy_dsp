//! Track - one voice playing a sequence
//!
//! Simple model: one track = one GraphNode = one voice.
//! Polyphony is achieved by creating multiple tracks.

use crate::{
    graph::{GraphNode, RenderCtx},
    sequencing::Sequence,
};

/// A monophonic track - one voice playing a sequence
pub struct Track {
    /// Display name
    pub name: String,
    /// The sequence to play
    pub sequence: Sequence,
    /// The audio processing node
    node: Box<dyn GraphNode>,
    /// Current note being played (if any)
    current_note: Option<u8>,
    /// Current velocity
    velocity: f32,
}

impl Track {
    /// Create a new track with a sequence and audio node
    pub fn new<N: GraphNode + 'static>(
        name: impl Into<String>,
        mut sequence: Sequence,
        node: N,
    ) -> Self {
        // Sort events by effective trigger time (tick_offset + offset_ticks)
        // This is necessary because negative offsets (swing/humanization) can
        // cause events to fire earlier than their tick_offset suggests
        sequence.events.sort_by_key(|e| {
            e.tick_offset.saturating_add_signed(e.offset_ticks)
        });

        Self {
            name: name.into(),
            sequence,
            node: Box::new(node),
            current_note: None,
            velocity: 0.0,
        }
    }

    /// Trigger a note on this track
    pub fn note_on(&mut self, note: u8, velocity: u8, sample_rate: f32) {
        self.current_note = Some(note);
        self.velocity = velocity as f32;

        let ctx = RenderCtx::from_note(sample_rate, note, self.velocity);
        self.node.note_on(&ctx);
    }

    /// Release the current note
    pub fn note_off(&mut self, note: u8, sample_rate: f32) {
        // Only release if it's the note we're playing
        if self.current_note == Some(note) {
            let ctx = RenderCtx::from_note(sample_rate, note, 0.0);
            self.node.note_off(&ctx);
            // Don't clear current_note yet - let envelope finish
        }
    }

    /// Render audio into the buffer
    pub fn render(&mut self, out: &mut [f32], sample_rate: f32) {
        if let Some(note) = self.current_note {
            let ctx = RenderCtx::from_note(sample_rate, note, self.velocity);
            self.node.render_block(out, &ctx);

            // Check if the node is done (envelope finished)
            if !self.node.is_active() {
                self.current_note = None;
            }
        } else {
            // No note playing - output silence
            out.fill(0.0);
        }
    }

    /// Check if this track is currently producing sound
    pub fn is_active(&self) -> bool {
        self.current_note.is_some() && self.node.is_active()
    }

    /// Get the envelope level (for visualization)
    pub fn envelope_level(&self) -> Option<f32> {
        self.node.get_envelope_level()
    }

    /// Get the current note being played (for visualization)
    pub fn current_note(&self) -> Option<u8> {
        self.current_note
    }
}
