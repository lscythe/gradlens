pub mod app;
pub mod theme;
pub mod ui;

use std::{io, time::Duration};

use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use ratatui::DefaultTerminal;

use crate::inspect::Inspector;
use app::{Action, App};
use theme::Theme;

pub fn run(inspector: Inspector) -> io::Result<()> {
    let mut terminal = ratatui::init();
    let result = run_loop(&mut terminal, inspector);
    ratatui::restore();
    result
}

fn run_loop(terminal: &mut DefaultTerminal, inspector: Inspector) -> io::Result<()> {
    let mut app = App::default();
    let theme = Theme::detect();
    app.update(Action::Refresh);
    match inspector.configurations() {
        Ok(values) => app.update(Action::ConfigurationsLoaded {
            request_id: app.request_id,
            values,
        }),
        Err(error) => app.update(Action::Failed {
            request_id: app.request_id,
            message: error.to_string(),
        }),
    }
    while !app.should_quit {
        terminal.draw(|frame| ui::render(frame, &mut app, &theme))?;
        if event::poll(Duration::from_millis(100))?
            && let Event::Key(key) = event::read()?
        {
            if key.kind != KeyEventKind::Press {
                continue;
            }
            let action = match key.code {
                KeyCode::Char('q') => Action::Quit,
                KeyCode::Tab => Action::NextFocus,
                KeyCode::BackTab => Action::PreviousFocus,
                KeyCode::Down | KeyCode::Char('j') => Action::MoveDown,
                KeyCode::Up | KeyCode::Char('k') => Action::MoveUp,
                KeyCode::Char('/') => Action::StartSearch,
                KeyCode::Char('?') => Action::ToggleHelp,
                KeyCode::Char('r') => Action::Refresh,
                KeyCode::Esc => Action::Escape,
                KeyCode::Backspace => Action::Backspace,
                KeyCode::Char(c)
                    if app.mode == app::Mode::Search
                        && !key.modifiers.contains(KeyModifiers::CONTROL) =>
                {
                    Action::SearchChar(c)
                }
                _ => continue,
            };
            app.update(action);
        }
    }
    Ok(())
}
