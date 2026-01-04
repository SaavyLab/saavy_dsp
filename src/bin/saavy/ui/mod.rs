//! TUI module for saavy
//!
//! Provides real-time visualization of audio output and pattern playback.

pub mod state;
mod timeline;
mod transport;
mod waveform;

use color_eyre::eyre::Result as EyreResult;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    widgets::{Block, Borders},
    DefaultTerminal, Frame,
};
use rtrb::Consumer;
use std::time::Duration;

pub use state::{ControlMessage, TrackDynamicState, TrackStaticInfo, UiStateInit, UiStateUpdate};

use timeline::render_timeline;
use transport::render_transport;
use waveform::render_waveform;

/// Audio visualization buffer size
const VIS_BUFFER_SIZE: usize = 1024;

/// UI application state
pub struct UiApp {
    /// Ring buffer receiver for audio samples
    audio_rx: Consumer<f32>,
    /// Ring buffer receiver for UI state updates (allocation-free)
    state_rx: Consumer<UiStateUpdate>,
    /// Ring buffer sender for control messages
    control_tx: rtrb::Producer<ControlMessage>,
    /// Static state (set once at init, never changes)
    static_state: UiStateInit,
    /// Current dynamic state (updated from audio thread)
    dynamic_state: UiStateUpdate,
    /// Audio sample buffer for visualization
    audio_buffer: Vec<f32>,
    /// Whether the app should quit
    should_quit: bool,
}

impl UiApp {
    /// Create a new UI application
    pub fn new(
        audio_rx: Consumer<f32>,
        state_rx: Consumer<UiStateUpdate>,
        control_tx: rtrb::Producer<ControlMessage>,
        static_state: UiStateInit,
    ) -> Self {
        Self {
            audio_rx,
            state_rx,
            control_tx,
            static_state,
            dynamic_state: UiStateUpdate::new(),
            audio_buffer: vec![0.0; VIS_BUFFER_SIZE],
            should_quit: false,
        }
    }

    /// Run the UI event loop
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> EyreResult<()> {
        while !self.should_quit {
            // Poll for new audio samples
            self.poll_audio();

            // Poll for state updates
            self.poll_state();

            // Draw the UI
            terminal.draw(|frame| self.render(frame))?;

            // Handle keyboard input (non-blocking, ~60fps)
            if event::poll(Duration::from_millis(16))? {
                if let Event::Key(key) = event::read()? {
                    if key.kind == KeyEventKind::Press {
                        self.handle_key(key.code);
                    }
                }
            }
        }

        Ok(())
    }

    /// Poll for new audio samples from ring buffer
    fn poll_audio(&mut self) {
        // Read as many samples as available, keeping last VIS_BUFFER_SIZE
        let mut new_samples = Vec::new();
        while let Ok(sample) = self.audio_rx.pop() {
            new_samples.push(sample);
        }

        if !new_samples.is_empty() {
            // Append new samples and keep only the last VIS_BUFFER_SIZE
            self.audio_buffer.extend(new_samples);
            if self.audio_buffer.len() > VIS_BUFFER_SIZE {
                let excess = self.audio_buffer.len() - VIS_BUFFER_SIZE;
                self.audio_buffer.drain(0..excess);
            }
        }
    }

    /// Poll for state updates from ring buffer
    fn poll_state(&mut self) {
        // Keep only the latest state update
        while let Ok(update) = self.state_rx.pop() {
            self.dynamic_state = update;
        }
    }

    /// Handle keyboard input
    fn handle_key(&mut self, key: KeyCode) {
        match key {
            KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => {
                self.should_quit = true;
            }
            KeyCode::Char(' ') => {
                let _ = self.control_tx.push(ControlMessage::TogglePlayback);
            }
            KeyCode::Char('r') | KeyCode::Char('R') => {
                let _ = self.control_tx.push(ControlMessage::Reset);
            }
            _ => {}
        }
    }

    /// Render the UI
    fn render(&self, frame: &mut Frame) {
        let area = frame.area();

        // Main layout: transport, timeline, waveform, help
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),  // Transport bar
                Constraint::Min(6),     // Timeline
                Constraint::Length(8),  // Waveform
                Constraint::Length(1),  // Help bar
            ])
            .split(area);

        // Transport bar
        render_transport(frame, chunks[0], &self.static_state, &self.dynamic_state);

        // Timeline with pattern blocks
        let timeline_block = Block::default()
            .title(" Timeline ")
            .borders(Borders::ALL);
        let timeline_inner = timeline_block.inner(chunks[1]);
        frame.render_widget(timeline_block, chunks[1]);
        render_timeline(frame, timeline_inner, &self.static_state, &self.dynamic_state);

        // Waveform oscilloscope
        render_waveform(frame, chunks[2], &self.audio_buffer);

        // Help bar
        let help = ratatui::widgets::Paragraph::new(
            " [Q] Quit  [Space] Play/Pause  [R] Reset"
        )
        .style(ratatui::style::Style::default().fg(ratatui::style::Color::DarkGray));
        frame.render_widget(help, chunks[3]);
    }
}
