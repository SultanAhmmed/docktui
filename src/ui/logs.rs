use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

use crate::app::App;

pub fn render(f: &mut Frame, app: &App, area: Rect) {
    let lines: Vec<Line> = app
        .log_lines
        .iter()
        .map(|line| {
            let color = if line.to_lowercase().contains("error") {
                Color::Red
            } else if line.to_lowercase().contains("warn") {
                Color::Yellow
            } else {
                Color::White
            };
            Line::from(Span::styled(line.clone(), Style::default().fg(color)))
        })
        .collect();

    let title = format!(" Logs: {} (Esc to close) ", app.log_container_name);

    let paragraph = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(title)
                .border_style(Style::default().fg(Color::Green)),
        )
        .wrap(Wrap { trim: false })
        .scroll((app.log_lines.len().saturating_sub(area.height as usize) as u16, 0));

    f.render_widget(paragraph, area);
}
