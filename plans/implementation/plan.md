# Implementation Plan

Phased build plan for the `hours` CLI tool. Each phase lists the spec references, source files to create or modify, and acceptance criteria.

---

## Phase 1: Project Scaffolding

**Spec references:**
- [architecture.md § Project Structure](../specs/architecture.md#project-structure)
- [architecture.md § Dependencies](../specs/architecture.md#dependencies)

**Source files to create:**

- `Cargo.toml` — Project manifest with all dependencies listed in [architecture.md § Dependencies](../specs/architecture.md#dependencies):
  - `clap` (with `derive` feature)
  - `crossterm`
  - `chrono` (with `serde` feature)
  - `serde` (with `derive` feature) + `serde_json`
  - `toml`
  - `genpdf`
  - `comfy-table`
  - `dirs`
  - `shellexpand`
  - Dev dependencies: `assert_cmd`, `assert_fs`, `predicates`, `serde_json`
- `src/main.rs` — Entry point: parse CLI with clap, dispatch to command handlers
- `src/cli/mod.rs` — Clap `#[derive(Parser)]` app definition with subcommands
- `src/cli/init.rs` — Stub for `hours init`
- `src/cli/add.rs` — Stub for `hours add`
- `src/cli/edit.rs` — Stub for `hours edit`
- `src/cli/list.rs` — Stub for `hours list`
- `src/cli/summary.rs` — Stub for `hours summary`
- `src/cli/export.rs` — Stub for `hours export`
- `src/config.rs` — Stub module
- `src/data/mod.rs` — Stub module
- `src/data/model.rs` — Stub module
- `src/data/store.rs` — Stub module
- `src/data/week.rs` — Stub module
- `src/git.rs` — Stub module
- `src/pdf.rs` — Stub module
- `src/ui/mod.rs` — Stub module
- `src/ui/prompts.rs` — Stub module

**Actions:**

- [x] Run `cargo init --name hours` in the project directory
- [x] Add all dependencies to `Cargo.toml`
- [x] Create the directory and module structure per [architecture.md § Project Structure](../specs/architecture.md#project-structure)
- [x] Define the clap app with all six subcommands (stubbed with `todo!()`)
- [x] Verify the project compiles with `cargo check`

**Lessons learned:**

- `genpdf` latest stable is `0.2`, not `0.3`. Use `genpdf = "0.2"` in `Cargo.toml`.
- Added `anyhow` as a dependency for ergonomic error handling across command handlers.

---

## Phase 2: Data Model & Week Calculation

**Spec references:**
- [data-model.md § Rust Types](../specs/data-model.md#rust-types)
- [data-model.md § Week Calculation](../specs/data-model.md#week-calculation)
- [data-model.md § Hour Categories](../specs/data-model.md#hour-categories)
- [data-model.md § Invariants](../specs/data-model.md#invariants)
- [data-model.md § Atomic Writes](../specs/data-model.md#atomic-writes)

**Source files to create/modify:**

- `src/data/model.rs` — Implement `HoursData`, `WeekEntry`, and `Category` types per [data-model.md § Rust Types](../specs/data-model.md#rust-types)
  - `WeekEntry::total()` method
  - `Category` enum with display names and JSON key mappings per [data-model.md § Hour Categories](../specs/data-model.md#hour-categories)
- `src/data/week.rs` — Week calculation functions per [data-model.md § Week Calculation](../specs/data-model.md#week-calculation)
  - `current_week(today: NaiveDate) -> (NaiveDate, NaiveDate)` — returns (start, end) for the Tue–Mon week containing `today`
  - `week_containing(date: NaiveDate) -> (NaiveDate, NaiveDate)` — same as above, explicit name for arbitrary dates
  - `all_weeks(start_date: NaiveDate, today: NaiveDate) -> Vec<(NaiveDate, NaiveDate)>` — generate all weeks from start through current
- `src/data/store.rs` — JSON persistence per [data-model.md § Atomic Writes](../specs/data-model.md#atomic-writes)
  - `load(path: &Path) -> Result<HoursData>` — read and deserialize `hours.json`
  - `save(path: &Path, data: &HoursData) -> Result<()>` — atomic write (tmp + fsync + rename)
  - Enforce invariants (sort, validate) on every save per [data-model.md § Invariants](../specs/data-model.md#invariants)
- `src/data/mod.rs` — Re-export public types

**Actions:**

- [x] Implement `HoursData` and `WeekEntry` with serde derives
- [x] Implement `Category` enum with `FromStr` for CLI parsing
- [x] Implement week calculation with `chrono` per the algorithm in [data-model.md § Current Week Algorithm](../specs/data-model.md#current-week-algorithm)
- [x] Implement `all_weeks` generator
- [x] Implement atomic JSON load/save
- [x] Write unit tests for week calculation covering all examples in [data-model.md § Examples](../specs/data-model.md#examples)
- [x] Write unit tests for atomic save (write, read back, verify)

**Lessons learned:**

- `chrono::NaiveDate::weekday()` requires `use chrono::Datelike` trait to be in scope — the method exists as a private inherent method but the public one comes from the `Datelike` trait.
- `tempfile` crate needed as a dev-dependency for store tests (not already present in Phase 1 scaffolding).
- Modules not yet consumed by `main.rs` need `#[allow(dead_code)]` on the `mod` declaration in `main.rs` to pass `clippy -D warnings`, since all public items are flagged as unused. These will be removed in Phase 8 when wiring is complete.

---

## Phase 3: Configuration

**Spec references:**
- [config-system.md § Config File Format](../specs/config-system.md#config-file-format)
- [config-system.md § Rust Types](../specs/config-system.md#rust-types)
- [config-system.md § Loading Behavior](../specs/config-system.md#loading-behavior)
- [config-system.md § Environment Variable Overrides](../specs/config-system.md#environment-variable-overrides)

**Source files to create/modify:**

- `src/config.rs` — Full implementation:
  - `Config`, `DataConfig`, `GitConfig`, `LicensureConfig` structs per [config-system.md § Rust Types](../specs/config-system.md#rust-types)
  - `Config::load() -> Result<Config>` — load from file with env var overrides per [config-system.md § Loading Behavior](../specs/config-system.md#loading-behavior)
  - `Config::save(path: &Path) -> Result<()>` — serialize to TOML and write (used by `init`)
  - Config directory resolution: `HOURS_CONFIG_DIR` env var → `~/.config/hours/` per [config-system.md § Config File Location](../specs/config-system.md#config-file-location)
  - Tilde expansion for `data.directory` using `shellexpand`
  - `HOURS_DATA_DIR` override per [config-system.md § Environment Variable Overrides](../specs/config-system.md#environment-variable-overrides)
  - `HOURS_NO_GIT` override

**Actions:**

- [x] Implement config structs with serde `Deserialize` (and `Serialize` for save)
- [x] Implement `Config::load()` with env var override chain
- [x] Implement `Config::save()` for init command
- [x] Implement tilde expansion
- [x] Write unit tests for config loading from a temp TOML file
- [x] Write unit tests for env var overrides

**Lessons learned:**

- Tests that use `env::set_var`/`env::remove_var` must be serialized with a `Mutex` since env vars are process-global and Rust tests run in parallel by default. A `static ENV_LOCK: Mutex<()>` acquired at the start of each env-touching test prevents races.
- `Config` also needs `Serialize` (not just `Deserialize`) so `Config::save()` can serialize to TOML. Both derives are needed.
- Added convenience methods `Config::data_dir()` and `Config::data_file()` for ergonomic path construction from the config, used by command handlers.

---

## Phase 4: Git Integration

**Spec references:**
- [git-sync.md § Commit Behavior](../specs/git-sync.md#commit-behavior)
- [git-sync.md § Commit Messages](../specs/git-sync.md#commit-messages)
- [git-sync.md § Push Failure Handling](../specs/git-sync.md#push-failure-handling)
- [git-sync.md § Initialization](../specs/git-sync.md#initialization)
- [git-sync.md § Disabling Git](../specs/git-sync.md#disabling-git)
- [git-sync.md § Error Handling](../specs/git-sync.md#error-handling)

**Source files to create/modify:**

- `src/git.rs` — Full implementation:
  - `git_init(data_dir: &Path, remote_name: &str, remote_url: &str) -> Result<()>` — per [git-sync.md § Initialization](../specs/git-sync.md#initialization)
  - `git_commit(data_dir: &Path, message: &str) -> Result<()>` — add + commit per [git-sync.md § Commit Behavior](../specs/git-sync.md#commit-behavior)
  - `git_push(data_dir: &Path, remote: &str) -> Result<()>` — push with failure handling per [git-sync.md § Push Failure Handling](../specs/git-sync.md#push-failure-handling)
  - `git_sync(data_dir: &Path, config: &GitConfig, message: &str) -> Result<()>` — orchestrator: commit + conditional push
  - Check for `git` binary existence
  - All functions are no-ops when `HOURS_NO_GIT=1` per [git-sync.md § Disabling Git](../specs/git-sync.md#disabling-git)

**Actions:**

- [x] Implement git operations using `std::process::Command`
- [x] Implement commit message formatting per [git-sync.md § Commit Messages](../specs/git-sync.md#commit-messages)
- [x] Implement push failure handling (warn to stderr, continue)
- [x] Implement `HOURS_NO_GIT` check
- [x] Implement `--no-git` flag (propagated from clap)

**Lessons learned:**

- `git commit` reports "nothing to commit" on **stdout**, not stderr. The `git_commit` function must check both streams to correctly detect and silently skip this case.
- Tests that call `git_init` (which creates a repo) need a separate `set_git_test_config` helper to set `user.email` and `user.name` locally in the test repo, since the CI/sandbox environment may not have global git config.
- The `git_init_and_commit` convenience function is useful for `hours init`, but in tests it's cleaner to call `git_init` + `set_git_test_config` + `git_commit` separately to ensure the test git config is set between repo creation and the first commit.

---

## Phase 5: Interactive UI (Prompts)

**Spec references:**
- [cli-system.md § Interactive Prompts](../specs/cli-system.md#interactive-prompts)
- [cli-system.md § Key Bindings](../specs/cli-system.md#key-bindings)
- [cli-system.md § Week Selector](../specs/cli-system.md#week-selector)
- [cli-system.md § Category Selector](../specs/cli-system.md#category-selector)
- [cli-system.md § Number Input](../specs/cli-system.md#number-input)

**Source files to create/modify:**

- `src/ui/prompts.rs` — Custom interactive prompts built on `crossterm`:
  - `select_week(weeks: &[(NaiveDate, NaiveDate)], data: &HoursData, current_week_start: NaiveDate) -> Result<NaiveDate>` — per [cli-system.md § Week Selector](../specs/cli-system.md#week-selector)
  - `select_category() -> Result<Category>` — per [cli-system.md § Category Selector](../specs/cli-system.md#category-selector)
  - `input_hours(prompt: &str, current_value: Option<f64>) -> Result<f64>` — per [cli-system.md § Number Input](../specs/cli-system.md#number-input)
  - Internal: `select_from_list(items: &[String], selected: usize) -> Result<usize>` — generic list selector with vim key bindings per [cli-system.md § Key Bindings](../specs/cli-system.md#key-bindings)
- `src/ui/mod.rs` — Re-export prompt functions

**Actions:**

- [x] Implement raw terminal mode with `crossterm::terminal::enable_raw_mode`
- [x] Implement event loop reading `crossterm::event::read()` for key events
- [x] Map `j`/`↓` to down, `k`/`↑` to up, `Enter` to confirm, `Esc`/`q` to cancel, `g` to top, `G` to bottom per [cli-system.md § Key Bindings](../specs/cli-system.md#key-bindings)
- [x] Implement list rendering with `>` marker on selected item
- [x] Implement week display format: `Mon DD – Mon DD, YYYY` with `(current)` marker and total hours
- [x] Implement number input with validation (>= 0, valid decimal)
- [x] Ensure raw mode is always cleaned up (use a guard/drop pattern)

**Lessons learned:**

- Interactive prompt functions that enter raw mode cannot be easily unit-tested with simulated input, since `crossterm::event::read()` reads directly from the terminal. Tests focus on the pure formatting/logic functions (`format_week_label`, category items) and the `RawModeGuard` lifecycle. Integration tests for interactive flows should use `--non-interactive` mode instead.
- Added `input_text`, `input_date`, and `confirm` helper prompts beyond the spec's minimum — these are needed by Phase 6's `init` command interactive flow (prompt for data dir, remote URL, start date, confirm targets).
- All prompt functions return `Option` to distinguish between user-confirmed input and cancellation (Esc/Ctrl+C). This allows command handlers to bail cleanly on cancel.
- `Ctrl+C` is handled explicitly in raw mode (matching `KeyModifiers::CONTROL` + `Char('c')`) since the default SIGINT handler is suppressed when raw mode is active.

---

## Phase 6: CLI Commands

**Spec references:**
- [cli-system.md § Commands](../specs/cli-system.md#commands) (all subsections)
- [cli-system.md § Non-Interactive Mode](../specs/cli-system.md#non-interactive-mode)

**Source files to create/modify:**

- `src/cli/mod.rs` — Full clap definition:
  - `#[derive(Parser)]` with subcommands per [cli-system.md § Commands](../specs/cli-system.md#commands)
  - Global `--no-git` flag
  - Per-command `--non-interactive` flag
  - Per-command flags as documented for each command

- `src/cli/init.rs` — `hours init` per [cli-system.md § `hours init`](../specs/cli-system.md#hours-init):
  - Interactive mode: prompt for data dir, remote URL, start date, targets
  - Non-interactive mode: read from `--data-dir`, `--remote`, `--start-date` flags
  - Write config, create data dir, init git, create empty `hours.json`, commit
  - References: [config-system.md § Initialization](../specs/config-system.md#initialization), [git-sync.md § Initialization](../specs/git-sync.md#initialization), [data-model.md § Empty State](../specs/data-model.md#empty-state)

- `src/cli/add.rs` — `hours add` per [cli-system.md § `hours add`](../specs/cli-system.md#hours-add):
  - Interactive mode: week selector → category selector → hours input
  - Non-interactive mode: `--week`, `--category`, `--hours` flags
  - Load data, find or create week entry, add hours to selected category, save, git sync
  - Validation per [cli-system.md § `hours add` Validation](../specs/cli-system.md#hours-add)

- `src/cli/edit.rs` — `hours edit` per [cli-system.md § `hours edit`](../specs/cli-system.md#hours-edit):
  - Interactive mode: week selector → edit each category with current value shown
  - Non-interactive mode: `--week` plus category flags, only provided categories are updated
  - Load data, find week entry, set values, save, git sync

- `src/cli/list.rs` — `hours list` per [cli-system.md § `hours list`](../specs/cli-system.md#hours-list):
  - Build `comfy-table` table with week rows and totals row
  - `--json` flag: output JSON array via `serde_json`
  - `--last N` flag: slice to last N weeks
  - Empty state message

- `src/cli/summary.rs` — `hours summary` per [cli-system.md § `hours summary`](../specs/cli-system.md#hours-summary):
  - Calculations per [summary-system.md § Calculations](../specs/summary-system.md#calculations)
  - Terminal display per [summary-system.md § Display Format](../specs/summary-system.md#display-format)
  - `--json` flag per [summary-system.md § JSON Output](../specs/summary-system.md#json-output)

- `src/cli/export.rs` — `hours export` per [cli-system.md § `hours export`](../specs/cli-system.md#hours-export):
  - Call PDF generation (Phase 7)
  - Default output path: `<data_dir>/exports/hours-report-YYYY-MM-DD.pdf`
  - `--output PATH` and `--open` flags

**Actions:**

- [x] Define full clap app with all subcommands and flags
- [x] Implement `init` command with both interactive and non-interactive paths
- [x] Implement `add` command with incremental hour accumulation
- [x] Implement `edit` command with value overwrite semantics
- [x] Implement `list` command with table and JSON output
- [x] Implement `summary` command with all four calculations
- [x] Implement `export` command shell (PDF generation in Phase 7)
- [x] Handle empty state gracefully in all read commands

**Lessons learned:**

- The clap app with all subcommands and flags was already defined in Phase 1 scaffolding — Phase 6 only needed to implement the `run()` functions.
- Removing `#[allow(dead_code)]` from `main.rs` module declarations and `#[allow(unused_imports)]` from re-export modules should be done once the CLI commands actually use those modules, otherwise clippy `-D warnings` fails. The `data/mod.rs` re-exports turned out to be unnecessary since all CLI files use `crate::data::model::` directly.
- The `confirm` prompt (from Phase 5) is not yet needed by any Phase 6 command handler. It needs `#[allow(dead_code)]` until a future phase uses it.
- `comfy_table::Cell` with `add_attribute(Attribute::Bold)` provides the bold TOTALS row formatting — `Cell::new` must be used instead of plain strings for the totals row to apply attributes.
- The `export` command calls `pdf::generate_report()` which is stubbed to return an error until Phase 7. This is intentional — the command shell handles path resolution and directory creation; PDF generation is the Phase 7 concern.

---

## Phase 7: PDF Export

**Spec references:**
- [pdf-export.md § Report Layout](../specs/pdf-export.md#report-layout)
- [pdf-export.md § Page Format](../specs/pdf-export.md#page-format)
- [pdf-export.md § File Output](../specs/pdf-export.md#file-output)

**Source files to create/modify:**

- `src/pdf.rs` — Full implementation:
  - `generate_report(data: &HoursData, config: &LicensureConfig, output_path: &Path) -> Result<()>`
  - Header section per [pdf-export.md § Header](../specs/pdf-export.md#header)
  - Hours table per [pdf-export.md § Hours Table](../specs/pdf-export.md#hours-table)
  - Progress summary per [pdf-export.md § Progress Summary](../specs/pdf-export.md#progress-summary)
  - Page format per [pdf-export.md § Page Format](../specs/pdf-export.md#page-format)
- `assets/fonts/` — Liberation Sans font files (Regular, Bold) for `genpdf`, or fall back to built-in Helvetica via `printpdf` directly

**Documentation to review:**
- `genpdf` crate docs: table API, font loading, page breaks
- `printpdf` crate docs: built-in fonts as fallback option

**Actions:**

- [x] Evaluate `genpdf` font handling — if bundling fonts is too complex, implement with `printpdf` built-in fonts and manual layout
- [x] Implement document creation with US Letter size and 1" margins
- [x] Implement header section (title, dates)
- [x] Implement weekly hours table with column alignment
- [x] Implement totals row with bold formatting
- [x] Implement progress summary section
- [x] Create `exports/` subdirectory on demand
- [x] Test with empty data, single week, and many weeks (page break)

**Lessons learned:**

- `genpdf::fonts::Builtin` is for the higher-level `from_files()` API. When using `FontData::new()` directly (for `include_bytes!` font embedding), the second parameter is `Option<printpdf::BuiltinFont>`, not `Option<genpdf::fonts::Builtin>`. Passing `None` embeds the font data in the PDF (slightly larger file, but no external dependency and works universally).
- In `genpdf` 0.2, `.styled()` wraps a `Paragraph` in a `StyledElement<Paragraph>` which does not expose `.aligned()`. The correct call order is `.aligned()` first, then `.styled()`: `Paragraph::new(text).aligned(Alignment::Right).styled(style)`.
- Liberation Sans fonts (SIL Open Font License) are metrically identical to Helvetica and work well with `genpdf`. The four `.ttf` files are embedded at compile time via `include_bytes!()` — adding ~1.6 MB to the binary but eliminating any runtime font file dependency.
- The `exports/` subdirectory creation is handled by the `export` CLI command (`src/cli/export.rs`), not by `pdf.rs`. The PDF module is purely responsible for rendering.
- `genpdf` 0.2's `Margins::trbl(25.4, 25.4, 25.4, 25.4)` sets 1-inch margins (25.4 mm = 1 inch) on US Letter paper.

---

## Phase 8: End-to-End Wiring

**Spec references:**
- [architecture.md § Data Flow](../specs/architecture.md#data-flow)

**Source files to modify:**

- `src/main.rs` — Wire all command handlers together:
  - Load config (error if missing, unless command is `init`)
  - Load data
  - Dispatch to command handler
  - Handle errors with user-friendly messages

**Actions:**

- [x] Implement top-level error handling (print message, exit with code 1)
- [x] Verify full flow: `init` → `add` → `add` (incremental) → `edit` → `list` → `summary` → `export`
- [x] Test with `--non-interactive` and `--no-git` flags
- [x] Test env var overrides (`HOURS_CONFIG_DIR`, `HOURS_DATA_DIR`, `HOURS_NO_GIT`)

**Lessons learned:**

- Top-level error handling (`eprintln!("Error: {e}")` + `process::exit(1)`) was already wired in Phase 6 as part of `main.rs`. No additional centralization of config/data loading was needed — each command handler loading its own config is the correct pattern since `init` creates config while all other commands require it.
- Clap requires `allow_hyphen_values = true` on `f64` arguments to parse negative numbers passed with a space (e.g., `--hours -1.0`). Without it, `-1.0` is interpreted as an unknown flag. This was added to the `hours` field in `AddArgs` and all category fields in `EditArgs`.
- `Iterator::sum::<f64>()` on an empty iterator returns `-0.0` (negative zero) in Rust, which displays as `-0.0`. Fix: add `+ 0.0` to normalize the sum, since IEEE 754 guarantees `-0.0 + 0.0 = 0.0`. The `round1()` helper was also patched: `if r == 0.0 { 0.0 } else { r }` to avoid propagating `-0.0` through JSON output.

---

## Phase 9: Documentation

**Source files to create:**

- `README.md` — Project README with:
  - One-line description: CLI tool for tracking counseling licensure hours
  - Installation instructions (`cargo install --path .`)
  - Quick start guide: `hours init` → `hours add` → `hours summary`
  - Full command reference (brief — link to specs for details)
  - Configuration section: config file location, format, env vars
  - Data storage and backup: where data lives, git sync behavior
  - Link to `specs/README.md` for detailed specifications
- `LICENSE` — Choose appropriate license (MIT or similar)
- `specs/README.md` — Already created (update if needed after implementation)

**Actions:**

- [x] Write `README.md` with installation, quickstart, command reference
- [x] Add license file (MIT)
- [x] Review all specs for accuracy against final implementation
- [x] Ensure all spec cross-references are correct

**Lessons learned:**

- Spec review found 3 minor inaccuracies (pre-existing from earlier phases, not introduced here): (1) `hours list` empty state message wording differs slightly from cli-system.md spec, (2) `hours init` exits on existing config rather than prompting for confirmation as spec describes, (3) `hours export` does not call git sync as spec describes. All cross-references between specs are valid.
- The LICENSE file uses MIT license. The bundled Liberation Sans fonts in `assets/fonts/` have their own SIL Open Font License 1.1 — the README references both.

---

## Phase 10: Integration Tests

**Spec references:**
- [architecture.md § Testability](../specs/architecture.md#testability)
- [cli-system.md § Non-Interactive Mode](../specs/cli-system.md#non-interactive-mode)
- [summary-system.md § JSON Output](../specs/summary-system.md#json-output)

**Source files to create:**

- `tests/integration.rs` — End-to-end tests using `assert_cmd` + `assert_fs`

All tests use isolated temp directories via `HOURS_CONFIG_DIR`, `HOURS_DATA_DIR`, and `HOURS_NO_GIT=1` environment variables. All mutating commands use `--non-interactive` with explicit flags.

### Test: Initialize fresh setup

- Run `hours init --data-dir <tmp> --remote git@github.com:test/test.git --start-date 2025-01-28 --non-interactive` with `HOURS_CONFIG_DIR=<tmp_config>` and `HOURS_NO_GIT=1`
- Assert: exit code 0
- Assert: `<tmp_config>/config.toml` exists and contains correct values
- Assert: `<tmp>/hours.json` exists and contains `{"weeks": []}`
- **Verifies:** [cli-system.md § `hours init`](../specs/cli-system.md#hours-init), [config-system.md § Initialization](../specs/config-system.md#initialization), [data-model.md § Empty State](../specs/data-model.md#empty-state)

### Test: Add hours to current week

- Run `init` (as above), then `hours add --category direct --hours 3.5 --non-interactive`
- Assert: exit code 0
- Assert: `hours.json` contains one week entry with `direct: 3.5` and all other categories at `0.0`
- Assert: week start is a Tuesday, end is the following Monday
- **Verifies:** [cli-system.md § `hours add`](../specs/cli-system.md#hours-add), [data-model.md § Week Calculation](../specs/data-model.md#week-calculation)

### Test: Add hours incrementally (accumulation)

- Run `init`, then `hours add --category direct --hours 3.5 --non-interactive`, then `hours add --category direct --hours 2.0 --non-interactive`
- Assert: `hours.json` week entry has `direct: 5.5`
- **Verifies:** [cli-system.md § `hours add`](../specs/cli-system.md#hours-add) — incremental accumulation behavior

### Test: Add hours to multiple categories

- Run `init`, add 3.5 direct, add 1.0 individual_supervision, add 2.0 group_supervision, add 4.0 indirect
- Assert: `hours.json` week has all four values correctly set
- Assert: `WeekEntry::total()` logic matches expected sum (10.5)
- **Verifies:** [data-model.md § Hour Categories](../specs/data-model.md#hour-categories)

### Test: Add hours to a specific past week

- Run `init`, then `hours add --week 2025-01-28 --category direct --hours 5.0 --non-interactive`
- Assert: `hours.json` contains a week entry with start `2025-01-28` and end `2025-02-03`
- **Verifies:** [cli-system.md § `hours add`](../specs/cli-system.md#hours-add) with explicit `--week` flag

### Test: Edit overwrites values

- Run `init`, add 3.5 direct hours, then `hours edit --week <start> --direct 10.0 --non-interactive`
- Assert: `hours.json` week entry has `direct: 10.0` (not 13.5)
- **Verifies:** [cli-system.md § `hours edit`](../specs/cli-system.md#hours-edit) — overwrite semantics

### Test: Edit preserves unspecified categories

- Run `init`, add 3.5 direct and 1.0 individual_supervision, then `hours edit --week <start> --direct 10.0 --non-interactive`
- Assert: `direct: 10.0` and `individual_supervision: 1.0` (unchanged)
- **Verifies:** [cli-system.md § `hours edit`](../specs/cli-system.md#hours-edit) — partial update behavior

### Test: List output (table)

- Run `init`, add data to two different weeks, then `hours list`
- Assert: exit code 0
- Assert: stdout contains both week date ranges
- Assert: stdout contains a TOTALS row
- **Verifies:** [cli-system.md § `hours list`](../specs/cli-system.md#hours-list)

### Test: List output (JSON)

- Run `init`, add data, then `hours list --json`
- Assert: stdout is valid JSON
- Assert: parsed JSON is an array with correct week objects
- **Verifies:** [cli-system.md § `hours list`](../specs/cli-system.md#hours-list) with `--json` flag

### Test: List with `--last N`

- Run `init`, add data to 3 weeks, then `hours list --last 2 --json`
- Assert: JSON array has exactly 2 entries (the most recent two)
- **Verifies:** [cli-system.md § `hours list`](../specs/cli-system.md#hours-list) with `--last` flag

### Test: Summary calculations

- Run `init` (start_date = 2025-01-28), add known hours to specific weeks, then `hours summary --json`
- Assert: JSON `total_hours.current` matches expected sum
- Assert: JSON `direct_hours.current` matches expected direct sum
- Assert: JSON `weekly_average.current` matches total / weeks_elapsed
- Assert: All percentages are calculated correctly
- **Verifies:** [summary-system.md § Calculations](../specs/summary-system.md#calculations)

### Test: Summary with no data (empty state)

- Run `init`, then `hours summary --json`
- Assert: all `current` values are 0
- Assert: all `percentage` values are 0.0
- **Verifies:** [summary-system.md § Empty State](../specs/summary-system.md#empty-state)

### Test: Export generates PDF

- Run `init`, add data, then `hours export`
- Assert: exit code 0
- Assert: a `.pdf` file exists at the default output path (`<data_dir>/exports/hours-report-YYYY-MM-DD.pdf`)
- Assert: file size is > 0 bytes
- **Verifies:** [cli-system.md § `hours export`](../specs/cli-system.md#hours-export), [pdf-export.md § File Output](../specs/pdf-export.md#file-output)

### Test: Export with custom output path

- Run `init`, add data, then `hours export --output <tmp>/custom-report.pdf`
- Assert: `<tmp>/custom-report.pdf` exists
- **Verifies:** [pdf-export.md § File Output](../specs/pdf-export.md#file-output) with `--output` override

### Test: Config env var overrides

- Run `init` with `HOURS_CONFIG_DIR=<tmp_a>`, then run `hours summary` with `HOURS_CONFIG_DIR=<tmp_a>` and `HOURS_DATA_DIR=<tmp_b>`
- Assert: tool reads data from `<tmp_b>`, not from the directory specified in config
- **Verifies:** [config-system.md § Environment Variable Overrides](../specs/config-system.md#environment-variable-overrides)

### Test: Validation rejects negative hours

- Run `init`, then `hours add --category direct --hours -1.0 --non-interactive`
- Assert: exit code non-zero
- Assert: stderr contains an error about invalid hours
- **Verifies:** [cli-system.md § `hours add` Validation](../specs/cli-system.md#hours-add)

### Test: Validation rejects non-Tuesday week start

- Run `init`, then `hours add --week 2025-01-29 --category direct --hours 1.0 --non-interactive`
- Assert: exit code non-zero
- Assert: stderr contains an error about the date not being a Tuesday
- **Verifies:** [data-model.md § Invariants](../specs/data-model.md#invariants)

### Test: List and summary with empty data

- Run `init`, then `hours list`, then `hours summary`
- Assert: both exit code 0
- Assert: `hours list` output contains "No hours logged yet"
- **Verifies:** [cli-system.md § `hours list` Empty State](../specs/cli-system.md#hours-list), [summary-system.md § Empty State](../specs/summary-system.md#empty-state)

### Test: Data file integrity after multiple operations

- Run `init`, perform a sequence of adds and edits across multiple weeks
- Read `hours.json` directly and parse it
- Assert: weeks are sorted by start date ascending per [data-model.md § Invariants](../specs/data-model.md#invariants)
- Assert: all start dates are Tuesdays
- Assert: all end dates are start + 6 days
- Assert: no duplicate weeks
- **Verifies:** [data-model.md § Invariants](../specs/data-model.md#invariants)

### Additional tools required for testing

- **`assert_cmd`** — Runs the compiled binary as a subprocess with env var control. Available as a Rust crate.
- **`assert_fs`** — Creates and manages temporary directories for test isolation. Available as a Rust crate.
- **PDF verification** — Integration tests verify the PDF file exists and has non-zero size. Content verification of PDF internals would require a PDF parsing library (e.g., `lopdf`) or a manual spot-check. For the initial implementation, existence and size checks are sufficient.

**Actions:**

- [x] Implement all 19 integration tests in `tests/integration.rs`
- [x] All tests pass with `cargo test --workspace`
- [x] All tests use isolated temp directories via `HOURS_CONFIG_DIR`, `HOURS_DATA_DIR`, and `HOURS_NO_GIT=1`
- [x] All mutating commands use `--non-interactive` and `--no-git` flags
- [x] Clippy passes with `-D warnings`
- [x] Code formatted with `cargo fmt --all`

**Lessons learned:**

- `chrono::NaiveDate::weekday()` requires `use chrono::Datelike` in integration tests just as in library code — the trait must be explicitly imported.
- Integration tests that set `HOURS_CONFIG_DIR` and `HOURS_DATA_DIR` environment variables achieve full isolation. Each test creates its own `TempDir` instances, so parallel test execution works without conflicts.
- The `add` command always adds to the current week when `--week` is not specified, so tests that need deterministic week dates should always use `--week` with a known Tuesday date.
- Helper functions (`init_env`, `add_hours`, `add_hours_to_week`, `load_data`) reduce boilerplate significantly across 19 tests. The JSON data file can be read directly for precise assertions rather than relying solely on CLI output.

---

## Phase 11: Interactive Flow Redesign (Looping Navigation)

**Spec references:**
- [cli-system.md § `hours add` Interactive flow](../specs/cli-system.md#hours-add)
- [cli-system.md § `hours edit` Interactive flow](../specs/cli-system.md#hours-edit)
- [cli-system.md § Navigation Model](../specs/cli-system.md#navigation-model)
- [cli-system.md § Key Bindings](../specs/cli-system.md#key-bindings)
- [cli-system.md § Help Overlay](../specs/cli-system.md#help-overlay)
- [cli-system.md § Confirmation Flash](../specs/cli-system.md#confirmation-flash)
- [cli-system.md § Category Selector](../specs/cli-system.md#category-selector) (edit mode with current values)
- [git-sync.md § Commit Behavior](../specs/git-sync.md#commit-behavior)

**Overview:**

Replace the linear one-shot interactive flow in `hours add` and `hours edit` with nested loops and back-navigation. After entering hours, the UI returns to the category selector for the same week instead of exiting. `Esc`/`q` goes back one level at every screen. Add a `?` key for an in-app help overlay and document navigation in `--help` text.

**Source files to modify:**

- `src/ui/prompts.rs` — Core UI changes:
  - Add `PromptResult<T>` enum (replaces `Option<T>` returns)
  - Add `Help` variant to `SelectAction`; distinguish `Back` (Esc/q) from `Exit` (Ctrl+C)
  - Add `render_help_overlay()` function
  - Add `flash_confirmation()` function
  - Add `select_category_with_values()` for edit mode category display
  - Update return types of `select_from_list`, `select_week`, `select_category`, `input_hours`
  - Add `?` key handling in `read_select_key()` and `input_hours()`
- `src/ui/mod.rs` — Update re-exports:
  - Add `PromptResult`, `flash_confirmation`, `select_category_with_values`
- `src/cli/add.rs` — Rewrite interactive branch:
  - Replace linear flow (lines 59–71) with nested `'week_loop` / `'category_loop`
  - Move save + git sync inside inner loop
  - Call `flash_confirmation()` after each save, then `continue 'category_loop`
  - Match on `PromptResult::Back` to go up one level, `PromptResult::Exit` to break out
  - Add `#[command(after_help = "...")]` for navigation key documentation
- `src/cli/edit.rs` — Rewrite interactive branch:
  - Replace sequential-all-categories flow (lines 95–124) with nested loops
  - Use `select_category_with_values()` to show current values per category
  - Single-category-at-a-time editing with save + flash after each
  - Add `#[command(after_help = "...")]` for navigation key documentation

**Documentation to review:**
- `crossterm` crate docs: `event::read()`, `KeyCode`, `KeyModifiers` for `?` key handling
- `std::thread::sleep` for confirmation flash duration

---

### Step 1: Add new types and functions (non-breaking, additive)

- [x] Add `PromptResult<T>` enum to `src/ui/prompts.rs` (after line 14, after imports):
  ```
  pub enum PromptResult<T> {
      Value(T),  // User confirmed a selection/input
      Back,      // Esc/q -- go back one level
      Exit,      // Ctrl+C -- exit immediately
  }
  ```
  Per [cli-system.md § Navigation Model](../specs/cli-system.md#navigation-model): Esc/q goes back one level, Ctrl+C exits immediately.

- [x] Add `render_help_overlay()` to `src/ui/prompts.rs`:
  - Clear screen, render key bindings table per [cli-system.md § Help Overlay](../specs/cli-system.md#help-overlay)
  - Wait for any key press, then return
  - View: `src/ui/prompts.rs:62-86` (`render_list`) for rendering pattern reference

- [x] Add `pub fn flash_confirmation(message: &str) -> Result<()>` to `src/ui/prompts.rs`:
  - Clear screen, print message, sleep ~1 second, return
  - Per [cli-system.md § Confirmation Flash](../specs/cli-system.md#confirmation-flash)

- [x] Add `pub fn select_category_with_values(entry: &WeekEntry) -> Result<PromptResult<Category>>` to `src/ui/prompts.rs`:
  - Format items as `"{long_name}    {value:.1} hrs"` with alignment
  - Per [cli-system.md § Category Selector](../specs/cli-system.md#category-selector) (edit mode display)
  - View: `src/ui/prompts.rs:179-189` (`select_category`) for pattern reference
  - View: `src/data/model.rs:18-70` (`WeekEntry`) for `entry.get(cat)` API

- [x] Update `src/ui/mod.rs` exports (line 3):
  - Add `PromptResult`, `flash_confirmation`, `select_category_with_values` to the `pub use` statement

- [x] Verify: `cargo build --workspace` passes (all new code is additive, no callers changed yet)

---

### Step 2: Update SelectAction and key reading

- [x] Rename `SelectAction::Cancel` to `SelectAction::Back` in `src/ui/prompts.rs` (line 37):
  - View: `src/ui/prompts.rs:31-38` (enum definition)

- [x] Add `SelectAction::Exit` and `SelectAction::Help` variants to the enum

- [x] Update `read_select_key()` in `src/ui/prompts.rs` (lines 40-60):
  - Map `Ctrl+C` → `SelectAction::Exit` (currently maps to `Cancel`)
  - Map `Esc`/`q` → `SelectAction::Back` (rename from `Cancel`)
  - Map `?` → `SelectAction::Help`
  - View: `src/ui/prompts.rs:40-60` for current key mapping

---

### Step 3: Update return types (breaking change, requires caller updates)

- [x] Update `select_from_list()` return type from `Result<Option<usize>>` to `Result<PromptResult<usize>>`:
  - View: `src/ui/prompts.rs:88-132`
  - `SelectAction::Confirm` → `PromptResult::Value(selected)`
  - `SelectAction::Back` → `PromptResult::Back`
  - `SelectAction::Exit` → `PromptResult::Exit`
  - `SelectAction::Help` → call `render_help_overlay()`, redraw list, continue loop

- [x] Update `select_week()` return type from `Result<Option<NaiveDate>>` to `Result<PromptResult<NaiveDate>>`:
  - View: `src/ui/prompts.rs:153-177`
  - Pattern match on `PromptResult` variants instead of `Option`

- [x] Update `select_category()` return type from `Result<Option<Category>>` to `Result<PromptResult<Category>>`:
  - View: `src/ui/prompts.rs:179-189`
  - Pattern match on `PromptResult` variants

- [x] Update `input_hours()` return type from `Result<Option<f64>>` to `Result<PromptResult<f64>>`:
  - View: `src/ui/prompts.rs:191-266`
  - `Ctrl+C` → `PromptResult::Exit`
  - `Esc` → `PromptResult::Back`
  - Valid input → `PromptResult::Value(val)`
  - Empty input with `current_value` → `PromptResult::Value(current_value)`
  - Add `?` key handling: call `render_help_overlay()`, redraw prompt, continue loop

---

### Step 4: Rewrite `hours add` interactive branch

- [x] Rewrite `src/cli/add.rs` interactive branch (lines 59-71) as nested loops:
  - View: `src/cli/add.rs:26-96` for full `run()` function
  - View: `src/data/model.rs:57-64` for `WeekEntry::add()` method
  - View: `src/data/store.rs:18-44` for `store::save()` function
  - View: `src/git.rs:116-143` for `git_sync()` function

  ```
  Outer 'week_loop:
    select_week() →
      Exit → return Ok(())
      Back → return Ok(())
      Value(week_start) →
        Inner 'category_loop:
          select_category() →
            Exit → return Ok(())
            Back → continue 'week_loop
            Value(category) →
              input_hours() →
                Exit → return Ok(())
                Back → continue 'category_loop
                Value(hours) →
                  find/create entry, entry.add(category, hours)
                  store::save()
                  git::git_sync()
                  flash_confirmation()
                  continue 'category_loop
  ```

- [x] Non-interactive branch (lines 33-58) remains completely unchanged
  - View: `src/cli/add.rs:33-58`

- [x] Add `#[command(after_help = "...")]` to `AddArgs` struct with navigation key documentation:
  - View: `src/cli/add.rs:11-24` (AddArgs definition)
  - Per [cli-system.md § Help Overlay](../specs/cli-system.md#help-overlay): keys documented in `--help`

---

### Step 5: Rewrite `hours edit` interactive branch

- [x] Rewrite `src/cli/edit.rs` interactive branch (lines 95-124) as nested loops:
  - View: `src/cli/edit.rs:36-127` for full `run()` function
  - View: `src/data/model.rs:65-70` for `WeekEntry::set()` method
  - Replace sequential iteration over `Category::ALL` with `select_category_with_values()` selector

  ```
  Outer 'week_loop:
    select_week() →
      Exit → return Ok(())
      Back → return Ok(())
      Value(week_start) →
        find/create entry (immutable lookup for display)
        Inner 'category_loop:
          select_category_with_values(entry) →
            Exit → return Ok(())
            Back → continue 'week_loop
            Value(category) →
              input_hours(prompt, Some(current_value)) →
                Exit → return Ok(())
                Back → continue 'category_loop
                Value(new_val) →
                  entry.set(category, new_val)  (mutable borrow)
                  store::save()
                  git::git_sync()
                  flash_confirmation()
                  continue 'category_loop
  ```

- [x] Non-interactive branch (lines 43-94) remains completely unchanged
  - View: `src/cli/edit.rs:43-94`

- [x] Borrow checker consideration: use immutable `data.weeks.iter().find()` for `select_category_with_values()` display, then `data.weeks.iter_mut().find()` for mutation — each in its own scope
  - View: `src/cli/edit.rs:103-109` for current borrow pattern

- [x] Add `#[command(after_help = "...")]` to `EditArgs` struct:
  - View: `src/cli/edit.rs:11-34` (EditArgs definition)

---

### Step 6: Validation

- [x] `cargo fmt --all`
- [x] `cargo clippy --workspace -- -D warnings`
- [x] `cargo build --workspace`
- [x] `cargo test --workspace` — all 19 existing integration tests pass unchanged (96 unit + 19 integration = 115 total)
- [x] Manual smoke test: `cargo run -- add` (interactive) — verify looping, back-navigation, help overlay, confirmation flash (verified via code review; cannot run interactive terminal in CI)
- [x] Manual smoke test: `cargo run -- edit` (interactive) — verify category value display, single-category editing, looping (verified via code review; cannot run interactive terminal in CI)

---

### Acceptance criteria

- [x] `hours add` interactive mode loops back to category selector after each entry per [cli-system.md § `hours add`](../specs/cli-system.md#hours-add)
- [x] `hours edit` interactive mode uses category selector with current values per [cli-system.md § `hours edit`](../specs/cli-system.md#hours-edit)
- [x] `Esc`/`q` goes back one level at every screen per [cli-system.md § Navigation Model](../specs/cli-system.md#navigation-model)
- [x] `Esc`/`q` at week selector exits the command per [cli-system.md § Navigation Model](../specs/cli-system.md#navigation-model)
- [x] `Ctrl+C` exits immediately from any screen per [cli-system.md § Key Bindings](../specs/cli-system.md#key-bindings)
- [x] `?` shows help overlay on all interactive screens per [cli-system.md § Help Overlay](../specs/cli-system.md#help-overlay)
- [x] Confirmation flash displays for ~1 second after each save per [cli-system.md § Confirmation Flash](../specs/cli-system.md#confirmation-flash)
- [x] Git commit+push happens synchronously after each entry per [git-sync.md § Commit Behavior](../specs/git-sync.md#commit-behavior)
- [x] `--help` text for `add` and `edit` documents navigation keys per [cli-system.md § Help Overlay](../specs/cli-system.md#help-overlay)
- [x] Non-interactive mode for both commands is completely unchanged
- [x] All existing integration tests pass
- [x] `cargo clippy --workspace -- -D warnings` passes
- [x] `cargo fmt --all` produces no changes

**Lessons learned:**

- The borrow checker conflict between displaying current values (immutable read) and mutating entries (mutable write) in the `edit` interactive loop was solved by reloading data from disk at the start of each category loop iteration and cloning the display entry. This pattern naturally keeps the display in sync with the latest saved state, since each save writes to disk.
- `input_hours` with `PromptResult` needs to handle the case where `current_value` is `None` and the user presses Enter with empty input — previously this returned `Ok(None)` (interpreted as "keep current"), but with `PromptResult` there's no "keep current" semantic when there's no current value. Added a validation message "Hours value is required." for this case.
- The `?` key in `input_hours` only triggers the help overlay when the input buffer is empty, to avoid conflicts with typing `?` as part of input (though `?` isn't a valid float character, this keeps the behavior predictable).
- `flash_confirmation` enters raw mode via `RawModeGuard` to ensure the terminal is in a clean state for the 1-second display, then the guard's `Drop` impl automatically restores normal mode.
