//! Transport bar widget - shows BPM, play state, and position

use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use super::UiState;

/// Render the transport bar
pub fn render_transport(frame: &mut Frame, area: Rect, state: &UiState) {
    let block = Block::default()
        .title(" saavy ")
        .borders(Borders::ALL);

    // Calculate bar and beat from tick position
    let ticks_per_beat = state.ppq;
    let ticks_per_bar = ticks_per_beat * 4; // Assuming 4/4 time

    let current_bar = state.tick_position / ticks_per_bar + 1;
    let current_beat = (state.tick_position % ticks_per_bar) / ticks_per_beat + 1;

    // Build the status line
    let play_symbol = if state.is_playing { "▶" } else { "⏸" };
    let play_state = if state.is_playing { "Playing" } else { "Paused" };

    let line = Line::from(vec![
        Span::styled(
            format!(" BPM: {:.0}  ", state.bpm),
            Style::default().fg(Color::Cyan),
        ),
        Span::styled(
            format!("{} {}  ", play_symbol, play_state),
            Style::default().fg(if state.is_playing {
                Color::Green
            } else {
                Color::Yellow
            }),
        ),
        Span::raw("    "),
        Span::styled(
            format!("Bar {} | Beat {}  ", current_bar, current_beat),
            Style::default().fg(Color::White),
        ),
        Span::styled(
            format!("{}/{}", state.tick_position, state.total_ticks),
            Style::default().fg(Color::DarkGray),
        ),
    ]);

    let paragraph = Paragraph::new(line).block(block);
    frame.render_widget(paragraph, area);
}
