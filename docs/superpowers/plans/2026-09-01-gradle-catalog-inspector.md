# Gradle Catalog Inspector Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a Rust CLI whose default command opens a polished Ratatui dependency browser and whose `inspect` subcommand prints the Gradle-resolved transitive dependency tree and selected-version release links.

**Architecture:** A single binary crate separates catalog parsing, Gradle integration, normalized dependency data, release-link lookup, plain rendering, and Ratatui state/rendering. Both interfaces consume one `Inspection` model. Gradle Wrapper output is authoritative; Tokio runs Gradle and HTTP work without blocking Ratatui.

**Tech Stack:** Rust stable; Cargo; clap; serde/serde_json; toml; tempfile; tokio; reqwest with rustls; url; ratatui 0.30; crossterm; futures-util; opener; thiserror.

**Spec:** `docs/superpowers/specs/2026-09-01-gradle-catalog-inspector-design.md`

## Global Constraints

- `gradle-checker` opens the Ratatui interface; `gradle-checker inspect` is noninteractive.
- Use the repository Gradle Wrapper; never resolve Maven dependencies independently.
- Match catalog entries to resolved components by `group:name`; use Gradle's selected version everywhere downstream.
- Exact release links must be verified; otherwise label a metadata-derived fallback `generic` or report `none`.
- Do not modify the inspected Gradle project.
- Respect `NO_COLOR`; restore terminal state after normal exit, errors, panics, and interrupts.
- Keep one binary crate and focused internal modules; add no plugin, daemon, database, mouse-only action, or web-search provider.

---

### Task 1: CLI shell and shared domain model

**Files:**
- Create: `Cargo.toml`
- Create: `src/main.rs`
- Create: `src/cli.rs`
- Create: `src/model.rs`
- Create: `tests/cli.rs`

**Interfaces:**
- Produces: `cli::Cli`, `cli::Command::Inspect(InspectArgs)`, `model::{ModuleId, ComponentId, DependencyNode, LibraryInspection, Inspection, ReleaseLink, ReleaseMatch}`.
- `Inspection` is the sole completed-result input to both renderers.

- [ ] **Step 1: Write failing CLI tests**

Add integration tests using `std::process::Command` and Cargo's binary path. Assert `gradle-checker --help` documents the interactive default and `gradle-checker inspect --help` accepts `--catalog` and `--configuration`. Add model unit tests proving `ModuleId` display is `group:name` and `ComponentId` display is `group:name:version`.

- [ ] **Step 2: Verify the tests fail**

Run `cargo test --test cli` and `cargo test model`. Expected: failure because the crate and types do not exist.

- [ ] **Step 3: Add the minimal crate and types**

Use clap derive. Model definitions:

```rust
pub struct ModuleId { pub group: String, pub name: String }
pub struct ComponentId { pub module: ModuleId, pub version: String }
pub struct DependencyNode { pub component: ComponentId, pub children: Vec<DependencyNode>, pub cycle: bool }
pub enum ReleaseMatch { Exact, Generic, None }
pub struct ReleaseLink { pub version: String, pub url: Option<url::Url>, pub match_kind: ReleaseMatch, pub diagnostic: Option<String> }
pub struct LibraryInspection { pub alias: String, pub requested: Option<ComponentId>, pub selected: ComponentId, pub dependencies: Vec<DependencyNode>, pub release: ReleaseLink }
pub struct Inspection { pub configuration: String, pub libraries: Vec<LibraryInspection> }
```

The default command only dispatches to a temporary `run_tui()` error until the TUI task replaces it; `inspect` parses arguments and returns a clear not-yet-wired error. Do not add behavior beyond argument parsing and model formatting.

- [ ] **Step 4: Verify green**

Run `cargo test --test cli` and `cargo test model`. Expected: pass.

### Task 2: Version catalog parser

**Files:**
- Create: `src/catalog.rs`
- Create: `tests/fixtures/catalog/complete.toml`
- Create: `tests/fixtures/catalog/malformed.toml`

**Interfaces:**
- Consumes: `model::{ModuleId, ComponentId}`.
- Produces: `catalog::parse(path: &Path) -> Result<Catalog, CatalogError>` and `Catalog { libraries: Vec<CatalogLibrary> }`, where `CatalogLibrary { alias, module, requested_version }`.

- [ ] **Step 1: Write failing parser tests**

Cover string coordinates, `{ module, version }`, `{ module, version.ref }`, `{ group, name, version }`, rich versions with `strictly`/`require`/`prefer`, missing `version.ref`, malformed coordinates, and deterministic alias ordering. Rich versions choose `strictly`, then `require`, then `prefer` as the requested display value; absent versions remain `None` because Gradle decides them.

- [ ] **Step 2: Verify red**

Run `cargo test catalog`. Expected: unresolved module/parser failures.

- [ ] **Step 3: Implement the minimal parser**

Deserialize only `[versions]` and `[libraries]` through private serde types. Normalize each supported declaration into `CatalogLibrary`. Reject aliases without a valid module and references to absent version keys with alias-specific errors.

- [ ] **Step 4: Verify green**

Run `cargo test catalog`. Expected: pass.

