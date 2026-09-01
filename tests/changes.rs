#[path = "../src/catalog.rs"]
mod catalog;
#[path = "../src/changes.rs"]
mod changes;
#[path = "../src/model.rs"]
mod model;

use catalog::{Catalog, CatalogLibrary};
use changes::compare;
use model::{ChangeKind, ModuleId};

fn library(alias: &str, module: &str, version: Option<&str>) -> CatalogLibrary {
    let (group, name) = module.split_once(':').unwrap();
    CatalogLibrary {
        alias: alias.into(),
        module: ModuleId {
            group: group.into(),
            name: name.into(),
        },
        requested_version: version.map(Into::into),
    }
}

#[test]
fn classifies_added_removed_updated_and_module_changes() {
    let baseline = Catalog {
        libraries: vec![
            library("removed", "g:old", Some("1")),
            library("updated", "g:u", Some("1")),
            library("moved", "g:old-module", Some("1")),
            library("same", "g:s", Some("1")),
        ],
    };
    let current = Catalog {
        libraries: vec![
            library("added", "g:new", Some("1")),
            library("updated", "g:u", Some("2")),
            library("moved", "g:new-module", Some("1")),
            library("same", "g:s", Some("1")),
        ],
    };
    let result = compare(&baseline, &current);
    assert_eq!(
        result
            .iter()
            .map(|c| (&*c.alias, c.kind))
            .collect::<Vec<_>>(),
        vec![
            ("added", ChangeKind::Added),
            ("moved", ChangeKind::ModuleChanged),
            ("removed", ChangeKind::Removed),
            ("updated", ChangeKind::Updated)
        ]
    );
}
