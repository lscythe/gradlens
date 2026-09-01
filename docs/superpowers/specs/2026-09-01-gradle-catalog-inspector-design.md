# Gradle Version Catalog Inspector Design

## Goal

Build a Rust CLI that reads an Android project's Gradle version catalog, identifies catalog libraries used by a selected resolved Gradle configuration, displays each library's complete resolved transitive dependency tree, and reports release-note links for the version Gradle actually selected.

## Commands

```text
gradle-checker
gradle-checker inspect \
  [--catalog gradle/libs.versions.toml] \
  [--configuration :app:releaseRuntimeClasspath]
```

Both commands run from a Gradle project root. `gradle-checker` opens the interactive Ratatui interface. `gradle-checker inspect` resolves one configuration and emits stable, noninteractive terminal output suitable for scripts and CI. The default catalog path is `gradle/libs.versions.toml`.

## Authority and Scope

Gradle is authoritative for dependency resolution. The CLI must not reimplement resolution from Maven POM files. This preserves Gradle behavior for variants, platforms and BOMs, constraints, substitutions, exclusions, dependency locking, repositories, and conflict resolution.

A "used library" is a library declared in the selected version catalog whose module occurs in the selected configuration's resolved component graph. Unused catalog declarations are not shown.

The selected version is the version in Gradle's resolved graph. It may differ from the catalog's requested version. Release-note discovery and displayed transitive dependencies must use the selected version.

## Architecture

The implementation has six focused modules:

1. **Catalog parser** reads TOML and resolves library declarations expressed with `module`, `group` plus `name`, string coordinates, inline versions, and `version.ref`. It produces aliases with requested module coordinates and optional requested versions.
2. **Gradle adapter** locates the project wrapper, creates a temporary init script, invokes the wrapper, and decodes a machine-readable resolved component graph. The init script adds inspection behavior without changing project files.
3. **Dependency graph** maps catalog modules to resolved components, distinguishes roots from transitive nodes, preserves dependency edges, handles cycles safely, and produces deterministic ordering.
4. **Release-link resolver** starts from Gradle/POM project, SCM, and homepage metadata. It searches for a URL tied to the selected version, verifies exact targets, and otherwise returns a clearly labelled generic project changelog or releases URL.
5. **Plain renderer** prints concise, deterministic output for `gradle-checker inspect` without terminal control sequences when redirected.
6. **Ratatui application** owns interactive state, keyboard events, responsive layout, background resolution jobs, and terminal lifecycle. It consumes the same inspection result used by the plain renderer; dependency and release logic is not duplicated in UI code.

These are internal Rust modules, not separate crates or speculative public abstractions. Ratatui 0.30 with its Crossterm backend provides immediate-mode rendering and terminal setup/restoration. Tokio drives Gradle and HTTP work outside the rendering path, while Crossterm's event stream and a bounded application message channel keep input responsive.

## Data Flow

1. Validate the project root, catalog path, wrapper, and requested configuration.
2. Parse catalog library aliases and requested coordinates.
3. Write a temporary Gradle init script outside the project source tree.
4. Run the project's wrapper with that script and the requested configuration.
5. Have Gradle emit a delimited JSON payload containing resolved component identities, selected versions, dependency edges, selection reasons where available, and metadata locations or URLs available through Gradle.
6. Parse the payload and match catalog modules by `group:name` to resolved components.
7. For every matched catalog library, traverse its outgoing graph to collect the full transitive tree while preventing infinite recursion on cycles.
8. Resolve release links against the selected version.
9. Render results in stable alias and dependency order.
10. Remove temporary resources whether Gradle succeeds or fails.

## Interactive Terminal Interface

The default command opens a persistent dependency browser:

```text
┌ Libraries ─────────┬ Dependency tree ─────────────────────────┐
│ > room-runtime     │ androidx.room:room-runtime:2.8.0          │
│   okhttp           │ ├─ androidx.annotation:annotation:1.9.1  │
│   kotlin-stdlib    │ └─ kotlin-stdlib:2.2.10                  │
├ Configurations ────┤                                          │
│ releaseRuntime…    ├ Release notes ───────────────────────────┤
│ debugRuntime…      │ 2.8.0  exact                             │
└────────────────────┴ https://…#2.8.0                          │
 [/] search [Enter] expand [o] open [r] refresh [?] help [q] quit
```

The left column contains resolvable Gradle configurations and the catalog libraries used by the selected configuration. The main panel shows an expandable transitive tree for the selected library. The lower detail panel shows requested and selected versions, release-link confidence, the URL, and nonfatal lookup diagnostics.

`Tab` and `Shift+Tab` move focus. Arrow keys and `j`/`k` move selection. `Enter` and `h`/`l` collapse or expand tree nodes. `/` starts live filtering across visible libraries or dependencies; `Esc` exits search or cancels the active background operation. `o` opens the selected release URL through the operating system. `r` reruns resolution. `?` opens contextual help, and `q` exits. Every action is keyboard-accessible; mouse support is optional and excluded from the initial release.

Gradle resolution and release-link verification run asynchronously. The UI remains navigable, displays a delayed spinner and current operation, accepts cancellation, and applies results only if they belong to the current configuration request. Errors appear in a dismissible detail panel without leaving the terminal in raw mode.

At 100 columns or wider, the interface uses the persistent layout shown above. From 80 through 99 columns it switches to a single active panel with a tab bar. Below 80x24 it shows a resize message. Resize events never terminate the program.

