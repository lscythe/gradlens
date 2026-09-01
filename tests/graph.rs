#[path = "../src/catalog.rs"]
mod catalog;
#[path = "../src/gradle.rs"]
mod gradle;
#[path = "../src/graph.rs"]
mod graph;
#[path = "../src/model.rs"]
mod model;

use catalog::{Catalog, CatalogLibrary};
use gradle::{ResolvedComponent, ResolvedGraph};
use model::{ComponentId, ModuleId};
use std::collections::BTreeMap;

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
fn maps_only_used_aliases_with_selected_versions_and_cycles() {
    let catalog = Catalog {
        libraries: vec![
            CatalogLibrary {
                alias: "used".into(),
                module: id("a", "1").module,
                requested_version: Some("1".into()),
            },
            CatalogLibrary {
                alias: "unused".into(),
                module: id("z", "1").module,
                requested_version: Some("1".into()),
            },
        ],
    };
    let components = BTreeMap::from([
        (
            "a".into(),
            ResolvedComponent {
                id: id("a", "2"),
                children: vec!["b".into(), "b".into()],
                metadata_urls: vec![],
            },
        ),
        (
            "b".into(),
            ResolvedComponent {
                id: id("b", "1"),
                children: vec!["a".into()],
                metadata_urls: vec![],
            },
        ),
    ]);
    let result = graph::map_used_libraries(
        &catalog,
        &ResolvedGraph {
            components,
            roots: vec!["a".into()],
        },
    );
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].alias, "used");
    assert_eq!(result[0].selected.version, "2");
    assert_eq!(result[0].dependencies.len(), 1);
    assert!(result[0].dependencies[0].children[0].cycle);
}
