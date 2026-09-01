#[path = "../src/tui/app.rs"]
mod app;
#[path = "../src/model.rs"]
mod model;
#[path = "../src/tui/theme.rs"]
mod theme;
#[path = "../src/tui/ui.rs"]
mod ui;

use app::App;
use ratatui::{Terminal, backend::TestBackend};
use theme::Theme;

fn rendered(width: u16, height: u16) -> String {
    let backend = TestBackend::new(width, height);
    let mut terminal = Terminal::new(backend).unwrap();
    let mut app = App::default();
    app.configurations = vec![":app:runtimeClasspath".into()];
    terminal
        .draw(|frame| ui::render(frame, &mut app, &Theme::monochrome()))
        .unwrap();
    terminal
        .backend()
        .buffer()
        .content
        .iter()
        .map(|cell| cell.symbol())
        .collect()
}

#[test]
fn wide_layout_shows_all_panels_and_help() {
    let output = rendered(120, 40);
    for label in [
        "Configurations",
        "Libraries",
        "Dependency tree",
        "Release notes",
        "search",
        "quit",
    ] {
        assert!(output.contains(label), "missing {label}");
    }
}

#[test]
fn compact_and_too_small_layouts_are_explicit() {
    assert!(rendered(80, 24).contains("Configurations"));
    assert!(rendered(79, 23).contains("Terminal too small"));
}
