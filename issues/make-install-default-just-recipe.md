---
status: closed
priority: p3
type: chore
deps: []
---

# Make install the default just recipe

Running a bare `just` (no recipe argument) listed the recipes via a `default`
recipe whose body was `@just --list`. Switch the default so bare `just` builds
and installs the `hours` binary instead. Use the idiomatic just approach: a
`default` recipe whose sole dependency is `install` (`default: install`), not an
alias. Keep `install`, `build`, `test`, `lint`, `fmt` unchanged. Users can still
run `just --list` explicitly to see all recipes.

## Source refs

- justfile — `default: install` (was `default:` / `@just --list`)
- justfile — `install:` recipe (`cargo install --path . --force`) is the new default target
- README.md — Development section; updated to note bare `just` runs install and `just --list` lists recipes

## Comments

### 2026-06-30 — closed

Replaced the `default:` recipe body (`@just --list`) with `default: install`.
README Development section updated: removed "run `just` to list them", added that
a bare `just` runs install and `just --list` lists recipes; added a `just`
default line to the alias block. No spec documents the default `just` behavior,
so no spec change was needed (`specs/validate` not run).

Verification: `just --list` shows build, default, fmt, install, lint, test;
`just --dry-run` (bare) prints `cargo install --path . --force`. Cargo gates
untouched (no Rust source changed).
