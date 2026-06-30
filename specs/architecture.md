---
status: implemented
refs: [cli-system, config-system, data-model, git-sync]
---

# Architecture

> **Spec:** `specs/architecture.md`
> **Code:** `src/`

## Overview

Hours is a single-binary Rust CLI tool for tracking counseling licensure hours. It stores data in a local JSON file, reads configuration from a TOML file, and uses git for backup and version history.

## Architecture

The codebase is a thin `clap` dispatch layer (`src/cli/`) over a data core (`src/data/`), with `config.rs`, `git.rs`, and `pdf.rs` as supporting services and `src/ui/` for interactive prompts. The concrete module layout and runtime flow are detailed in [Project Structure](#project-structure) and [Data Flow](#data-flow) below.

## Project Structure

```
hours/
├── Cargo.toml
├── justfile                 # Dev task recipes (build, test, lint, fmt, install)
├── src/
│   ├── main.rs              # Entry point, clap CLI dispatch
│   ├── cli/
│   │   ├── mod.rs           # CLI module, clap app definition
│   │   ├── init.rs          # `hours init` command
│   │   ├── add.rs           # `hours add` command
│   │   ├── edit.rs          # `hours edit` command
│   │   ├── list.rs          # `hours list` command
│   │   ├── summary.rs       # `hours summary` command
│   │   └── export.rs        # `hours export` command
│   ├── config.rs            # Configuration loading and parsing
│   ├── data/
│   │   ├── mod.rs           # Data module re-exports
│   │   ├── model.rs         # Data types (WeekEntry, HoursData)
│   │   ├── store.rs         # JSON persistence (read/write/atomic save)
│   │   └── week.rs          # Tue–Mon week date calculation
│   ├── git.rs               # Git commit and push operations
│   ├── pdf.rs               # PDF report generation
│   └── ui/
│       ├── mod.rs           # UI module re-exports
│       └── prompts.rs       # Interactive prompts with vim key bindings
├── tests/
│   └── integration.rs       # End-to-end integration tests
├── specs/                   # Specification documents
└── plans/                   # Implementation plans
```

## Data Flow

1. User invokes `hours <command>`.
2. `clap` parses arguments and dispatches to the command handler in `src/cli/`.
3. Command handler loads config from `~/.config/hours/config.toml` (see [config-system.md](./config-system.md)).
4. Command handler loads data from `<data_dir>/hours.json` (see [data-model.md](./data-model.md)).
5. For interactive commands (`add`, `edit`, `init`), prompts collect input via `src/ui/prompts.rs` (see [cli-system.md](./cli-system.md)).
6. Data is validated and updated in memory.
7. Updated data is written atomically to `hours.json` (see [data-model.md § Atomic Writes](./data-model.md#atomic-writes)).
8. Git commit and push are attempted; failures warn but do not block (see [git-sync.md](./git-sync.md)).

## Dependencies

| Crate | Purpose |
|-------|---------|
| `clap` (derive) | CLI argument parsing |
| `crossterm` | Raw terminal input for vim-style interactive prompts |
| `chrono` | Date arithmetic for Tue–Mon week calculation |
| `serde` + `serde_json` | JSON serialization and deserialization |
| `toml` | TOML config file parsing |
| `genpdf` | PDF report generation |
| `comfy-table` | Terminal table formatting for `list` and `summary` |
| `dirs` | XDG-compliant home/config directory resolution |
| `shellexpand` | Tilde expansion for paths in config |

### Dev Dependencies

| Crate | Purpose |
|-------|---------|
| `assert_cmd` | Integration test harness for running the compiled binary |
| `assert_fs` | Temporary directories and file assertions |
| `predicates` | Output matching in integration tests |
| `serde_json` | JSON parsing in test assertions |

## Testability

All commands support non-interactive operation for automated testing:

- **`HOURS_CONFIG_DIR`** — Overrides the config directory (see [config-system.md § Environment Variable Overrides](./config-system.md#environment-variable-overrides)).
- **`HOURS_DATA_DIR`** — Overrides `data.directory` from config.
- **`HOURS_NO_GIT=1`** — Disables all git operations.
- **`--non-interactive`** — Accepts all input via CLI flags instead of interactive prompts (see [cli-system.md § Non-Interactive Mode](./cli-system.md#non-interactive-mode)).
- **`--json`** — Machine-parseable JSON output for `list` and `summary` commands.

These flags allow full end-to-end testing from the command line using temporary directories, with no git side effects and no terminal interaction required.

## Error handling

Errors are handled per subsystem rather than centrally: a missing configuration aborts every command except `init` (see [config-system.md](./config-system.md)); data-invariant violations are rejected before persistence (see [data-model.md](./data-model.md)); and git push failures warn on stderr without failing the command (see [git-sync.md](./git-sync.md)).

## Testing

End-to-end coverage lives in `tests/integration.rs`, driven through the non-interactive flags and environment overrides described in [Testability](#testability) above. Subsystem-level unit tests live alongside their modules under `src/`.
