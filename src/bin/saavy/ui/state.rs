//! Shared state types for UI communication

/// State sent from audio thread to UI thread
#[derive(Clone)]
pub struct UiState {
    /// Current position in ticks
    pub tick_position: u32,
    /// Total duration in ticks
    pub total_ticks: u32,
    /// Tempo in BPM
    pub bpm: f64,
    /// Pulses per quarter note
    pub ppq: u32,
    /// Whether playback is active
    pub is_playing: bool,
    /// Per-track activity info
    pub track_info: Vec<TrackInfo>,
}

/// Information about a single track for UI display
#[derive(Clone)]
pub struct TrackInfo {
    /// Track name
    pub name: String,
    /// Whether the track is currently producing sound
    pub is_active: bool,
    /// Current envelope level (0.0-1.0)
    pub envelope_level: f32,
    /// Current note being played (if any)
    pub current_note: Option<u8>,
    /// Pattern events for timeline visualization (tick, duration)
    pub events: Vec<(u32, u32)>,
}

impl UiState {
    /// Create initial UI state
    pub fn new(bpm: f64, ppq: u32, total_ticks: u32, track_info: Vec<TrackInfo>) -> Self {
        Self {
            tick_position: 0,
            total_ticks,
            bpm,
            ppq,
            is_playing: true,
            track_info,
        }
    }
}
