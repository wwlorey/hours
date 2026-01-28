# Data Model

> **Spec:** `specs/data-model.md`
> **Code:** `src/data/`

## Overview

All hours data is stored in a single JSON file (`hours.json`) in the configured data directory. The file contains an array of week entries, sorted by start date ascending.

## JSON Schema

### `hours.json`

```json
{
  "weeks": [
    {
      "start": "2025-01-28",
      "end": "2025-02-03",
      "individual_supervision": 1.0,
      "group_supervision": 2.0,
      "direct": 14.5,
      "indirect": 6.0
    }
  ]
}
```

### Field Definitions

| Field | Type | Description |
|-------|------|-------------|
| `start` | `String` (ISO 8601 date, `YYYY-MM-DD`) | Tuesday start of the week |
| `end` | `String` (ISO 8601 date, `YYYY-MM-DD`) | Monday end of the week |
| `individual_supervision` | `f64` | Hours of one-on-one supervision |
| `group_supervision` | `f64` | Hours of group supervision |
| `direct` | `f64` | Hours of direct client contact |
| `indirect` | `f64` | Hours of indirect work (documentation, admin, etc.) |

### Rust Types

```rust
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HoursData {
    pub weeks: Vec<WeekEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeekEntry {
    pub start: NaiveDate,
    pub end: NaiveDate,
    pub individual_supervision: f64,
    pub group_supervision: f64,
    pub direct: f64,
    pub indirect: f64,
}

impl WeekEntry {
    /// Total hours for this week across all categories.
    pub fn total(&self) -> f64 {
        self.individual_supervision
            + self.group_supervision
            + self.direct
            + self.indirect
    }
}
```

## Hour Categories

The four hour categories map to CLI flag names as follows:

| Category | JSON key | CLI flag | Display name |
|----------|----------|----------|--------------|
| Individual Supervision | `individual_supervision` | `--individual-supervision` | Ind Sv |
| Group Supervision | `group_supervision` | `--group-supervision` | Grp Sv |
| Direct | `direct` | `--direct` | Direct |
| Indirect | `indirect` | `--indirect` | Indirect |

A Rust enum is used for category selection in the CLI:

```rust
pub enum Category {
    IndividualSupervision,
    GroupSupervision,
    Direct,
    Indirect,
}
```

## Week Calculation

Weeks always run **Tuesday through Monday**.

### Current Week Algorithm

```
Given a date `today`:
  1. weekday_num = today.weekday().num_days_from_monday()
     (Monday=0, Tuesday=1, ..., Sunday=6)
  2. days_since_tuesday = (weekday_num + 6) % 7
     (Tuesday=0, Wednesday=1, ..., Monday=6)
  3. start = today - days_since_tuesday days
  4. end = start + 6 days
```

`start` is always a Tuesday. `end` is always the following Monday.

### Examples

| Today | weekday_num | days_since_tuesday | start (Tue) | end (Mon) |
|-------|-------------|-------------------|-------------|-----------|
| 2025-01-28 (Tue) | 1 | 0 | 2025-01-28 | 2025-02-03 |
| 2025-01-30 (Thu) | 3 | 2 | 2025-01-28 | 2025-02-03 |
| 2025-02-03 (Mon) | 0 | 6 | 2025-01-28 | 2025-02-03 |
| 2025-02-04 (Tue) | 1 | 0 | 2025-02-04 | 2025-02-10 |

### Week List Generation

When presenting available weeks (see [cli-system.md § Week Selector](./cli-system.md#week-selector)), generate all Tue–Mon weeks from the licensure `start_date` (see [config-system.md](./config-system.md)) through the current week. This is computed by iterating from `start_date` in 7-day increments.

## Invariants

- All hour values must be `>= 0.0`.
- `end` must equal `start + 6 days`.
- `start` must be a Tuesday (`chrono::Weekday::Tue`).
- No duplicate weeks: each `start` date appears at most once.
- The `weeks` array is sorted by `start` date ascending.
- On every write, re-sort the array and validate all invariants before persisting.

## Atomic Writes

To prevent data corruption from interrupted writes:

1. Serialize `HoursData` to a JSON string with `serde_json::to_string_pretty`.
2. Write to `hours.json.tmp` in the same directory.
3. Call `fsync` on the temp file handle.
4. Rename `hours.json.tmp` to `hours.json` (atomic on POSIX).

This ensures `hours.json` is never in a partially-written state. The `.tmp` file is in `.gitignore` (see [git-sync.md § Initialization](./git-sync.md#initialization)).

## Empty State

A freshly-initialized `hours.json` contains:

```json
{
  "weeks": []
}
```

All commands must handle this empty state gracefully (see [cli-system.md § Empty State](./cli-system.md#hours-list) notes on individual commands).
