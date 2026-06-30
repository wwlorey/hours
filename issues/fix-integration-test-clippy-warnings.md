---
status: in_progress
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
