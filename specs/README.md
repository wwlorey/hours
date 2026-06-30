---
status: implemented
refs: [architecture, cli-system, config-system, data-model, git-sync, pdf-export, summary-system]
---

# Hours Specifications

Design documentation for Hours, a Rust CLI tool for tracking counseling licensure hours.

## Overview

This directory is the spec library for Hours: design documentation for the shipped CLI, with one markdown file per design unit. The index tables below map each spec to the code that implements it.

## Architecture

The library is flat — one file per design unit, with the filename stem serving as the spec ID and no subdirectories. The entry point is [architecture.md](./architecture.md), which links down to the per-subsystem specs covering the [CLI](./cli-system.md), [data model](./data-model.md), [configuration](./config-system.md), [git sync](./git-sync.md), [PDF export](./pdf-export.md), and [summary system](./summary-system.md).

## Dependencies

Each spec declares its neighbor specs in its `refs:` frontmatter, forming a cross-reference graph across the seven specs indexed below. The specs collectively describe the Rust crate under [src/](../src/).

## Error handling

Structural problems — missing or malformed frontmatter, an unknown `status:` value, a missing required section, an unresolved `refs:` entry, or a cycle in the ref graph — are reported by `specs/validate`.

## Testing

Run `specs/validate` from anywhere in the repo to validate the whole library; it parses frontmatter, checks the required H2 sections, resolves refs, and detects cycles, exiting non-zero on any structural error.

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
