//! Timeline widget - shows pattern blocks with playhead

use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use super::{UiStateInit, UiStateUpdate};

/// Render the timeline with pattern blocks and playhead
pub fn render_timeline(
    frame: &mut Frame,
    area: Rect,
    static_state: &UiStateInit,
    dynamic_state: &UiStateUpdate,
) {
    if area.height < 2 || area.width < 20 {
        return;
    }

    let ticks_per_beat = static_state.ppq;
    let ticks_per_bar = ticks_per_beat * 4; // 4/4 time
    let total_bars = (static_state.total_ticks + ticks_per_bar - 1) / ticks_per_bar;

    // Calculate how many characters per bar based on available width
    let track_label_width = 8u16;
    let timeline_width = area.width.saturating_sub(track_label_width + 2);

    // Each bar gets equal space, minimum 4 chars per bar
    let chars_per_bar = (timeline_width as u32 / total_bars.max(1)).max(4);
    let chars_per_tick = chars_per_bar as f64 / ticks_per_bar as f64;

    // Calculate playhead position in characters
    let playhead_char = (dynamic_state.tick_position as f64 * chars_per_tick) as u16;

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
    for (track_idx, track) in static_state.tracks.iter().enumerate() {
        let mut spans = Vec::new();

        // Get dynamic state for this track
        let is_active = if track_idx < dynamic_state.num_tracks as usize {
            dynamic_state.track_states[track_idx].is_active
        } else {
            false
        };

        // Track name (padded)
        let name = if track.name.len() > 6 {
            format!("{:.6}  ", track.name)
        } else {
            format!("{:6}  ", track.name)
        };
        spans.push(Span::styled(
            name,
            Style::default().fg(if is_active {
                Color::White
            } else {
                Color::DarkGray
            }),
        ));

        // Build pattern visualization character by character
        // Use different characters to show note boundaries
        let base_color = if is_active {
            Color::Cyan
        } else {
            Color::DarkGray
        };

        // Sort events by start time for proper rendering
        let mut sorted_events = track.events.clone();
        sorted_events.sort_by_key(|(start, _)| *start);

        for char_idx in 0..timeline_width {
            let tick_pos = (char_idx as f64 / chars_per_tick) as u32;

            // Find which event (if any) is active at this tick
            let active_event = sorted_events.iter().find(|(start, duration)| {
                tick_pos >= *start && tick_pos < start + duration
            });

            let ch = if let Some((start, duration)) = active_event {
                // Check if this is the start of the note (first char)
                let note_start_char = (*start as f64 * chars_per_tick) as u16;
                let note_end_char = ((*start + *duration) as f64 * chars_per_tick) as u16;

                if char_idx == note_start_char {
                    // Note attack - bright marker
                    '█'
                } else if char_idx + 1 >= note_end_char {
                    // End of note - add gap
                    ' '
                } else {
                    // Sustain portion
                    '▓'
                }
            } else {
                // Rest/silence
                '░'
            };

            spans.push(Span::styled(
                ch.to_string(),
                Style::default().fg(base_color),
            ));
        }

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
