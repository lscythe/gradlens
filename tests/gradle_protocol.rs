#[path = "../src/gradle.rs"]
mod gradle;
#[path = "../src/model.rs"]
mod model;

use gradle::{ConfigurationCandidate, decode_graph, resolve_selector};

#[test]
fn decodes_one_delimited_graph_amid_logs() {
    let output = "noise\nGRADLE_CHECKER_BEGIN\n{\"components\":{\"a\":{\"id\":{\"module\":{\"group\":\"g\",\"name\":\"a\"},\"version\":\"1\"},\"children\":[\"b\"],\"metadata_urls\":[]}},\"roots\":[\"a\"]}\nGRADLE_CHECKER_END\nmore";
    let error = decode_graph(output).unwrap_err().to_string();
    assert!(error.contains("unknown child 'b'"));
}

#[test]
fn rejects_missing_and_duplicate_payloads() {
    assert!(decode_graph("nothing").is_err());
    let payload = "GRADLE_CHECKER_BEGIN\n{\"components\":{},\"roots\":[]}\nGRADLE_CHECKER_END";
    assert!(decode_graph(&format!("{payload}\n{payload}")).is_err());
}

#[test]
fn resolves_qualified_unique_and_ambiguous_selectors() {
    let values = vec![
        ConfigurationCandidate {
            project: ":app".into(),
            name: "runtimeClasspath".into(),
        },
        ConfigurationCandidate {
            project: ":lib".into(),
            name: "runtimeClasspath".into(),
        },
        ConfigurationCandidate {
            project: ":app".into(),
            name: "compileClasspath".into(),
        },
    ];
    assert_eq!(
        resolve_selector(":app:runtimeClasspath", &values)
            .unwrap()
            .project,
        ":app"
    );
    assert_eq!(
        resolve_selector("compileClasspath", &values).unwrap().name,
        "compileClasspath"
    );
    assert!(
        resolve_selector("runtimeClasspath", &values)
            .unwrap_err()
            .to_string()
            .contains("ambiguous")
    );
    assert!(resolve_selector("missing", &values).is_err());
}
