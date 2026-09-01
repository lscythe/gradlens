#[path = "../src/catalog.rs"]
mod catalog;
#[path = "../src/model.rs"]
mod model;

use std::{fs, path::PathBuf};

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/catalog")
        .join(name)
}

#[test]
fn parses_supported_library_forms_in_alias_order() {
    let catalog = catalog::parse(&fixture("complete.toml")).unwrap();
    let values: Vec<_> = catalog
        .libraries
        .iter()
        .map(|it| {
            (
                it.alias.as_str(),
                it.module.to_string(),
                it.requested_version.as_deref(),
            )
        })
        .collect();
    assert_eq!(
        values,
        vec![
            ("group-name", "com.acme:grouped".into(), Some("3.0")),
            ("inline", "com.acme:inline".into(), Some("2.0")),
            ("noversion", "com.acme:platform".into(), None),
            ("referenced", "com.acme:referenced".into(), Some("1.2.3")),
            ("rich", "com.acme:rich".into(), Some("4.0")),
            ("string", "com.acme:string".into(), Some("1.0")),
        ]
    );
}

#[test]
fn reports_missing_version_reference_with_alias() {
    let error = catalog::parse(&fixture("malformed.toml"))
        .unwrap_err()
        .to_string();
    assert!(error.contains("broken"));
    assert!(error.contains("missing"));
}

#[test]
fn rejects_malformed_string_coordinate() {
    let path = std::env::temp_dir().join(format!("gradle-checker-{}.toml", std::process::id()));
    fs::write(&path, "[libraries]\nbad = \"only:two\"\n").unwrap();
    let error = catalog::parse(&path).unwrap_err().to_string();
    fs::remove_file(path).unwrap();
    assert!(error.contains("bad"));
}
