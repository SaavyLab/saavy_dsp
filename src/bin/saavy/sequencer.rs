//! Sequencer - sample-accurate pattern playback
//!
//! The Sequencer runs in the audio thread and converts tick-based
//! pattern timing into sample-accurate note events.

use super::track::Track;

/// Playback state for a single track
struct TrackPlayback {
    /// Index into the track's sequence events
    event_index: usize,
    /// Active notes: (note, end_tick)
    active_notes: Vec<(u8, u32)>,
}

impl TrackPlayback {
    fn new() -> Self {
        Self {
            event_index: 0,
            active_notes: Vec::with_capacity(16),
        }
    }

    fn reset(&mut self) {
        self.event_index = 0;
        self.active_notes.clear();
    }
}

/// Sample-accurate sequencer that drives multiple tracks
#[allow(dead_code)] // Fields retained for TUI display
pub struct Sequencer {
    /// Tempo in beats per minute
    bpm: f64,
    /// Pulses per quarter note (tick resolution)
    ppq: u32,
    /// Audio sample rate
    sample_rate: f64,
    /// Current position in ticks (fractional for sub-tick accuracy)
    tick_position: f64,
    /// Samples per tick (computed from bpm, ppq, sample_rate)
    samples_per_tick: f64,
    /// Per-track playback state
    track_states: Vec<TrackPlayback>,
    /// Whether playback is active
    playing: bool,
    /// Whether to loop
    looping: bool,
    /// Total duration in ticks (max of all tracks)
    total_ticks: u32,
}

impl Sequencer {
    /// Create a new sequencer
    pub fn new(bpm: f64, ppq: u32, sample_rate: f64, num_tracks: usize) -> Self {
        let samples_per_tick = Self::compute_samples_per_tick(bpm, ppq, sample_rate);

        Self {
            bpm,
            ppq,
            sample_rate,
            tick_position: 0.0,
            samples_per_tick,
            track_states: (0..num_tracks).map(|_| TrackPlayback::new()).collect(),
            playing: true,
            looping: true,
            total_ticks: 0,
        }
    }

    /// Compute samples per tick from tempo
    fn compute_samples_per_tick(bpm: f64, ppq: u32, sample_rate: f64) -> f64 {
        // ticks per second = (bpm / 60) * ppq
        // samples per tick = sample_rate / ticks_per_second
        let ticks_per_second = (bpm / 60.0) * ppq as f64;
        sample_rate / ticks_per_second
    }

    /// Set the total duration from tracks
    pub fn set_total_ticks(&mut self, total_ticks: u32) {
        self.total_ticks = total_ticks;
    }

    /// Set BPM (can be called at any time)
    pub fn set_bpm(&mut self, bpm: f64) {
        self.bpm = bpm;
        self.samples_per_tick = Self::compute_samples_per_tick(bpm, self.ppq, self.sample_rate);
    }

    /// Get current tick position
    pub fn tick_position(&self) -> u32 {
        self.tick_position as u32
    }

    /// Process one block of samples, triggering note events on tracks
    ///
    /// This should be called from the audio callback before rendering tracks.
    pub fn process_block(&mut self, block_size: usize, tracks: &mut [Track], sample_rate: f32) {
        if !self.playing {
            return;
        }

        // Process each sample in the block
        for _ in 0..block_size {
            let current_tick = self.tick_position as u32;

            // Process each track
            for (track_idx, track) in tracks.iter_mut().enumerate() {
                if track_idx >= self.track_states.len() {
                    continue;
                }

                let state = &mut self.track_states[track_idx];

                // Collect note-on events (to avoid borrow conflict)
                let mut note_ons: Vec<(u8, u8, u32)> = Vec::new();
                while state.event_index < track.sequence.events.len() {
                    let event = &track.sequence.events[state.event_index];
                    let event_tick = event.tick_offset.saturating_add_signed(event.offset_ticks);

                    if event_tick <= current_tick {
                        if let Some(note) = event.note {
                            let end_tick = current_tick + event.duration_ticks;
                            note_ons.push((note, event.velocity, end_tick));
                        }
                        state.event_index += 1;
                    } else {
                        break;
                    }
                }

                // Now trigger note-ons
                for (note, velocity, end_tick) in note_ons {
                    track.note_on(note, velocity, sample_rate);
                    state.active_notes.push((note, end_tick));
                }

                // Collect note-offs
                let mut note_offs: Vec<u8> = Vec::new();
                state.active_notes.retain(|&(note, end_tick)| {
                    if current_tick >= end_tick {
                        note_offs.push(note);
                        false
                    } else {
                        true
                    }
                });

                // Trigger note-offs
                for note in note_offs {
                    track.note_off(note, sample_rate);
                }
            }

            // Advance time
            self.tick_position += 1.0 / self.samples_per_tick;

            // Handle looping
            if self.tick_position >= self.total_ticks as f64 {
                if self.looping {
                    self.tick_position = 0.0;
                    // Reset all track states
                    for state in &mut self.track_states {
                        state.reset();
                    }
                } else {
                    self.playing = false;
                }
            }
        }
    }

    /// Reset playback to the beginning
    pub fn reset(&mut self) {
        self.tick_position = 0.0;
        for state in &mut self.track_states {
            state.reset();
        }
    }

    /// Start playback
    pub fn play(&mut self) {
        self.playing = true;
    }

    /// Pause playback
    pub fn pause(&mut self) {
        self.playing = false;
    }

    /// Toggle play/pause
    pub fn toggle(&mut self) {
        self.playing = !self.playing;
    }

    /// Check if playing
    pub fn is_playing(&self) -> bool {
        self.playing
    }
}
