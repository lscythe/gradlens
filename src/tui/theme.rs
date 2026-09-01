use ratatui::style::{Color, Modifier, Style};

#[derive(Clone)]
pub struct Theme {
    pub text: Style,
    pub muted: Style,
    pub focus: Style,
    pub warning: Style,
}

impl Theme {
    pub fn detect() -> Self {
        if std::env::var_os("NO_COLOR").is_some() {
            return Self::monochrome();
        }
        Self {
            text: Style::default().fg(Color::White),
            muted: Style::default().fg(Color::Gray),
            focus: Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
            warning: Style::default().fg(Color::Yellow),
        }
    }
    pub fn monochrome() -> Self {
        Self {
            text: Style::default(),
            muted: Style::default().add_modifier(Modifier::DIM),
            focus: Style::default().add_modifier(Modifier::BOLD),
            warning: Style::default(),
        }
    }
}
