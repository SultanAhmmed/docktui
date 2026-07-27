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
        .images
        .iter()
        .map(|img| {
            let line = Line::from(vec![
                ratatui::text::Span::styled(
                    format!("{:<30} ", format!("{}:{}", img.repository, img.tag)),
                    Style::default().add_modifier(Modifier::BOLD),
                ),
                ratatui::text::Span::styled(
                    format!("{:<15} ", img.size),
                    Style::default().fg(Color::Cyan),
                ),
                ratatui::text::Span::styled(
                    format!("{:<15} ", img.created_since),
                    Style::default().fg(Color::DarkGray),
                ),
                ratatui::text::Span::styled(
                    &img.id[..12.min(img.id.len())],
                    Style::default().fg(Color::Yellow),
                ),
            ]);
            ListItem::new(line)
        })
        .collect();

    let title = " Images | q:quit p:prune unused r:refresh 1-4:switch ";

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(title)
                .border_style(Style::default().fg(Color::Magenta)),
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
