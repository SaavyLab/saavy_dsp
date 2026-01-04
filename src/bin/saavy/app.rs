//! Saavy - main application builder and runner

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use color_eyre::eyre::{eyre, Result as EyreResult, WrapErr};
use std::sync::{Arc, Mutex};

use super::sequencer::Sequencer;
use super::track::Track;

use saavy_dsp::{
    graph::GraphNode,
    sequencing::{Pattern, PatternChain, Sequence},
    MAX_BLOCK_SIZE,
};

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

        println!("=== Saavy ===");
        println!("BPM: {}", self.bpm);
        println!("Sample rate: {} Hz", sample_rate);
        println!("Channels: {}", channels);
        println!();

        // Calculate total duration from tracks
        let mut total_ticks = 0u32;
        for track in &self.tracks {
            println!("  Track: {} ({} events, {} ticks)",
                track.name,
                track.sequence.events.len(),
                track.sequence.total_ticks
            );
            total_ticks = total_ticks.max(track.sequence.total_ticks);
        }

        println!();
        println!("Total duration: {} ticks", total_ticks);
        println!("Playing... Press Ctrl+C to stop");
        println!();

        // Create sequencer
        let mut sequencer = Sequencer::new(self.bpm, self.ppq, sample_rate as f64, self.tracks.len());
        sequencer.set_total_ticks(total_ticks);

        // Wrap in Arc<Mutex> for sharing with audio thread
        let state = Arc::new(Mutex::new(AudioState {
            tracks: self.tracks,
            sequencer,
            sample_rate,
        }));

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
                let AudioState { tracks, sequencer, sample_rate } = &mut *state;
                let sample_rate = *sample_rate;

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

                    frames_written += frames_to_render;
                }
            },
            |err| eprintln!("Audio error: {}", err),
            None,
        )?;

        stream.play()?;

        // For now, just loop forever
        // TODO: Add TUI
        loop {
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
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
