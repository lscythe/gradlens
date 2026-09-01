use std::{
    fs, io,
    path::{Path, PathBuf},
};

pub fn write(path: &Path, contents: &str, force: bool) -> io::Result<()> {
    if path.exists() && !force {
        return Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            format!(
                "{} already exists; use --force to overwrite",
                path.display()
            ),
        ));
    }
    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        fs::create_dir_all(parent)?;
    }
    let temporary = path.with_extension(format!("tmp.{}", std::process::id()));
    fs::write(&temporary, contents)?;
    if let Err(error) = fs::rename(&temporary, path) {
        let _ = fs::remove_file(&temporary);
        return Err(error);
    }
    Ok(())
}

pub fn available_tui_path(directory: &Path, timestamp: u64) -> PathBuf {
    let default = directory.join("gradlens-report.txt");
    if default.exists() {
        directory.join(format!("gradlens-report-{timestamp}.txt"))
    } else {
        default
    }
}
