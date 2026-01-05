//! Saavy - main application builder and runner

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use color_eyre::eyre::{eyre, Result as EyreResult, WrapErr};
use rtrb::RingBuffer;
use std::sync::{Arc, Mutex};

use super::sequencer::Sequencer;
use super::track::Track;
use super::ui::{ControlMessage, TrackDynamicState, TrackStaticInfo, UiApp, UiStateInit, UiStateUpdate};

use saavy_dsp::{
    graph::GraphNode,
    sequencing::{Pattern, PatternChain, Sequence},
    MAX_BLOCK_SIZE,
};

/// Ring buffer capacity for audio samples (enough for ~340ms at 48kHz)
const AUDIO_RING_SIZE: usize = 16384;
/// Ring buffer capacity for UI state updates
const STATE_RING_SIZE: usize = 32;
/// Ring buffer capacity for control messages
const CONTROL_RING_SIZE: usize = 64;

/// Main application builder
pub struct Saavy {
    bpm: f64,
    ppq: u32,
    tracks: Vec<Track>,
}

impl Saavy {
    /// Create a new Saavy instance
    pub fn new() -> Self {
        Self {
            bpm: 120.0,
            ppq: 480,
            tracks: Vec::new(),
        }
    }

    /// Set the tempo in beats per minute
    pub fn bpm(mut self, bpm: f64) -> Self {
        self.bpm = bpm;
        self
    }

