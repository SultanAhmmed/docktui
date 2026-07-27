use crossterm::event::{self, Event, KeyCode, KeyEventKind};

use crate::app::{InputMode, View};

pub enum Action {
    Next,
    Previous,
    Refresh,
    OpenLogs,
    CloseLogs,
    Start,
    Stop,
    Restart,
    Remove,
    Confirm,
    Cancel,
    SwitchView(View),
    ToggleFilter,
    InputChar(char),
    Backspace,
    ToggleFavorite,
    ComposeUp,
    ComposeDown,
    PruneImages,
    Quit,
    None,
}

pub fn poll_action(tick_ms: u64, current_view: &View, input_mode: &InputMode) -> Action {
    if event::poll(std::time::Duration::from_millis(tick_ms)).unwrap_or(false) {
        if let Ok(Event::Key(key)) = event::read() {
            if key.kind != KeyEventKind::Press {
                return Action::None;
            }

            // Filter input mode takes priority
            if *input_mode == InputMode::Filtering {
                return match key.code {
                    KeyCode::Esc => Action::ToggleFilter,
                    KeyCode::Enter => Action::ToggleFilter,
                    KeyCode::Backspace => Action::Backspace,
                    KeyCode::Char(c) => Action::InputChar(c),
                    _ => Action::None,
                };
            }

            // Main view match — handles ALL 5 views
            return match current_view {
                View::Logs => match key.code {
                    KeyCode::Esc | KeyCode::Char('q') => Action::CloseLogs,
                    _ => Action::None,
                },
                View::Containers => match key.code {
                    KeyCode::Char('q') => Action::Quit,
                    KeyCode::Down | KeyCode::Char('j') => Action::Next,
                    KeyCode::Up | KeyCode::Char('k') => Action::Previous,
                    KeyCode::Char('r') => Action::Refresh,
                    KeyCode::Enter => Action::OpenLogs,
                    KeyCode::Char('s') => Action::Start,
                    KeyCode::Char('x') => Action::Stop,
                    KeyCode::Char('e') => Action::Restart,
                    KeyCode::Char('d') => Action::Remove,
                    KeyCode::Char('y') => Action::Confirm,
                    KeyCode::Char('n') => Action::Cancel,
                    KeyCode::Char('/') => Action::ToggleFilter,
                    KeyCode::Char('f') => Action::ToggleFavorite,
                    KeyCode::Char('1') => Action::SwitchView(View::Containers),
                    KeyCode::Char('2') => Action::SwitchView(View::Images),
                    KeyCode::Char('3') => Action::SwitchView(View::Compose),
                    KeyCode::Char('4') => Action::SwitchView(View::Stats),
                    _ => Action::None,
                },
                View::Images => match key.code {
                    KeyCode::Char('q') => Action::Quit,
                    KeyCode::Down | KeyCode::Char('j') => Action::Next,
                    KeyCode::Up | KeyCode::Char('k') => Action::Previous,
                    KeyCode::Char('r') => Action::Refresh,
                    KeyCode::Char('p') => Action::PruneImages,
                    KeyCode::Char('y') => Action::Confirm,
                    KeyCode::Char('n') => Action::Cancel,
                    KeyCode::Char('1') => Action::SwitchView(View::Containers),
                    KeyCode::Char('2') => Action::SwitchView(View::Images),
                    KeyCode::Char('3') => Action::SwitchView(View::Compose),
                    KeyCode::Char('4') => Action::SwitchView(View::Stats),
                    _ => Action::None,
                },
                View::Compose => match key.code {
                    KeyCode::Char('q') => Action::Quit,
                    KeyCode::Down | KeyCode::Char('j') => Action::Next,
                    KeyCode::Up | KeyCode::Char('k') => Action::Previous,
                    KeyCode::Char('r') => Action::Refresh,
                    KeyCode::Char('u') => Action::ComposeUp,
                    KeyCode::Char('w') => Action::ComposeDown,
                    KeyCode::Char('y') => Action::Confirm,
                    KeyCode::Char('n') => Action::Cancel,
                    KeyCode::Char('1') => Action::SwitchView(View::Containers),
                    KeyCode::Char('2') => Action::SwitchView(View::Images),
                    KeyCode::Char('3') => Action::SwitchView(View::Compose),
                    KeyCode::Char('4') => Action::SwitchView(View::Stats),
                    _ => Action::None,
                },
                View::Stats => match key.code {
                    KeyCode::Char('q') => Action::Quit,
                    KeyCode::Char('r') => Action::Refresh,
                    KeyCode::Char('1') => Action::SwitchView(View::Containers),
                    KeyCode::Char('2') => Action::SwitchView(View::Images),
                    KeyCode::Char('3') => Action::SwitchView(View::Compose),
                    KeyCode::Char('4') => Action::SwitchView(View::Stats),
                    _ => Action::None,
                },
            };
        }
    }
    Action::None
}
