#[path = "../src/catalog.rs"]
mod catalog;
#[path = "../src/changes.rs"]
mod changes;
#[path = "../src/model.rs"]
mod model;
#[path = "../src/plain.rs"]
mod plain;

use model::*;

fn id(name: &str, version: &str) -> ComponentId {
    ComponentId {
        module: ModuleId {
            group: "g".into(),
            name: name.into(),
        },
        version: version.into(),
    }
}

#[test]
fn renders_selected_version_release_and_tree_without_ansi() {
    let inspection = Inspection {
        configuration: ":app:runtimeClasspath".into(),
        libraries: vec![LibraryInspection {
            alias: "alpha".into(),
            requested: Some(id("a", "1")),
            selected: id("a", "2"),
            dependencies: vec![DependencyNode {
                component: id("b", "1"),
                children: vec![],
                cycle: false,
                repeated: false,
            }],
            release: ReleaseLink {
                version: "2".into(),
                url: None,
                match_kind: ReleaseMatch::None,
                diagnostic: None,
            },
            change: None,
        }],
        removed: vec![],
    };
    let output = plain::render(&inspection);
    assert!(output.contains("requested: g:a:1"));
    assert!(output.contains("selected:  g:a:2"));
    assert!(output.contains("match: none"));
    assert!(output.contains("g:b:1"));
    assert!(!output.contains("\u{1b}["));
}

#[test]
fn renders_changes_not_present_in_configuration() {
    let inspection = Inspection {
        configuration: ":app:runtimeClasspath".into(),
        libraries: vec![],
        removed: vec![LibraryChange {
            alias: "kotlin-gradle-plugin".into(),
            kind: ChangeKind::Updated,
            baseline: Some(id("kotlin-gradle-plugin", "2.0")),
            current: Some(id("kotlin-gradle-plugin", "2.2")),
        }],
    };
    let output = plain::render(&inspection);
    assert!(output.contains("changes not present in configuration"));
    assert!(output.contains("kotlin-gradle-plugin"));
}

fn changed_library(alias: &str, release_url: Option<&str>) -> LibraryInspection {
    LibraryInspection {
        alias: alias.into(),
        requested: Some(id(alias, "2")),
        selected: id(alias, "3"),
        dependencies: vec![],
        release: ReleaseLink {
            version: "3".into(),
            url: release_url.map(|url| url::Url::parse(url).unwrap()),
            match_kind: if release_url.is_some() {
                ReleaseMatch::Exact
            } else {
                ReleaseMatch::None
            },
            diagnostic: None,
        },
        change: Some(LibraryChange {
            alias: alias.into(),
            kind: ChangeKind::Updated,
            baseline: Some(id(alias, "1")),
            current: Some(id(alias, "2")),
        }),
    }
}

#[test]
fn summary_renders_counts_and_changed_dependency_rows_without_trees() {
    let mut library = changed_library("alpha", Some("https://example.com/alpha/3"));
    library.dependencies.push(DependencyNode {
        component: id("transitive", "1"),
        children: vec![],
        cycle: false,
        repeated: false,
    });
    let inspection = Inspection {
        configuration: ":app:runtimeClasspath".into(),
        libraries: vec![library],
        removed: vec![LibraryChange {
            alias: "absent".into(),
            kind: ChangeKind::Removed,
            baseline: Some(id("absent", "1")),
            current: None,
        }],
    };

    let output = plain::render_summary(&inspection);

    assert!(output.contains("updated: 1"));
    assert!(output.contains("removed: 1"));
    assert!(output.contains("alpha"));
    assert!(output.contains("g:alpha:1"));
    assert!(output.contains("g:alpha:2"));
    assert!(output.contains("g:alpha:3"));
    assert!(output.contains("https://example.com/alpha/3"));
    assert!(!output.contains("g:transitive:1"));
}

#[test]
fn release_notes_filter_keeps_only_libraries_with_urls() {
    let inspection = Inspection {
        configuration: ":app:runtimeClasspath".into(),
        libraries: vec![
            changed_library("documented", Some("https://example.com/documented/3")),
            changed_library("undocumented", None),
        ],
        removed: vec![],
    };

    let filtered = plain::with_release_notes(&inspection);

    assert_eq!(filtered.libraries.len(), 1);
    assert_eq!(filtered.libraries[0].alias, "documented");
}
