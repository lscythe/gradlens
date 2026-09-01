use ratatui::{
    Frame,
    layout::{Constraint, Layout},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Tabs, Wrap},
};

use super::{
    app::{App, Focus, Mode},
    theme::Theme,
};
use crate::model::{DependencyNode, ReleaseMatch};

pub fn render(frame: &mut Frame, app: &mut App, theme: &Theme) {
    let area = frame.area();
    if area.width < 80 || area.height < 24 {
        frame.render_widget(
            Paragraph::new("Terminal too small — resize to at least 80x24").centered(),
            area,
        );
        return;
    }
    let [body, footer] = Layout::vertical([Constraint::Fill(1), Constraint::Length(1)]).areas(area);
    if area.width >= 100 {
        render_wide(frame, body, app, theme);
    } else {
        render_compact(frame, body, app, theme);
    }
    let status = if app.loading {
        "⠋ resolving…  [Esc] cancel"
    } else {
        "[/] search  [Enter] expand  [o] open  [r] refresh  [?] help  [q] quit"
    };
    frame.render_widget(Paragraph::new(status).style(theme.muted), footer);
    if app.mode == Mode::Help {
        render_help(frame, theme);
    }
}

fn render_wide(frame: &mut Frame, area: ratatui::layout::Rect, app: &mut App, theme: &Theme) {
    let [left, right] =
        Layout::horizontal([Constraint::Percentage(30), Constraint::Fill(1)]).areas(area);
    let [config, libraries] =
        Layout::vertical([Constraint::Percentage(35), Constraint::Fill(1)]).areas(left);
    let [tree, release] =
        Layout::vertical([Constraint::Percentage(70), Constraint::Fill(1)]).areas(right);
    render_configurations(frame, config, app, theme);
    render_libraries(frame, libraries, app, theme);
    render_tree(frame, tree, app, theme);
    render_release(frame, release, app, theme);
}

fn render_compact(frame: &mut Frame, area: ratatui::layout::Rect, app: &mut App, theme: &Theme) {
    let [tabs, panel] = Layout::vertical([Constraint::Length(3), Constraint::Fill(1)]).areas(area);
    frame.render_widget(
        Tabs::new([
            "Configurations",
            "Libraries",
            "Dependency tree",
            "Release notes",
        ])
        .select(app.focus as usize)
        .block(Block::bordered())
        .highlight_style(theme.focus),
        tabs,
    );
    match app.focus {
        Focus::Configurations => render_configurations(frame, panel, app, theme),
        Focus::Libraries => render_libraries(frame, panel, app, theme),
        Focus::Dependencies => render_tree(frame, panel, app, theme),
        Focus::Release => render_release(frame, panel, app, theme),
    }
}

fn block(title: &'static str, focused: bool, theme: &Theme) -> Block<'static> {
    Block::default()
        .borders(Borders::ALL)
        .title(title)
        .border_style(if focused { theme.focus } else { theme.muted })
}
fn render_configurations(frame: &mut Frame, area: ratatui::layout::Rect, app: &App, theme: &Theme) {
    let items: Vec<_> = app
        .configurations
        .iter()
        .enumerate()
        .filter(|(_, v)| filter(v, &app.search))
        .map(|(i, v)| {
            ListItem::new(if i == app.configuration_index {
                format!("> {v}")
            } else {
                format!("  {v}")
            })
        })
        .collect();
    frame.render_widget(
        List::new(items)
            .block(block(
                "Configurations",
                app.focus == Focus::Configurations,
                theme,
            ))
            .style(theme.text),
        area,
    );
}
fn render_libraries(frame: &mut Frame, area: ratatui::layout::Rect, app: &App, theme: &Theme) {
    let items: Vec<_> = app
        .inspection
        .as_ref()
        .map(|value| {
            value
                .libraries
                .iter()
                .enumerate()
                .filter(|(_, v)| filter(&v.alias, &app.search))
                .map(|(i, v)| {
                    ListItem::new(if i == app.library_index {
                        format!("> {}", v.alias)
                    } else {
                        format!("  {}", v.alias)
                    })
                })
                .collect()
        })
        .unwrap_or_default();
    frame.render_widget(
        List::new(items)
            .block(block("Libraries", app.focus == Focus::Libraries, theme))
            .style(theme.text),
        area,
    );
}
fn render_tree(frame: &mut Frame, area: ratatui::layout::Rect, app: &App, theme: &Theme) {
    let mut lines = Vec::new();
    if let Some(lib) = selected(app) {
        lines.push(Line::from(lib.selected.to_string()).style(theme.focus));
        for (i, node) in lib.dependencies.iter().enumerate() {
            tree_lines(node, "", i + 1 == lib.dependencies.len(), &mut lines);
        }
    }
    frame.render_widget(
        Paragraph::new(lines)
            .block(block(
                "Dependency tree",
                app.focus == Focus::Dependencies,
                theme,
            ))
            .wrap(Wrap { trim: false }),
        area,
    );
}
fn tree_lines<'a>(node: &'a DependencyNode, prefix: &str, last: bool, out: &mut Vec<Line<'a>>) {
    out.push(Line::from(format!(
        "{prefix}{} {}{}",
        if last { "└─" } else { "├─" },
        node.component,
        if node.cycle { " (cycle)" } else { "" }
    )));
    let next = format!("{prefix}{} ", if last { " " } else { "│" });
    for (i, child) in node.children.iter().enumerate() {
        tree_lines(child, &next, i + 1 == node.children.len(), out);
    }
}
fn render_release(frame: &mut Frame, area: ratatui::layout::Rect, app: &App, theme: &Theme) {
    let text = selected(app)
        .map(|lib| {
            let label = match lib.release.match_kind {
                ReleaseMatch::Exact => "exact",
                ReleaseMatch::Generic => "generic",
                ReleaseMatch::None => "none",
            };
            vec![
                Line::from(format!("Version {}  [{label}]", lib.release.version)),
                Line::from(
                    lib.release
                        .url
                        .as_ref()
                        .map(ToString::to_string)
                        .unwrap_or_else(|| "not found".into()),
                )
                .style(Style::default().add_modifier(Modifier::UNDERLINED)),
                Line::from(lib.release.diagnostic.clone().unwrap_or_default()).style(theme.warning),
            ]
        })
        .unwrap_or_default();
    frame.render_widget(
        Paragraph::new(text)
            .block(block("Release notes", app.focus == Focus::Release, theme))
            .style(theme.text),
        area,
    );
}
fn selected(app: &App) -> Option<&crate::model::LibraryInspection> {
    app.inspection.as_ref()?.libraries.get(app.library_index)
}
fn filter(value: &str, query: &str) -> bool {
    query.is_empty() || value.to_lowercase().contains(&query.to_lowercase())
}
fn render_help(frame: &mut Frame, theme: &Theme) {
    let area = frame.area();
    let popup = ratatui::layout::Rect {
        x: area.x + area.width / 6,
        y: area.y + area.height / 5,
        width: area.width * 2 / 3,
        height: area.height * 3 / 5,
    };
    frame.render_widget(Clear, popup);
    frame.render_widget(
        Paragraph::new(vec![
            Line::from(Span::styled("Keyboard help", theme.focus)),
            Line::from("Tab/Shift+Tab focus   ↑↓ or j/k navigate"),
            Line::from("/ search   r refresh   o open URL"),
            Line::from("Esc close/cancel   q quit"),
        ])
        .block(Block::bordered().title("Help")),
        popup,
    );
}
