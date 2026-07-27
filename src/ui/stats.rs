use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Sparkline},
    Frame,
};

use crate::app::App;

pub fn render(f: &mut Frame, app: &App, area: Rect) {
    let title = " Resource Monitor | r:refresh 1-4:switch q:quit ";
    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .border_style(Style::default().fg(Color::Yellow));

    // 1. Calculate inner area FIRST, before `block` is moved/consumed
    let inner = block.inner(area);

    // 2. Render the outer box (this consumes `block`)
    f.render_widget(block, area);

    // 3. Show error if fetching stats failed
    if let Some(err) = &app.stats_error {
        let err_widget = Paragraph::new(err.as_str())
            .style(Style::default().fg(Color::Red));
        f.render_widget(err_widget, inner);
        return;
    }

    // 4. Show helpful message if no containers are running
    if app.stats.is_empty() {
        let empty_widget = Paragraph::new("No running containers to monitor. Start one to see stats!")
            .style(Style::default().fg(Color::DarkGray));
        f.render_widget(empty_widget, inner);
        return;
    }

    // 5. Render stats for each container
    let mut y = inner.y;
    for (name, stats) in &app.stats {
        // Prevent drawing outside the bottom of the box
        if y + 4 > inner.y + inner.height {
            break;
        }

        let cpu_line = Line::from(vec![
            Span::styled(format!("{:<20} ", name), Style::default().fg(Color::Cyan)),
            Span::styled(format!("CPU: {:5.1}% ", stats.cpu_percent), Style::default().fg(Color::Yellow)),
            Span::styled(
                format!("Mem: {}MB / {}MB", stats.memory_usage / 1024 / 1024, stats.memory_limit / 1024 / 1024),
                Style::default().fg(Color::Green),
            ),
        ]);

        let cpu_area = Rect::new(inner.x, y, inner.width, 1);
        f.render_widget(Paragraph::new(cpu_line), cpu_area);
        y += 1;

        if !stats.history.is_empty() {
            let sparkline = Sparkline::default()
                .data(&stats.history)
                .max(1000) // 100.0% * 10 = 1000
                .style(Style::default().fg(Color::Yellow));

            let spark_area = Rect::new(inner.x, y, inner.width, 2);
            f.render_widget(sparkline, spark_area);
            y += 2;
        }
    }
}
