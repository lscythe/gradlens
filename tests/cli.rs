use std::process::Command;

fn command(arguments: &[&str]) -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_gradlens"));
    command.args(arguments);
    command
}

#[test]
fn help_documents_the_interactive_default() {
    let output = command(&["--help"]).output().unwrap();

    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("interactive"));
}

#[test]
fn inspect_help_accepts_catalog_and_configuration() {
    let output = command(&["inspect", "--help"]).output().unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("--catalog"));
    assert!(stdout.contains("--configuration"));
    assert!(stdout.contains("--output"));
    assert!(stdout.contains("--force"));
    assert!(stdout.contains("--summary"));
    assert!(stdout.contains("--release-notes-only"));
}
