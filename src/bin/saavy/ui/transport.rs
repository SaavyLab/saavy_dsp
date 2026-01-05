//! Transport bar widget - shows BPM, play state, position, and audio stats

use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use super::{UiStateInit, UiStateUpdate};

/// Audio statistics for display
pub struct AudioStats {
    pub peak: f32,
    pub rms: f32,
}

impl AudioStats {
    /// Compute audio stats from a buffer
    pub fn from_buffer(buffer: &[f32]) -> Self {
        if buffer.is_empty() {
            return Self { peak: 0.0, rms: 0.0 };
        }
        let peak = buffer.iter().fold(0.0f32, |acc, &x| acc.max(x.abs()));
        let rms = (buffer.iter().map(|&x| x * x).sum::<f32>() / buffer.len() as f32).sqrt();
        Self { peak, rms }
    }
}

/// Render the transport bar
pub fn render_transport(
    frame: &mut Frame,
    area: Rect,
    static_state: &UiStateInit,
    dynamic_state: &UiStateUpdate,
    audio_stats: &AudioStats,
) {
    let block = Block::default()
        .title(" saavy ")
        .borders(Borders::ALL);

    // Calculate bar and beat from tick position
    let ticks_per_beat = static_state.ppq;
    let ticks_per_bar = ticks_per_beat * 4; // Assuming 4/4 time

    let current_bar = dynamic_state.tick_position / ticks_per_bar + 1;
    let current_beat = (dynamic_state.tick_position % ticks_per_bar) / ticks_per_beat + 1;

    // Build the status line
    let play_symbol = if dynamic_state.is_playing { "▶" } else { "⏸" };
    let play_state_str = if dynamic_state.is_playing { "Playing" } else { "Paused" };

    // Format sample rate nicely (e.g., 48000 -> "48kHz")
    let sample_rate_khz = static_state.sample_rate / 1000.0;

    let line = Line::from(vec![
        Span::styled(
            format!(" BPM: {:.0}  ", static_state.bpm),
            Style::default().fg(Color::Cyan),
        ),
        Span::styled(
            format!("{} {}  ", play_symbol, play_state_str),
            Style::default().fg(if dynamic_state.is_playing {
                Color::Green
            } else {
                Color::Yellow
            }),
        ),
        Span::styled(
            format!("Bar {} | Beat {}  ", current_bar, current_beat),
            Style::default().fg(Color::White),
        ),
        Span::styled(
            format!("{}/{}  ", dynamic_state.tick_position, static_state.total_ticks),
            Style::default().fg(Color::DarkGray),
        ),
        Span::raw("  "),
        Span::styled(
            format!("{:.1}kHz  ", sample_rate_khz),
            Style::default().fg(Color::DarkGray),
        ),
        Span::styled(
            format!("Peak: {:.2}  RMS: {:.2}", audio_stats.peak, audio_stats.rms),
            Style::default().fg(Color::Magenta),
        ),
    ]);

    let paragraph = Paragraph::new(line).block(block);
    frame.render_widget(paragraph, area);
}
