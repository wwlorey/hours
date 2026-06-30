---
status: closed
priority: p3
type: chore
deps: []
---

# Fix pre-existing clippy warnings in integration tests

`cargo clippy --all-targets -- -D warnings` fails on two pre-existing issues in
tests/integration.rs caused by toolchain/dependency drift (not by any current
feature work). These block a clean clippy run.

1. Line 9: deprecated `assert_cmd::Command::cargo_bin` — "incompatible with a custom
   cargo build-dir, see instead `cargo::cargo_bin_cmd!`".
2. Line 488: `clippy::unnecessary_map_or` —
   `.map_or(false, |ext| ext == "pdf")` should be `.is_some_and(|ext| ext == "pdf")`.

Both predate the weekly-average-direct-only change and were surfaced while running
backpressure for it. `cargo build` and `cargo test` are unaffected and pass.

## Source refs

- tests/integration.rs — line 9 (cargo_bin) and line 488 (map_or)

## Comments

Fixed both lints in `tests/integration.rs`. The pinned `assert_cmd` 2.1.2 does
provide the `cargo_bin_cmd!` macro the deprecation warning points to, so applied
the real migration: imported `assert_cmd::cargo::cargo_bin_cmd` and replaced
`Command::cargo_bin("hours").unwrap()` with `cargo_bin_cmd!("hours")` (the macro
returns a `Command` directly, no `.unwrap()` needed). Replaced
`.map_or(false, |ext| ext == "pdf")` with `.is_some_and(|ext| ext == "pdf")`.
Backpressure: `cargo clippy --all-targets -- -D warnings` now clean,
`cargo fmt --check` clean (required reordering the new import), `cargo test`
green (96 unit + 20 integration tests pass).
