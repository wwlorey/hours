# Hours Specifications

Design documentation for Hours, a Rust CLI tool for tracking counseling licensure hours.

## Core Architecture

| Spec | Code | Purpose |
|------|------|---------|
| [architecture.md](./architecture.md) | [src/](../src/) | Project structure, data flow, dependencies, testability |

## CLI System

| Spec | Code | Purpose |
|------|------|---------|
| [cli-system.md](./cli-system.md) | [src/cli/](../src/cli/), [src/ui/](../src/ui/) | Commands, interactive prompts, vim-style navigation |

## Data Model

| Spec | Code | Purpose |
|------|------|---------|
| [data-model.md](./data-model.md) | [src/data/](../src/data/) | JSON schema, week calculation (Tue–Mon), hour categories |

## Configuration

| Spec | Code | Purpose |
|------|------|---------|
| [config-system.md](./config-system.md) | [src/config.rs](../src/config.rs) | TOML config at `~/.config/hours/`, env var overrides, licensure targets |

## Git Sync

| Spec | Code | Purpose |
|------|------|---------|
| [git-sync.md](./git-sync.md) | [src/git.rs](../src/git.rs) | Auto-commit, auto-push, offline resilience, push retry |

## PDF Export

| Spec | Code | Purpose |
|------|------|---------|
| [pdf-export.md](./pdf-export.md) | [src/pdf.rs](../src/pdf.rs) | Report generation with weekly hours table and progress summary |

## Summary & Progress

| Spec | Code | Purpose |
|------|------|---------|
| [summary-system.md](./summary-system.md) | [src/cli/summary.rs](../src/cli/summary.rs) | Licensure target tracking, percentages, weekly averages |

---

## Implementation Plans

| Plan | Purpose |
|------|---------|
| [plans/implementation/plan.md](../plans/implementation/plan.md) | Phased build plan with spec citations and test strategy |