### Task 3: Gradle inspection protocol

**Files:**
- Create: `src/gradle.rs`
- Create: `src/gradle/init.gradle.kts`
- Create: `tests/gradle_protocol.rs`
- Create: `tests/fixtures/gradle/settings.gradle.kts`
- Create: `tests/fixtures/gradle/build.gradle.kts`
- Create: `tests/fixtures/gradle/gradle/libs.versions.toml`
- Add fixture wrapper files generated by `gradle wrapper` using a pinned Gradle version recorded in `gradle-wrapper.properties`.

**Interfaces:**
- Produces: `GradleInspector::configurations() -> Result<Vec<String>, GradleError>` and `GradleInspector::resolve(selector: &str) -> Result<ResolvedGraph, GradleError>`.
- Protocol models: `ResolvedGraph { components: BTreeMap<String, ResolvedComponent>, roots: Vec<String> }`; each component carries `ComponentId`, child IDs, selection reason, and metadata URLs/paths.

- [ ] **Step 1: Write failing protocol decoder tests**

Use captured delimited output containing ordinary Gradle logs around JSON. Assert extraction rejects absent markers, duplicate payloads, malformed JSON, and missing child IDs. Add selector tests for root `:configuration`, qualified `:app:configuration`, unique unqualified names, ambiguity, and absence.

- [ ] **Step 2: Verify red**

Run `cargo test gradle_protocol`. Expected: missing adapter failures.

- [ ] **Step 3: Implement payload decoding and wrapper invocation**

Embed the init script with `include_str!`. Copy it to a `NamedTempFile`, invoke `gradlew`/`gradlew.bat` with `--init-script`, `--console=plain`, and an internal task name, capture stdout/stderr, and preserve relevant stderr on failure. The script enumerates resolvable configurations and serializes selected module components and dependency edges between unique markers. It resolves only the selected configuration.

- [ ] **Step 4: Verify unit green**

Run `cargo test gradle_protocol`. Expected: pass.

- [ ] **Step 5: Exercise the real fixture**

Run the fixture's wrapper inspection through the Rust integration test. Assert the selected graph contains a direct module and at least one transitive child. Expected: pass without changing fixture build files.

### Task 4: Catalog-to-resolved dependency mapping

**Files:**
- Create: `src/graph.rs`
- Create: `tests/graph.rs`

**Interfaces:**
- Consumes: `Catalog`, `ResolvedGraph`.
- Produces: `graph::map_used_libraries(&Catalog, &ResolvedGraph) -> Vec<ResolvedLibrary>`, with selected root and cycle-safe `DependencyNode` children.

- [ ] **Step 1: Write failing graph tests**

Prove unused catalog aliases are omitted; requested and selected versions may differ; shared descendants appear under each parent path; cycles are marked and stop recursion; duplicate edges are removed; aliases and siblings sort deterministically.

- [ ] **Step 2: Verify red**

Run `cargo test --test graph`. Expected: missing mapper.

- [ ] **Step 3: Implement minimal graph traversal**

Index resolved components by `ModuleId`. Match catalog roots by `group:name`. Traverse with a per-path `HashSet`, not a global visited set, so shared descendants remain visible. Sort by `ComponentId` before traversal.

- [ ] **Step 4: Verify green**

Run `cargo test --test graph`. Expected: pass.

### Task 5: Selected-version release-link resolution

**Files:**
- Create: `src/releases.rs`
- Create: `tests/releases.rs`

**Interfaces:**
- Consumes: selected `ComponentId` plus metadata-established project/SCM URLs.
- Produces: `ReleaseResolver::resolve(&ReleaseCandidate) -> ReleaseLink`.

- [ ] **Step 1: Write failing resolver tests**

Start a local TCP HTTP fixture. Test verified raw-version and `v`-version GitHub/GitLab targets, redirect following, unsuccessful targets falling back to a generic releases URL, missing metadata yielding `None`, network failure remaining nonfatal, and AndroidX coordinates producing the family page plus selected-version anchor. Assert no guessed URL is marked exact.

- [ ] **Step 2: Verify red**

Run `cargo test --test releases`. Expected: missing resolver.

- [ ] **Step 3: Implement conservative URL derivation**

Normalize metadata-established Git repository URLs, strip `.git`, and try only host-specific release/tag candidates. Verify through a reqwest client with bounded connect and total timeouts. Implement the explicit AndroidX group/family rule. Preserve a generic metadata URL when exact verification fails.

- [ ] **Step 4: Verify green**

Run `cargo test --test releases`. Expected: pass with no public-network access.

### Task 6: Inspection service and plain output

**Files:**
- Create: `src/inspect.rs`
- Create: `src/plain.rs`
- Modify: `src/main.rs`
- Modify: `tests/cli.rs`

**Interfaces:**
- Produces: `Inspector::inspect(selector: &str) -> Result<Inspection, InspectError>` and `plain::render(&Inspection, ColorChoice) -> String`.
- `inspect` orchestrates catalog parsing, Gradle resolution, graph mapping, and nonfatal release resolution.

- [ ] **Step 1: Write failing orchestration and snapshot-style assertions**

