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
            }],
            release: ReleaseLink {
                version: "2".into(),
                url: None,
                match_kind: ReleaseMatch::None,
                diagnostic: None,
            },
        }],
    };
    let output = plain::render(&inspection);
    assert!(output.contains("requested: g:a:1"));
    assert!(output.contains("selected:  g:a:2"));
    assert!(output.contains("match: none"));
    assert!(output.contains("g:b:1"));
    assert!(!output.contains("\u{1b}["));
}