    /// Add a track with a pattern and audio node
    ///
    /// Each track is monophonic (one voice). For polyphony, create multiple tracks.
    pub fn track<N: GraphNode + 'static>(
        mut self,
        name: &str,
        pattern: impl IntoSequence,
        node: N,
    ) -> Self {
        let sequence = pattern.into_sequence(self.ppq);
        self.tracks.push(Track::new(name, sequence, node));
        self
    }

    /// Run the application (takes over, plays audio)
    pub fn run(self) -> EyreResult<()> {
        // Set up audio
        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .ok_or_else(|| eyre!("no default output device available"))?;
        let config = device
            .default_output_config()
            .wrap_err("failed to fetch default output config")?;

        let sample_rate = config.sample_rate().0 as f32;
        let channels = config.channels() as usize;

        // Calculate total duration and build static track info for UI (sent once, can allocate)
        let mut total_ticks = 0u32;
        let tracks_static: Vec<TrackStaticInfo> = self
            .tracks
            .iter()
            .map(|track| {
                total_ticks = total_ticks.max(track.sequence.total_ticks);
                TrackStaticInfo {
                    name: track.name.clone(),
                    events: track
                        .sequence
                        .events
                        .iter()
                        .filter_map(|e| e.note.map(|_| (e.tick_offset, e.duration_ticks)))
                        .collect(),
                }
            })
            .collect();

        let num_tracks = self.tracks.len().min(8) as u8;

        // Create ring buffers for audio↔UI communication
        let (audio_tx, audio_rx) = RingBuffer::<f32>::new(AUDIO_RING_SIZE);
        let (state_tx, state_rx) = RingBuffer::<UiStateUpdate>::new(STATE_RING_SIZE);
        let (control_tx, control_rx) = RingBuffer::<ControlMessage>::new(CONTROL_RING_SIZE);

        // Static UI state (sent once at init, never changes)
        let static_state = UiStateInit::new(self.bpm, self.ppq, total_ticks, sample_rate, tracks_static);

        // Create sequencer
        let sequencer = Sequencer::new(self.bpm, self.ppq, sample_rate as f64, self.tracks.len());

        // Wrap in Arc<Mutex> for sharing with audio thread
        let state = Arc::new(Mutex::new(AudioState {
            tracks: self.tracks,
            sequencer,
            sample_rate,
            num_tracks,
            audio_tx,
            state_tx,
            control_rx,
        }));
        state.lock().unwrap().sequencer.set_total_ticks(total_ticks);

        // Set up audio stream
        let state_clone = state.clone();
        let mut render_buf = vec![0.0f32; MAX_BLOCK_SIZE];
        let mut track_buf = vec![0.0f32; MAX_BLOCK_SIZE];

        let stream = device.build_output_stream(
            &config.into(),
            move |data: &mut [f32], _| {
                let mut state = state_clone.lock().unwrap();
                let total_frames = data.len() / channels;
                let mut frames_written = 0;

                // Destructure to allow simultaneous mutable borrows
                let AudioState {
                    tracks,
                    sequencer,
                    sample_rate,
                    num_tracks,
                    audio_tx,
                    state_tx,
                    control_rx,
                } = &mut *state;
                let sample_rate = *sample_rate;
                let num_tracks = *num_tracks;

                // Process control messages from UI
                while let Ok(msg) = control_rx.pop() {
                    match msg {
                        ControlMessage::TogglePlayback => sequencer.toggle(),
                        ControlMessage::Reset => sequencer.reset(),
                    }
                }

                while frames_written < total_frames {
                    let frames_remaining = total_frames - frames_written;
                    let frames_to_render = frames_remaining.min(MAX_BLOCK_SIZE);

                    // Process sequencer (triggers note events)
                    sequencer.process_block(frames_to_render, tracks, sample_rate);

                    // Clear render buffer
                    let block = &mut render_buf[..frames_to_render];
                    block.fill(0.0);

                    // Render and mix all tracks
                    for track in tracks.iter_mut() {
                        let tbuf = &mut track_buf[..frames_to_render];
                        tbuf.fill(0.0);
                        track.render(tbuf, sample_rate);

                        // Mix into main buffer
                        for (out, &sample) in block.iter_mut().zip(tbuf.iter()) {
                            *out += sample;
                        }
                    }

                    // Copy to output (mono to all channels)
                    let out_off = frames_written * channels;
                    for (i, &s) in block.iter().enumerate() {
                        for ch in 0..channels {
                            data[out_off + i * channels + ch] = s;
                        }
                    }

                    // Push audio samples to UI (non-blocking, drop on overflow)
                    for &sample in &block[..frames_to_render] {
                        let _ = audio_tx.push(sample);
                    }

                    frames_written += frames_to_render;
                }

                // Push UI state update (once per callback, allocation-free)
                let mut track_states = [TrackDynamicState::default(); 8];
                for (i, track) in tracks.iter().enumerate().take(8) {
                    track_states[i] = TrackDynamicState {
                        is_active: track.is_active(),
                        envelope_level: track.envelope_level().unwrap_or(0.0),
                        current_note: track.current_note().unwrap_or(0),
                    };
                }

                let ui_update = UiStateUpdate {
                    tick_position: sequencer.tick_position(),
                    is_playing: sequencer.is_playing(),
                    track_states,
                    num_tracks,
                };
                let _ = state_tx.push(ui_update);
            },
            |err| eprintln!("Audio error: {}", err),
            None,
        )?;

        stream.play()?;

        // Initialize terminal and run TUI
        let mut terminal = ratatui::init();
        let mut ui = UiApp::new(audio_rx, state_rx, control_tx, static_state);
        let result = ui.run(&mut terminal);
        ratatui::restore();

        result
    }
}

impl Default for Saavy {
    fn default() -> Self {
        Self::new()
    }
}

/// Shared audio state
struct AudioState {
    tracks: Vec<Track>,
    sequencer: Sequencer,
    sample_rate: f32,
    num_tracks: u8,
    audio_tx: rtrb::Producer<f32>,
    state_tx: rtrb::Producer<UiStateUpdate>,
    control_rx: rtrb::Consumer<ControlMessage>,
}

/// Trait for types that can be converted to a Sequence
pub trait IntoSequence {
    fn into_sequence(self, ppq: u32) -> Sequence;
}

impl IntoSequence for Pattern {
    fn into_sequence(self, ppq: u32) -> Sequence {
        self.to_sequence(ppq)
    }
}

impl IntoSequence for PatternChain {
    fn into_sequence(self, ppq: u32) -> Sequence {
        self.to_sequence(ppq)
    }
}

impl IntoSequence for Sequence {
    fn into_sequence(self, _ppq: u32) -> Sequence {
        self
    }
}