Using in-memory graph/release fixtures, assert output includes alias, requested coordinate, Gradle-selected coordinate, exact/generic/none labels, dependency branches, and cycle markers in stable order. Assert `NO_COLOR` and redirected output contain no ANSI escapes. Assert release lookup failures do not fail inspection.

- [ ] **Step 2: Verify red**

Run `cargo test plain inspect`. Expected: missing service/renderer.

- [ ] **Step 3: Implement service and renderer**

Keep orchestration independent of Ratatui. Render Unicode tree guides only when stdout supports them; retain a plain ASCII-safe path for redirected output. Wire `gradle-checker inspect` to print on success and emit concise stderr plus nonzero status on local/Gradle errors.

- [ ] **Step 4: Verify green**

Run `cargo test plain inspect --test cli`. Expected: pass.

- [ ] **Step 5: Smoke-test the real subcommand**

Build and run `target/debug/gradle-checker inspect --catalog tests/fixtures/gradle/gradle/libs.versions.toml --configuration :runtimeClasspath` from the fixture root or with the supported project-root argument. Assert output contains the fixture's selected transitive component.

### Task 7: Ratatui application state and asynchronous jobs

**Files:**
- Create: `src/tui/mod.rs`
- Create: `src/tui/app.rs`
- Create: `src/tui/event.rs`
- Create: `tests/tui_state.rs`

**Interfaces:**
- Produces: `tui::run(Inspector) -> Result<(), TuiError>`; `App::update(Action)`; `Action` includes input, resize, configurations loaded, inspection loaded, release update, failure, and cancellation.

- [ ] **Step 1: Write failing state-machine tests**

Test focus cycling, list navigation, tree expansion, search entry/filter/escape, refresh request IDs, stale result rejection, cancellation, error dismissal, help overlay, and quit. Tests operate on `App` and `Action` without a real terminal.

- [ ] **Step 2: Verify red**

Run `cargo test --test tui_state`. Expected: missing state machine.

- [ ] **Step 3: Implement state and event translation**

Keep rendering pure and side effects in an event/job loop. Use monotonically increasing request IDs. Background tasks send bounded messages; stale IDs are ignored. Key events act only on `KeyEventKind::Press`. `Esc` follows search/help/cancel/back precedence before quitting.

- [ ] **Step 4: Verify green**

Run `cargo test --test tui_state`. Expected: pass.

### Task 8: Ratatui layout, theme, and terminal lifecycle

**Files:**
- Create: `src/tui/ui.rs`
- Create: `src/tui/theme.rs`
- Create: `tests/tui_render.rs`
- Modify: `src/tui/mod.rs`
- Modify: `src/main.rs`

**Interfaces:**
- `ui::render(frame: &mut Frame, app: &mut App, theme: &Theme)`.
- `Theme::detect()` returns semantic styles for dark/light, 16-color, and monochrome operation.

- [ ] **Step 1: Write failing TestBackend render tests**

At 120x40 assert the configuration/library sidebar, dependency tree, release panel, and footer are visible. At 80x24 and 99x30 assert compact tabbed layout. Below 80x24 assert only the resize message. Verify focused-panel indication, exact/generic/none text, spinner status, help overlay, and monochrome labels.

- [ ] **Step 2: Verify red**

Run `cargo test --test tui_render`. Expected: missing renderer.

- [ ] **Step 3: Implement responsive Ratatui widgets**

Use constraint-based `Layout`, stateful `List`, custom flattened tree rows, `Paragraph`, `Tabs`, and `Clear` for overlays. Use semantic theme slots only. Keep selection instantaneous and redraw on input/job/tick events rather than animation-heavy fixed FPS.

- [ ] **Step 4: Implement terminal safety**

Initialize Ratatui/Crossterm only for the default command. Install restoration for panic, error, and interrupt paths; enter raw mode and alternate screen after validation; always restore before returning. Open URLs only for a present release URL and surface opener errors in the detail panel.

- [ ] **Step 5: Verify green**

Run `cargo test --test tui_render --test tui_state`. Expected: pass.

### Task 9: End-to-end verification and cleanup

**Files:**
- Modify only files implicated by verification failures.

**Interfaces:**
- No new interfaces.

- [ ] **Step 1: Run focused tests**

Run `cargo test`. Expected: all unit and integration tests pass without public-network access.

- [ ] **Step 2: Run static checks**

Run `cargo fmt --check` and `cargo clippy --all-targets --all-features -- -D warnings`. Expected: clean.

- [ ] **Step 3: Smoke-test plain mode**

Run the compiled `inspect` subcommand against the fixture. Verify selected-version release status and a transitive dependency path are printed and no target project files changed.

- [ ] **Step 4: Smoke-test the actual TUI**

Launch the compiled binary in a pseudo-terminal at 120x40. Select a configuration and library, expand a dependency, open and close help, resize to 80x24, then quit. Verify the shell remains usable and terminal echo/raw state is restored.

- [ ] **Step 5: Perform required cleanup**

Remove temporary scaffolding, unused dependencies, dead code, and generated artifacts not required by the fixture. Re-run `cargo test`, `cargo fmt --check`, and Clippy after cleanup.
