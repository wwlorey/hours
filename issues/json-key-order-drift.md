---
status: open
priority: p3
type: bug
deps: []
---

# summary --json key order drifts from spec worked examples

`hours summary --json` emits each metric object with keys in **alphabetical**
order (`current`, `percentage`, `target`) because `serde_json::json!` builds a
`BTreeMap` (the `preserve_order` feature is not enabled). The worked-example JSON
blocks in `specs/summary-system.md` instead show insertion order
(`current`, `target`, `percentage`). This affects all four metric objects
(`total_hours`, `direct_hours`, `months`, `weekly_average`), not just one.

Pre-existing drift, surfaced during the `weekly-average-count-all-categories`
code-review gate; present at base commit `06acf87` and unrelated to that
reversal. Fix is a one-way decision: either reorder the spec's example blocks to
alphabetical, or enable serde_json `preserve_order` and assert the documented
order. Low priority — the integration tests parse by key, not position, so
behavior is unaffected; only the spec's illustrative glyphs disagree.

## Source refs

- src/cli/summary.rs — `serde_json::json!` object; key order is alphabetical at render time

## Doc refs

- specs/summary-system.md — JSON Output block shows current/target/percentage per metric

## Comments

### 2026-07-07 — file

Filed from the code-review gate of `weekly-average-count-all-categories`. Not a
regression from that work; left open for a future pass.
