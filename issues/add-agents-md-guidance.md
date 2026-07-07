---
status: closed
priority: p3
type: chore
deps: []
---

# Add AGENTS.md contributor guidance

The repository had no top-level guidance file for coding agents. Add an
`AGENTS.md` at the repo root describing the essentials: what Hours is (a
single-binary Rust CLI for tracking counseling licensure hours), the
install-after-change workflow (`just`, whose default recipe is `install` =
`cargo install --path . --force`), the other `just` recipes (`build`, `test`,
`lint`, `fmt`), and conventions pointing at `specs/` and `README.md`.

Also add a relative `CLAUDE.md` symlink pointing at `AGENTS.md` so
Claude-specific tooling picks up the same guidance.

## Source refs

- AGENTS.md — new contributor guidance file
- CLAUDE.md — relative symlink to AGENTS.md (`readlink CLAUDE.md` == `AGENTS.md`)

## Comments

### 2026-07-07 — closed

Added AGENTS.md and the CLAUDE.md → AGENTS.md relative symlink. Recipe names in
AGENTS.md verified against the justfile (`install`, `build`, `test`, `lint`,
`fmt`) — all accurate, no correction needed. Docs-only change; no specs or
source touched. Architecture spec's file tree left as-is per plan (AGENTS.md is
contributor tooling, not a described subsystem).
