#[path = "../src/gradle.rs"]
mod gradle;
#[path = "../src/model.rs"]
mod model;

#[test]
fn kotlin_init_script_has_kotlin_dsl_suffix() {
    let script = gradle::write_init_script().unwrap();
    assert_eq!(
        script
            .path()
            .file_name()
            .unwrap()
            .to_string_lossy()
            .ends_with(".gradle.kts"),
        true
    );
}
