---
status: in_progress
priority: p3
type: chore
deps: []
---

# Add a --version flag to the hours CLI

`hours --version` currently errors with "unexpected argument '--version'
found". The clap derive `#[command(name = "hours", about = "...")]` in
`src/cli/mod.rs` does not set the `version` attribute, so clap never generates a
`--version`/`-V` flag. Add `version` (e.g. `#[command(version, ...)]`, which
picks up `CARGO_PKG_VERSION`) so the binary reports `hours 0.1.0`.

Surfaced while verifying the `just install` recipe — the install succeeds but
the documented version check could not pass.

## Source refs

- src/cli/mod.rs — clap `#[command(...)]` attribute on the top-level `Cli` struct; add `version`

## Comments

### 2026-06-30 — open

Discovered during the justfile work (`add-just-install-recipe`). Not fixed
there to keep that change scoped to tooling/docs.
