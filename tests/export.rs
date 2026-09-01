#[path = "../src/export.rs"]
mod export;

#[test]
fn writes_report_and_refuses_existing_file_without_force() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("reports/dependencies.txt");
    export::write(&path, "first\n", false).unwrap();
    assert_eq!(std::fs::read_to_string(&path).unwrap(), "first\n");
    let error = export::write(&path, "second\n", false).unwrap_err();
    assert_eq!(error.kind(), std::io::ErrorKind::AlreadyExists);
    export::write(&path, "second\n", true).unwrap();
    assert_eq!(std::fs::read_to_string(&path).unwrap(), "second\n");
}

#[test]
fn chooses_timestamped_tui_path_when_default_exists() {
    let directory = tempfile::tempdir().unwrap();
    std::fs::write(directory.path().join("gradlens-report.txt"), "old").unwrap();
    let path = export::available_tui_path(directory.path(), 1_700_000_000);
    assert_eq!(path.file_name().unwrap(), "gradlens-report-1700000000.txt");
}
