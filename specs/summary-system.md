# Summary System

> **Spec:** `specs/summary-system.md`
> **Code:** `src/cli/summary.rs`

## Overview

The `hours summary` command displays progress toward all licensure targets as raw numbers with percentages. The same calculations are reused by the PDF export (see [pdf-export.md](./pdf-export.md)).

## Licensure Targets

All targets are configurable in `config.toml` (see [config-system.md § `[licensure]`](./config-system.md#section-licensure)):

| Target | Config key | Default | Description |
|--------|-----------|---------|-------------|
| Total supervised hours | `total_hours_target` | 3,000 | Sum of all four hour categories across all weeks |
| Direct client hours | `direct_hours_target` | 1,200 | Sum of `direct` across all weeks |
| Minimum months | `min_months` | 24 | Calendar months from start date to today |
| Weekly average | `min_weekly_average` | 15.0 | Total hours ÷ number of weeks elapsed |

## Calculations

### Total Supervised Hours

```
total = Σ week.total() for all weeks
     = Σ (individual_supervision + group_supervision + direct + indirect)
percentage = total / total_hours_target × 100
```

### Direct Client Hours

```
direct_total = Σ week.direct for all weeks
percentage = direct_total / direct_hours_target × 100
```

### Months of Experience

```
months = count of complete calendar months from start_date to today
percentage = months / min_months × 100
```

Calculation: from `start_date` to `today`, count the number of complete months. Use `chrono` date arithmetic: `(today.year() - start.year()) * 12 + (today.month() - start.month())`, adjusted if `today.day() < start.day()`.

### Weekly Average

```
weeks_elapsed = number of Tue–Mon weeks from start_date through current week
              = ((current_week_start - start_date).num_days() / 7) + 1
average = total_hours / weeks_elapsed
percentage = average / min_weekly_average × 100
```

`weeks_elapsed` counts all weeks in the tracking period, including weeks with zero logged hours. This reflects the licensure board's requirement for an _average_ of 15 hours/week across the full period.

## Display Format

Terminal output:

```
Licensure Progress
══════════════════════════════════════════════════

Total supervised hours:    247.0 / 3000   (  8.2%)
Direct client hours:       156.0 / 1200   ( 13.0%)
Months of experience:        2   /   24   (  8.3%)
Weekly average:             15.4 /   15.0 (102.7%)

Weeks logged: 16
Date range: Jan 28, 2025 – May 19, 2025
```

Formatting rules:

- Numbers are right-aligned within their columns.
- Percentages are displayed with one decimal place.
- "Date range" shows the start date of the first logged week through the end date of the last logged week.
- "Weeks logged" is the count of weeks with any non-zero data.

## JSON Output

With `--json`, output a single JSON object:

```json
{
  "total_hours": {
    "current": 247.0,
    "target": 3000,
    "percentage": 8.2
  },
  "direct_hours": {
    "current": 156.0,
    "target": 1200,
    "percentage": 13.0
  },
  "months": {
    "current": 2,
    "target": 24,
    "percentage": 8.3
  },
  "weekly_average": {
    "current": 15.4,
    "target": 15.0,
    "percentage": 102.7
  },
  "weeks_logged": 16,
  "start_date": "2025-01-28",
  "latest_week_start": "2025-05-13",
  "latest_week_end": "2025-05-19"
}
```

This format is used by integration tests to verify calculation correctness (see [architecture.md § Testability](./architecture.md#testability)).

## Empty State

If no weeks are logged:

```
Licensure Progress
══════════════════════════════════════════════════

Total supervised hours:      0.0 / 3000   (  0.0%)
Direct client hours:         0.0 / 1200   (  0.0%)
Months of experience:        0   /   24   (  0.0%)
Weekly average:              0.0 /   15.0 (  0.0%)

Weeks logged: 0
```

The "Date range" line is omitted when no weeks exist.
