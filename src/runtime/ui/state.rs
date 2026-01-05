//! Shared state types for UI communication
//!
//! Designed for real-time safety: static data is sent once at init,
//! dynamic updates are allocation-free.

/// Commands sent from UI thread to audio thread
#[derive(Clone, Copy, Debug)]
pub enum ControlMessage {
    /// Toggle play/pause
    TogglePlayback,
    /// Reset to beginning
    Reset,
}

/// Static state sent once at initialization (can allocate)
#[derive(Clone)]
pub struct UiStateInit {
    /// Tempo in BPM
    pub bpm: f64,
    /// Pulses per quarter note
    pub ppq: u32,
    /// Total duration in ticks
    pub total_ticks: u32,
    /// Audio sample rate in Hz
    pub sample_rate: f32,
    /// Per-track static info (names, patterns)
    pub tracks: Vec<TrackStaticInfo>,
}

/// Static information about a track (sent once, never in audio callback)
#[derive(Clone)]
pub struct TrackStaticInfo {
    /// Track name
    pub name: String,
    /// Pattern events for timeline visualization (tick, duration)
    pub events: Vec<(u32, u32)>,
}

/// Dynamic state update sent from audio thread (allocation-free, Copy)
#[derive(Clone, Copy, Debug)]
pub struct UiStateUpdate {
    /// Current position in ticks
    pub tick_position: u32,
    /// Whether playback is active
    pub is_playing: bool,
    /// Per-track dynamic state (fixed-size array for up to 8 tracks)
    pub track_states: [TrackDynamicState; 8],
    /// Number of active tracks
    pub num_tracks: u8,
}

/// Dynamic state for a single track (Copy, no allocations)
#[derive(Clone, Copy, Debug, Default)]
pub struct TrackDynamicState {
    /// Whether the track is currently producing sound
    pub is_active: bool,
    /// Current envelope level (0.0-1.0)
    pub envelope_level: f32,
    /// Current note being played (0 = none, 1-127 = MIDI note)
    pub current_note: u8,
}

impl UiStateInit {
    /// Create initial UI state
    pub fn new(bpm: f64, ppq: u32, total_ticks: u32, sample_rate: f32, tracks: Vec<TrackStaticInfo>) -> Self {
        Self {
            bpm,
            ppq,
            total_ticks,
            sample_rate,
            tracks,
        }
    }
}

impl UiStateUpdate {
    /// Create a new update with default values
    pub fn new() -> Self {
        Self {
            tick_position: 0,
            is_playing: true,
            track_states: [TrackDynamicState::default(); 8],
            num_tracks: 0,
        }
    }
}