The visual system is restrained and information-led: semantic foreground, muted, focus, exact, generic, warning, and error roles; a focused border; bold headings; dim metadata; and underlined URLs. A dark theme is the default with a light variant selected from terminal capabilities when available. Sixteen-color and monochrome mappings remain fully usable, `NO_COLOR` disables color, and status always has a textual or symbolic cue rather than color alone. Ratatui's buffered rendering prevents full-screen clearing and flicker.

Terminal initialization enters raw mode and the alternate screen only for the default TUI command. Normal exit, errors, panics, and interrupt signals restore the terminal. The `inspect` subcommand never enters raw mode or the alternate screen.

## Output

Exact version-specific release notes:

```text
androidx-room-runtime
  requested: androidx.room:room-runtime:2.7.2
  selected:  androidx.room:room-runtime:2.8.0
  release notes:
    version: 2.8.0
    url: https://developer.android.com/jetpack/androidx/releases/room#2.8.0
    match: exact
  dependencies:
    ├── androidx.annotation:annotation:1.9.1
    └── org.jetbrains.kotlin:kotlin-stdlib:2.2.10
```

Generic fallback:

```text
okhttp
  requested: com.squareup.okhttp3:okhttp:5.0.0
  selected:  com.squareup.okhttp3:okhttp:5.1.0
  release notes:
    version: 5.1.0
    url: https://github.com/square/okhttp/releases
    match: generic
```

When neither an exact nor generic metadata-derived URL is available, the CLI prints `url: not found` and `match: none`.

Repeated transitive components may appear under separate parent paths because the output represents dependency relationships, not only a flat set. A cycle is marked and not traversed again on the current path.

## Release-Link Semantics

The resolver must never label a constructed URL as exact without verification.

Resolution order:

1. Obtain project, SCM, issue, and homepage URLs from Gradle-accessible module metadata and cached POM metadata for the selected component.
2. Recognize conservative, host-specific release and tag URL patterns for supported hosts such as GitHub and GitLab.
3. Try common exact selected-version tag forms, including the raw version and `v`-prefixed version, only under a metadata-established repository.
4. Verify an exact target with an HTTP request that follows redirects and treats only a successful final response as exact.
5. If no exact target is verified, return a metadata-established generic changelog or releases page with `match: generic`.
6. Otherwise return no link.

AndroidX documentation requires family-level release pages and version anchors rather than repository release tags. The resolver may include a small explicit rule for AndroidX coordinates because this URL structure is stable and cannot be derived reliably from SCM metadata alone. No broad curated registry is included.

Network failures must not fail dependency inspection. They result in a generic fallback when known, or `match: none`; the diagnostic should distinguish inability to verify from malformed local input. HTTP requests use bounded connect and overall timeouts.

## Gradle Integration

The project wrapper is required and must be invoked directly (`gradlew` on Unix, `gradlew.bat` on Windows). The CLI must not depend on a globally installed Gradle version.

The temporary init script must:

- work without modifying build files;
- defer inspection until projects and configurations exist;
- select a configuration by its exact name;
- fail clearly when no resolvable configuration with that name exists;
- serialize only the required resolution data between unique start/end markers so normal Gradle logging cannot corrupt the payload;
- avoid resolving unrelated configurations.

If multiple subprojects expose the same configuration name, the first release requires a qualified configuration selector in the form `:module:configuration`; the root project uses `:configuration`. An unqualified name is accepted only when exactly one project provides it; ambiguity is an error listing candidates.

## Errors and Exit Behavior

User or project errors produce concise stderr diagnostics and a nonzero exit code:

- malformed or missing catalog;
- missing or non-executable Gradle Wrapper;
- missing or ambiguous configuration;
- Gradle invocation or resolution failure;
- malformed inspection payload.

A release-note lookup failure is nonfatal because dependency resolution remains useful.

Temporary files are removed on success and failure. Gradle's own relevant error output is retained in diagnostics rather than replaced by a generic message.

## Testing and Verification

Unit tests cover:

- all supported catalog declaration forms and `version.ref` lookup;
- malformed catalog diagnostics;
- requested-to-selected version differences;
- catalog-module matching;
- graph traversal, shared descendants, cycles, and deterministic ordering;
- exact, generic, and absent release-link classification using a local HTTP test server;
- rendering of requested version, selected version, match type, and tree structure;
- Ratatui state transitions for focus, selection, expansion, search, refresh, stale job results, cancellation, and errors;
- layout selection at 80x24, 99-column, and 100-column boundaries;
- semantic theme degradation with `NO_COLOR`.

An integration fixture with a Gradle Wrapper and a small JVM/Android-compatible dependency graph verifies the actual command path: temporary init script injection, configuration selection, Gradle resolution, graph decoding, and plain output. Network-facing release lookup is redirected to deterministic local fixtures in tests.

Final smoke tests run `gradle-checker inspect` against the integration project and drive the compiled TUI in a pseudo-terminal to select a configuration, inspect a dependency, open help, resize, and quit while verifying terminal restoration.

## Explicit Non-goals

The first release does not:

- resolve dependencies independently of Gradle;
- inspect every configuration automatically;
- mutate the target project;
- provide dependency-update recommendations;
- scrape arbitrary web search results;
- maintain a general library-to-release-notes registry;
- add a daemon, cache database, plugin installation step, or mouse-only action.
