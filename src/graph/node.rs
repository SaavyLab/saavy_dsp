/// Convert MIDI note number to frequency in Hz.
/// A4 = 440 Hz = MIDI note 69
#[inline]
fn midi_note_to_freq(note: u8) -> f32 {
    440.0 * 2.0_f32.powf((note as f32 - 69.0) / 12.0)
}

/// Context passed to graph nodes during rendering
///
/// Contains information about what to render:
/// - sample_rate: Audio sample rate (e.g., 48000.0)
/// - frequency: Pitch to render (Hz)
/// - velocity: Intensity/loudness (0.0-127.0, MIDI-style)
/// - time: Current playback time in seconds
pub struct RenderCtx {
    pub sample_rate: f32,
    pub frequency: f32,
    pub velocity: f32,
    pub time: f64,
}

impl RenderCtx {
    /// Create context from MIDI note (keyboard/sequencer use case)
    pub fn from_note(sample_rate: f32, note: u8, velocity: f32) -> Self {
        let frequency = midi_note_to_freq(note);

        Self {
            sample_rate,
            frequency,
            velocity,
            time: 0.0,
        }
    }

    /// Create context from direct frequency (metronome/drum machine use case)
    pub fn from_freq(sample_rate: f32, frequency: f32, velocity: f32) -> Self {
        Self {
            sample_rate,
            frequency,
            velocity,
            time: 0.0,
        }
    }
}

/// Trait for nodes that support parameter modulation
pub trait Modulatable: Send {
    type Param: Copy + Send;

    fn get_param(&self, param: Self::Param) -> f32;

    fn apply_modulation(&mut self, param: Self::Param, base: f32, modulation: f32);
}

/// Core trait for audio processing graph nodes
///
/// Nodes can render audio and respond to musical events
pub trait GraphNode: Send {
    fn render_block(&mut self, out: &mut [f32], ctx: &RenderCtx);

    /// Triggered when a note starts
    ///
    /// Default implementation does nothing (passthrough nodes).
    fn note_on(&mut self, _ctx: &RenderCtx) {
        // Default: do nothing
    }

    /// Triggered when a note is released
    ///
    /// Default implementation does nothing (passthrough nodes).
    fn note_off(&mut self, _ctx: &RenderCtx) {
        // Default: do nothing
    }

    fn get_envelope_level(&self) -> Option<f32> {
        None
    }

    /// Check if this node is still producing sound
    ///
    /// Used by voice management to know when a voice can be freed.
    fn is_active(&self) -> bool {
        true
    }
}

/// Allow boxed graph nodes to be used as graph nodes (for dynamic dispatch)
impl GraphNode for Box<dyn GraphNode> {
    fn render_block(&mut self, out: &mut [f32], ctx: &RenderCtx) {
        (**self).render_block(out, ctx)
    }

    fn note_on(&mut self, ctx: &RenderCtx) {
        (**self).note_on(ctx)
    }

    fn note_off(&mut self, ctx: &RenderCtx) {
        (**self).note_off(ctx)
    }

    fn get_envelope_level(&self) -> Option<f32> {
        (**self).get_envelope_level()
    }

    fn is_active(&self) -> bool {
        (**self).is_active()
    }
}
