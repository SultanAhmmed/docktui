use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use crate::app::PendingAction;

pub fn render(f: &mut Frame, action: &PendingAction, area: Rect) {
    let message = match action {
        PendingAction::Start(id) => format!("Start container {}?", &id[..12.min(id.len())]),
        PendingAction::Stop(id) => format!("Stop container {}?", &id[..12.min(id.len())]),
        PendingAction::Restart(id) => format!("Restart container {}?", &id[..12.min(id.len())]),
        PendingAction::Remove(id) => format!("Remove container {}?", &id[..12.min(id.len())]),
        PendingAction::ComposeUp(name) => format!("docker compose up {}?", name),
        PendingAction::ComposeDown(name) => format!("docker compose down {}?", name),
        PendingAction::PruneImages => "Prune ALL unused images?".to_string(),
    };

    let dialog_width = 55;
    // FIX: Increased from 6 to 8 to fit all 5 text lines + 2 borders + 1 padding
    let dialog_height = 8;

    let x = area.x + (area.width.saturating_sub(dialog_width)) / 2;
    let y = area.y + (area.height.saturating_sub(dialog_height)) / 2;
    let dialog_area = Rect::new(x, y, dialog_width, dialog_height);

    f.render_widget(Clear, dialog_area);

    let text = vec![
        Line::from(""), // Top padding
        Line::from(Span::styled(
            "[!] Confirm Action",
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(message, Style::default().fg(Color::White))),
        Line::from(""),
        Line::from(vec![
            Span::styled("Press ", Style::default().fg(Color::White)),
            Span::styled("y", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
            Span::styled(" to confirm, ", Style::default().fg(Color::White)),
            Span::styled("n", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
            Span::styled(" to cancel", Style::default().fg(Color::White)),
        ]),
        Line::from(""), // Bottom padding
    ];

    let paragraph = Paragraph::new(text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Yellow)),
        )
        .alignment(Alignment::Center);

    f.render_widget(paragraph, dialog_area);
}
