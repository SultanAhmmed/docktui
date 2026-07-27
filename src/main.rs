use std::io;

use anyhow::Result;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    Terminal,
};

mod app;
mod docker;
mod events;
mod ui;

use app::{App, View};
use events::{poll_action, Action};

#[tokio::main]
async fn main() -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = run_app(&mut terminal).await;

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    result
}

async fn run_app(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
    let mut app = App::new().await?;

    while app.running {
        app.drain_logs();
        app.update_stats().await; // Now has a 1-second cooldown internally

        terminal.draw(|f| {
            // 1. Draw the current view
            match app.view {
                View::Containers => ui::containers::render(f, &app, f.area()),
                View::Logs => {
                    let chunks = Layout::default()
                        .direction(Direction::Horizontal)
                        .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
                        .split(f.area());
                    ui::containers::render(f, &app, chunks[0]);
                    ui::logs::render(f, &app, chunks[1]);
                }
                View::Images => ui::images::render(f, &app, f.area()),
                View::Compose => ui::compose::render(f, &app, f.area()),
                View::Stats => ui::stats::render(f, &app, f.area()),
            }

            // 2. GLOBAL OVERLAY: Draw confirmation dialog on top of ANY view if pending
            if let Some(pending) = &app.pending_action {
                ui::dialog::render(f, pending, f.area());
            }
        })?;

        match poll_action(100, &app.view, &app.input_mode) {
            Action::Quit => app.running = false,
            Action::Next => app.next(),
            Action::Previous => app.previous(),
            Action::Refresh => app.refresh().await?,
            Action::OpenLogs => app.open_logs(),
            Action::CloseLogs => app.close_logs(),
            Action::Start => app.start_container(),
            Action::Stop => app.stop_container(),
            Action::Restart => app.restart_container(),
            Action::Remove => app.remove_container(),
            Action::Confirm => app.confirm_action().await?,
            Action::Cancel => app.cancel_action(),
            Action::SwitchView(view) => app.switch_view(view),
            Action::ToggleFilter => app.toggle_filter(),
            Action::InputChar(c) => app.input_char(c),
            Action::Backspace => app.input_backspace(),
            Action::ToggleFavorite => app.toggle_favorite(),
            Action::ComposeUp => app.compose_up(),
            Action::ComposeDown => app.compose_down(),
            Action::PruneImages => app.prune_images(),
            Action::None => {}
        }
    }

    Ok(())
}
