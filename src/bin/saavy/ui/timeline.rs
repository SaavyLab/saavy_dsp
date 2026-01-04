//! Timeline widget - shows pattern blocks with playhead

use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use super::UiState;

/// Render the timeline with pattern blocks and playhead
pub fn render_timeline(frame: &mut Frame, area: Rect, state: &UiState) {
    if area.height < 2 || area.width < 20 {
        return;
    }

    let ticks_per_beat = state.ppq;
    let ticks_per_bar = ticks_per_beat * 4; // 4/4 time
    let total_bars = (state.total_ticks + ticks_per_bar - 1) / ticks_per_bar;

    // Calculate how many characters per bar based on available width
    let track_label_width = 8u16;
    let timeline_width = area.width.saturating_sub(track_label_width + 2);

    // Each bar gets equal space, minimum 4 chars per bar
    let chars_per_bar = (timeline_width as u32 / total_bars.max(1)).max(4);
    let chars_per_tick = chars_per_bar as f64 / ticks_per_bar as f64;

    // Calculate playhead position in characters
    let playhead_char = (state.tick_position as f64 * chars_per_tick) as u16;

    let mut lines = Vec::new();

    // Beat markers row
    let mut beat_markers = String::new();
    beat_markers.push_str(&" ".repeat(track_label_width as usize));
    for bar in 0..total_bars {
        let bar_str = format!("|{}", bar + 1);
        beat_markers.push_str(&bar_str);
        let remaining = chars_per_bar as usize - bar_str.len();
        beat_markers.push_str(&" ".repeat(remaining.min(timeline_width as usize)));
    }
    lines.push(Line::from(Span::styled(
        beat_markers,
        Style::default().fg(Color::DarkGray),
    )));

    // Track rows
    for track in &state.track_info {
        let mut spans = Vec::new();

        // Track name (padded)
        let name = if track.name.len() > 6 {
            format!("{:.6}  ", track.name)
        } else {
            format!("{:6}  ", track.name)
        };
        spans.push(Span::styled(
            name,
            Style::default().fg(if track.is_active {
                Color::White
            } else {
                Color::DarkGray
            }),
        ));

        // Pattern blocks
        let mut pattern_str = String::new();
        for tick in 0..(timeline_width as u32) {
            let tick_pos = (tick as f64 / chars_per_tick) as u32;

            // Check if any event is active at this tick
            let is_note_on = track.events.iter().any(|(start, duration)| {
                tick_pos >= *start && tick_pos < start + duration
            });

            if is_note_on {
                pattern_str.push('▓');
            } else {
                pattern_str.push('░');
            }
        }

        spans.push(Span::styled(
            pattern_str,
            Style::default().fg(if track.is_active {
                Color::Cyan
            } else {
                Color::DarkGray
            }),
        ));

        lines.push(Line::from(spans));
    }

    // Playhead row
    let mut playhead_str = String::new();
    playhead_str.push_str(&" ".repeat(track_label_width as usize));
    for i in 0..timeline_width {
        if i == playhead_char {
            playhead_str.push('▲');
        } else {
            playhead_str.push(' ');
        }
    }
    lines.push(Line::from(Span::styled(
        playhead_str,
        Style::default().fg(Color::Yellow),
    )));

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, area);
}
