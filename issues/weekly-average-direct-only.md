---
status: closed
priority: p2
type: bug
deps: []
---

# Weekly average must count direct hours only

The "weekly average" stat in `hours summary` (and the PDF export) divided
**total** supervised hours by weeks elapsed. The licensure board's weekly-average
requirement applies only to direct client-contact time, so indirect hours and both
supervision categories were inflating the figure. Fix: change the numerator from
`total_hours` to the already-computed `direct_hours`. The denominator
(`weeks_elapsed`, every calendar week in the period) and the `min_weekly_average`
target (default 15.0) are unchanged — only the measurement changed, not the goal.

## Source refs

- src/cli/summary.rs — weekly_average numerator changed total_hours -> direct_hours
- src/pdf.rs — build_progress_summary mirrored the same numerator change
- tests/integration.rs — added summary_weekly_average_counts_direct_only (date-independent: indirect-only logging yields total > 0 but weekly_average == 0.0)

## Doc refs

- specs/summary-system.md — Weekly Average formula + targets table + worked examples updated to direct-only
- specs/config-system.md — min_weekly_average description clarified (direct hours only; default 15.0 unchanged)
- specs/cli-system.md — summary example weekly-average line updated for consistency
- specs/pdf-export.md — progress-summary example weekly-average line updated for consistency

## Comments

### 2026-06-30 — close

Implemented per the approved plan. Reused the existing direct_hours sum — no new
variable, no WeekEntry method, no data-model change (specs/data-model.md already
documents the direct category as "direct client contact").

Design decisions (locked): direct category ONLY (exclude indirect + both supervision
categories); denominator unchanged; min_weekly_average key name and default 15.0
unchanged.

Backpressure: cargo build, cargo fmt --check, and cargo test all pass
(96 unit + 20 integration, including the new assertion). cargo clippy -D warnings
fails on TWO PRE-EXISTING issues in tests/integration.rs unrelated to this change
(line 9 deprecated assert_cmd::Command::cargo_bin; line 488 unnecessary_map_or) —
toolchain/dependency drift, not introduced here. Logged as separate follow-up
fix-integration-test-clippy-warnings. Spec validate reports "no frontmatter" on
all specs (the spec library predates the frontmatter convention) — also pre-existing
and out of scope.

### 2026-06-30 — verify

Verify gate exercised against the real built binary: logging only indirect hours
shows "Weekly average: 0.0" (0.0%); after adding 300 direct hours it rises to
"Weekly average: 4.0" (26.7%). Both halves of the gate assertion pass.

Residual (what a live run would still catch beyond this coverage): the PDF export's
rendered weekly-average value was verified only via the shared code path and unit
tests, not by opening a generated PDF and reading the glyphs; a human eyeballing an
exported PDF could still catch a layout/label regression. The CLI text path was
verified live end-to-end.
