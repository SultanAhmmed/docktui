use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame,
};

use crate::app::{App, InputMode};

pub fn render(f: &mut Frame, app: &App, area: Rect) {
    let items: Vec<ListItem> = app
        .filtered_containers
        .iter()
        .map(|c| {
            let status_color = if c.is_running() { Color::Green } else { Color::Red };
            let is_favorite = app.favorites.container_ids.contains(&c.id);
            let favorite_marker = if is_favorite { "★ " } else { "  " };

            let line = Line::from(vec![
                Span::styled(favorite_marker, Style::default().fg(Color::Yellow)),
                Span::styled("● ", Style::default().fg(status_color)),
                Span::styled(format!("{:<20}", truncate(&c.names, 19)), Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(format!("{:<25} ", truncate(&c.image, 24))),
                Span::styled(format!("{:<18} ", truncate(&c.status, 17)), Style::default().fg(Color::DarkGray)),
                Span::styled(truncate(&c.ports, 25), Style::default().fg(Color::Cyan)),
            ]);
            ListItem::new(line)
        })
        .collect();


    let title = format!(
        " Containers ({}) | q:quit 1-4:switch ↑↓:nav Enter:logs s:start x:stop e:restart d:remove /:filter f:favorite ",
        app.filtered_containers.len()
    );  

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(title)
                .border_style(Style::default().fg(Color::Blue)),
        )
        .highlight_style(Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD))
        .highlight_symbol("▶ ");

    let mut state = ListState::default().with_selected(Some(app.selected));
    f.render_stateful_widget(list, area, &mut state);

    if let Some(msg) = &app.status_message {
        let msg_area = Rect::new(area.x, area.y + area.height - 2, area.width, 1);
        let color = if msg.starts_with("✓") { Color::Green } else { Color::Red };
        f.render_widget(Paragraph::new(msg.as_str()).style(Style::default().fg(color)), msg_area);
    }

    if app.input_mode == InputMode::Filtering {
        let footer_area = Rect::new(area.x, area.y + area.height - 1, area.width, 1);
        let footer_text = format!("🔍 Filter: {}_  (Press Enter or Esc to close)", app.filter_text);
        f.render_widget(Paragraph::new(footer_text).style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)), footer_area);
    }
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max { s.to_string() } else { format!("{}…", &s[..max.saturating_sub(1)]) }
}
