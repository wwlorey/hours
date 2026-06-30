---
status: closed
priority: p3
type: chore
deps: []
---

# Add a justfile with install and dev recipes

The repo documented its dev commands only in the README. A `just` task runner
makes them one-word invocations. Add a `justfile` at the repo root with an
`install` recipe (`cargo install --path . --force`, installing the single
`hours` binary into `~/.cargo/bin`) plus recipes mirroring the README
Development section: `build`, `test`, `lint`, `fmt`. A bare `just` lists the
recipes via a `default` recipe.

## Source refs

- justfile — new task runner with default/install/build/test/lint/fmt recipes
- Cargo.toml — `[package] name = "hours"`, single binary target installed by `install`

## Doc refs

- README.md — Development section; `just` aliases added alongside the existing cargo commands
- specs/architecture.md — Project Structure tree updated to include `justfile`

## Comments

### 2026-06-30 — closed

Shipped `justfile` with recipes: `default` (`@just --list`), `install`
(`cargo install --path . --force`), `build` (`cargo build`), `test`
(`cargo test --workspace`), `lint` (`cargo clippy --workspace -- -D warnings`),
`fmt` (`cargo fmt --all`). Flags mirror the README exactly (note: README's
clippy form omits `--all-targets`, so the recipe does too). README Development
section gained a `just` convenience-alias block; `specs/architecture.md` tree
gained a `justfile` line. `specs/validate` passes.

Verification: `just --list` shows all recipes; `cargo fmt --check`, `cargo
clippy --workspace -- -D warnings`, and `cargo test --workspace` (116 tests) all
pass; `just install` ran the real `cargo install --path . --force`, compiled in
release, replaced `~/.cargo/bin/hours`, and `command -v hours` resolves there.

Residual: the exit condition expected `hours --version` to print `hours 0.1.0`,
but the clap `#[command]` in `src/cli/mod.rs` never sets a `version` attribute,
so no `--version` flag exists. This is pre-existing app behavior unrelated to
the justfile and out of scope here; logged separately as
`add-version-flag-to-cli`.
