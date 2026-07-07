---
status: in_progress
priority: p2
type: bug
deps: []
---

# Weekly average must count all four categories

This **supersedes and reverses** `weekly-average-direct-only`. That change had
narrowed the "weekly average" stat in `hours summary` (and the PDF export) to
divide **direct** hours only by weeks elapsed. It must instead be computed from
**total** supervised counseling experience — the sum of all four week categories
(`direct` + `indirect` + `individual_supervision` + `group_supervision`, i.e.
`w.total()`) divided by `weeks_elapsed`. The weekly-average requirement measures
the overall pace of supervised experience, so every logged category counts.

Only the numerator's category set widens. The denominator (`weeks_elapsed`, every
calendar week in the period) and the `min_weekly_average` target (default 15.0)
are unchanged.

## Source refs

- src/cli/summary.rs — weekly_average numerator changed direct_hours -> total_hours (direct_hours binding retained for the Direct client hours metric)
- src/pdf.rs — build_progress_summary mirrored the same numerator change (separate copy, kept in lock-step)
- tests/integration.rs — summary_weekly_average_counts_direct_only inverted/renamed to summary_weekly_average_counts_all_categories (indirect + group_supervision logged, zero direct, asserts weekly_average > 0.0)

## Doc refs

- specs/summary-system.md — Weekly Average formula, targets table, and worked examples updated to total-based numerator
- specs/config-system.md — min_weekly_average description rewritten (all four categories count; default 15.0 unchanged)
- specs/cli-system.md — summary example weekly-average line recomputed for consistency
- specs/pdf-export.md — progress-summary example weekly-average line recomputed for consistency

## Comments

### 2026-07-07 — supersede

Created to reverse `weekly-average-direct-only` (now marked superseded, staying
closed). Design decisions (locked): numerator is `Σ w.total()` across all four
categories; denominator `weeks_elapsed` unchanged; `min_weekly_average` key name
and 15.0 default unchanged. Worked examples across summary-system.md,
cli-system.md, and pdf-export.md all use weeks_elapsed = 16: 247.0 total ÷ 16 ≈
15.4 hrs/week, 15.4 / 15.0 ≈ 102.9%.
