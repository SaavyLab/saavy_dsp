//! Transport bar widget - shows BPM, play state, and position

use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use super::{UiStateInit, UiStateUpdate};

/// Render the transport bar
pub fn render_transport(
    frame: &mut Frame,
    area: Rect,
    static_state: &UiStateInit,
    dynamic_state: &UiStateUpdate,
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
        Span::raw("    "),
        Span::styled(
            format!("Bar {} | Beat {}  ", current_bar, current_beat),
            Style::default().fg(Color::White),
        ),
        Span::styled(
            format!("{}/{}", dynamic_state.tick_position, static_state.total_ticks),
            Style::default().fg(Color::DarkGray),
        ),
    ]);

    let paragraph = Paragraph::new(line).block(block);
    frame.render_widget(paragraph, area);
}
