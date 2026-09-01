# gradle-checker

A Rust CLI for reviewing Android Gradle version-catalog dependencies. It resolves dependencies through the project's Gradle Wrapper, shows transitive dependency trees, finds release notes for Gradle-selected versions, and compares catalog changes against a baseline Git branch.

## Requirements

- Rust and Cargo for installation from source
- Git when using baseline comparison
- A Gradle project with a working `gradlew` or `gradlew.bat`
- A version catalog, normally `gradle/libs.versions.toml`

## Installation

### macOS and Linux

```sh
./install.sh
```

The default destination is `${CARGO_HOME:-$HOME/.cargo}/bin`. Override it with:

```sh
INSTALL_DIR="$HOME/.local/bin" ./install.sh
# or
PREFIX=/usr/local ./install.sh
```

### Windows PowerShell

```powershell
.\install.ps1
```

The default destination is `$env:CARGO_HOME\bin`, falling back to `$HOME\.cargo\bin`. Override it with:

```powershell
.\install.ps1 -InstallDir "$HOME\bin"
```

## Usage

Run commands from the Gradle project you want to inspect.

### Interactive interface

```sh
gradle-checker
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
gradle-checker inspect \
  --configuration :app:releaseRuntimeClasspath
```

Use another catalog path when needed:

```sh
gradle-checker inspect \
  --catalog gradle/custom.versions.toml \
  --configuration :app:releaseRuntimeClasspath
```

### Compare a version-update branch

Compare the current branch's catalog with the tip of an explicit baseline branch:

```sh
gradle-checker inspect \
  --baseline develop \
  --configuration :app:releaseRuntimeClasspath
```

The interactive equivalent is:

```sh
gradle-checker --baseline develop
```

Comparison mode reports added, removed, updated, and module-coordinate changes. Unchanged libraries are hidden. For current libraries it also shows Gradle's selected version, transitive dependencies, and release notes. The baseline catalog is read with `git show`; the baseline branch is never checked out or modified.

## How resolution works

`gradle-checker` injects a temporary Gradle init script and invokes the target project's Wrapper. Gradle therefore remains authoritative for variants, BOMs, constraints, substitutions, exclusions, repositories, and conflict resolution. The target project's build files are not modified.

Release-note lookup prefers an exact page for the selected version. When that cannot be verified, the output labels a metadata-derived generic releases page as `generic`, or reports `none` rather than presenting a guessed URL as exact.

## Development

```sh
cargo test
cargo clippy --bin gradle-checker -- -D warnings
cargo fmt --check
```
