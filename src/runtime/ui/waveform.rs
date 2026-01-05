//! Waveform oscilloscope widget

use ratatui::{
    layout::Rect,
    style::{Color, Style},
    symbols,
    widgets::{Axis, Block, Borders, Chart, Dataset, GraphType},
    Frame,
};

/// Render the waveform oscilloscope
pub fn render_waveform(frame: &mut Frame, area: Rect, audio_buffer: &[f32]) {
    let block = Block::default()
        .title(" Waveform ")
        .borders(Borders::ALL);

    // Convert audio samples to chart data points
    let data: Vec<(f64, f64)> = audio_buffer
        .iter()
        .enumerate()
        .map(|(i, &sample)| {
            let x = i as f64 / audio_buffer.len() as f64;
            let y = sample as f64;
            (x, y)
        })
        .collect();

    let dataset = Dataset::default()
        .marker(symbols::Marker::Braille)
        .graph_type(GraphType::Line)
        .style(Style::default().fg(Color::Cyan))
        .data(&data);

    let chart = Chart::new(vec![dataset])
        .block(block)
        .x_axis(
            Axis::default()
                .bounds([0.0, 1.0])
                .style(Style::default().fg(Color::DarkGray)),
        )
        .y_axis(
            Axis::default()
                .bounds([-1.0, 1.0])
                .style(Style::default().fg(Color::DarkGray)),
        );

    frame.render_widget(chart, area);
}
