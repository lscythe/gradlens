# gradlens

A Rust CLI for reviewing Android Gradle version-catalog dependencies. It resolves dependencies through the project's Gradle Wrapper, shows transitive dependency trees, finds release notes for Gradle-selected versions, and compares catalog changes against a baseline Git branch.

## Requirements

- Rust and Cargo for installation from source
- Git when using baseline comparison
- A Gradle project with a working `gradlew` or `gradlew.bat`
- A version catalog, normally `gradle/libs.versions.toml`

## Installation

### macOS and Linux

```sh
curl --proto '=https' --tlsv1.2 -LsSf \
  https://github.com/lscythe/gradlens/releases/latest/download/gradlens-installer.sh | sh
```

The default destination is `${CARGO_HOME:-$HOME/.cargo}/bin`. To select another location:

```sh
curl --proto '=https' --tlsv1.2 -LsSf \
  https://github.com/lscythe/gradlens/releases/latest/download/gradlens-installer.sh |
  INSTALL_DIR="$HOME/.local/bin" sh
```

### Windows PowerShell

```powershell
irm https://github.com/lscythe/gradlens/releases/latest/download/gradlens-installer.ps1 | iex
```

The remote installers download the matching release archive and verify its SHA-256 checksum. To build and install from a source checkout instead, use `./install.sh` or `.\install.ps1`.

## Usage

### Interactive interface

```sh
gradlens
```

Controls:

- `Tab` / `Shift+Tab`: change panel
- Arrow keys or `j` / `k`: navigate
- `/`: search
- `r`: refresh
- `?`: help
- `Esc`: close or cancel
- `q`: quit

### Plain output

```sh
gradlens inspect \
  --configuration :app:releaseRuntimeClasspath
```

Use another catalog path when needed:

```sh
gradlens inspect \
  --catalog gradle/custom.versions.toml \
  --configuration :app:releaseRuntimeClasspath
```


Write the report directly to a file:

```sh
gradlens inspect \
  --configuration :app:releaseRuntimeClasspath \
  --output dependency-report.txt
```

Gradlens refuses to replace an existing report unless `--force` is supplied. Shell redirection remains supported. In the interactive interface, press `e` to export the complete active inspection to `gradlens-report.txt`; if that file exists, Gradlens creates a timestamped filename.

### Compare a version-update branch

Compare the current branch's catalog with the tip of an explicit baseline branch:

```sh
gradlens inspect \
  --baseline develop \
  --configuration :app:releaseRuntimeClasspath
```

The interactive equivalent is:

```sh
gradlens --baseline develop
```

Comparison mode reports added, removed, updated, and module-coordinate changes. Unchanged libraries are hidden. For current libraries it also shows Gradle's selected version, transitive dependencies, and release notes. The baseline catalog is read with `git show`; the baseline branch is never checked out or modified.

## How resolution works

`gradlens` injects a temporary Gradle init script and invokes the target project's Wrapper. Gradle therefore remains authoritative for variants, BOMs, constraints, substitutions, exclusions, repositories, and conflict resolution. The target project's build files are not modified.

Release-note lookup prefers an exact page for the selected version. When that cannot be verified, the output labels a metadata-derived generic releases page as `generic`, or reports `none` rather than presenting a guessed URL as exact.

## License

Licensed under the Apache License, Version 2.0. See [LICENSE](LICENSE).

## Development

```sh
cargo test
cargo clippy --bin gradlens -- -D warnings
cargo fmt --check
```
