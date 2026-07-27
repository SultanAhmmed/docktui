use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{Block, Borders, List, ListItem, ListState},
    Frame,
};

use crate::app::App;

pub fn render(f: &mut Frame, app: &App, area: Rect) {
    let items: Vec<ListItem> = app
        .compose_projects
        .iter()
        .map(|proj| {
            let line = Line::from(vec![
                ratatui::text::Span::styled(
                    format!("{:<25} ", proj.name),
                    Style::default().add_modifier(Modifier::BOLD),
                ),
                ratatui::text::Span::styled(
                    format!("{:<20} ", proj.status),
                    Style::default().fg(Color::Green),
                ),
                ratatui::text::Span::styled(
                    &proj.config_files,
                    Style::default().fg(Color::DarkGray),
                ),
            ]);
            ListItem::new(line)
        })
        .collect();

    let title = " Compose Projects | q:quit u:up w:down r:refresh 1-4:switch ";

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(title)
                .border_style(Style::default().fg(Color::Cyan)),
        )
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▶ ");

    let mut state = ListState::default().with_selected(Some(app.selected));
    f.render_stateful_widget(list, area, &mut state);
}
