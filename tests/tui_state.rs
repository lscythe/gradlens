#[path = "../src/tui/app.rs"]
mod app;
#[path = "../src/model.rs"]
mod model;

use app::{Action, App, Focus, Mode};

#[test]
fn focus_search_help_and_quit_transitions_are_predictable() {
    let mut app = App::default();
    assert_eq!(app.focus, Focus::Configurations);
    app.update(Action::NextFocus);
    assert_eq!(app.focus, Focus::Libraries);
    app.update(Action::StartSearch);
    assert_eq!(app.mode, Mode::Search);
    app.update(Action::SearchChar('r'));
    assert_eq!(app.search, "r");
    app.update(Action::Escape);
    assert_eq!(app.mode, Mode::Normal);
    app.update(Action::ToggleHelp);
    assert_eq!(app.mode, Mode::Help);
    app.update(Action::Escape);
    assert_eq!(app.mode, Mode::Normal);
    app.update(Action::Quit);
    assert!(app.should_quit);
}

#[test]
fn refresh_ids_reject_stale_results() {
    let mut app = App::default();
    app.update(Action::Refresh);
    let old = app.request_id;
    app.update(Action::Refresh);
    let current = app.request_id;
    app.update(Action::ConfigurationsLoaded {
        request_id: old,
        values: vec!["stale".into()],
    });
    assert!(app.configurations.is_empty());
    app.update(Action::ConfigurationsLoaded {
        request_id: current,
        values: vec![":app:runtimeClasspath".into()],
    });
    assert_eq!(app.configurations.len(), 1);
}
