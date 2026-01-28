# CLI System

> **Spec:** `specs/cli-system.md`
> **Code:** `src/cli/`, `src/ui/`

## Overview

The CLI provides six commands for managing licensure hours. Interactive prompts use vim-style key bindings for navigation. All mutating commands support a `--non-interactive` flag for scripting and testing.

## Commands

### `hours init`

First-time setup wizard. Creates config and data directories, initializes the git repository, and sets licensure parameters.

**Interactive flow:**

1. Prompt for data directory path (default: `~/Sync/.hours`).
2. Prompt for GitHub remote URL.
3. Prompt for licensure start date (YYYY-MM-DD, must be a Tuesday).
4. Prompt to confirm default licensure targets or customize them.
5. Write config file to `~/.config/hours/config.toml` (see [config-system.md](./config-system.md)).
6. Create data directory if it doesn't exist.
7. Initialize git repo in data directory (see [git-sync.md § Initialization](./git-sync.md#initialization)).
8. Create empty `hours.json` with `{"weeks": []}`.
9. Initial commit and push.

**Non-interactive mode:**

```
hours init \
  --data-dir ~/Sync/.hours \
  --remote git@github.com:user/hours-data.git \
  --start-date 2025-01-28 \
  --non-interactive
```

Default licensure targets are used in non-interactive mode. To customize, edit the config file afterward.

### `hours add`

Add hours incrementally to a week. Each invocation adds to the existing total for one category in one week.

**Interactive flow:**

1. Display week selector with current week pre-selected (see [Interactive Prompts § Week Selector](#week-selector)).
2. User confirms or navigates to a different week.
3. Display category selector (see [Interactive Prompts § Category Selector](#category-selector)).
4. Prompt for hours (decimal number, must be ≥ 0).
5. Display confirmation: `Added 3.5 direct hours -> week total: 12.5 direct`.
6. Save data, git commit, git push (see [git-sync.md](./git-sync.md)).

If the selected week does not yet exist in `hours.json`, it is created with all categories at `0.0` before adding.

**Non-interactive mode:**

```
hours add --week 2025-01-28 --category direct --hours 3.5 --non-interactive
```

- `--week` — Tuesday start date of the week (ISO 8601). Defaults to current week if omitted.
- `--category` — One of: `individual_supervision`, `group_supervision`, `direct`, `indirect`.
- `--hours` — Decimal number of hours to add.

**Validation:**

- Hours must be ≥ 0.
- Hours must be a valid decimal number.
- Category must be one of the four valid values.
- If `--week` is provided, it must be a Tuesday.

### `hours edit`

Set the total hours for any or all categories in a specific week. Overwrites existing values.

**Interactive flow:**

1. Display week selector (all weeks with existing data, plus current week).
2. Display current values for the selected week.
3. For each category, show current value and prompt for new value. Press Enter to keep the current value unchanged.
4. Display summary of changes.
5. Save data, git commit, git push.

**Non-interactive mode:**

```
hours edit --week 2025-01-28 \
  --individual-supervision 1.0 \
  --group-supervision 2.0 \
  --direct 14.5 \
  --indirect 6.0 \
  --non-interactive
```

Only the categories provided as flags are updated. Omitted categories remain unchanged.

**Validation:**

- Same as `hours add` for individual values.
- At least one category flag must be provided in non-interactive mode.

### `hours list`

Display a table of all logged weeks sorted by start date ascending.

**Output format:**

```
┌────────────────────────┬────────┬────────┬────────┬──────────┬───────┐
│ Week                   │ Ind Sv │ Grp Sv │ Direct │ Indirect │ Total │
├────────────────────────┼────────┼────────┼────────┼──────────┼───────┤
│ Jan 28 – Feb 03, 2025  │   1.0  │   2.0  │  14.5  │     6.0  │  23.5 │
│ Feb 04 – Feb 10, 2025  │   1.0  │   1.5  │  12.0  │     4.0  │  18.5 │
├────────────────────────┼────────┼────────┼────────┼──────────┼───────┤
│ TOTALS                 │  12.0  │  21.0  │ 156.0  │    58.0  │ 247.0 │
└────────────────────────┴────────┴────────┴────────┴──────────┴───────┘
```

**Flags:**

- `--json` — Output as a JSON array of week objects.
- `--last N` — Show only the last N weeks (most recent).

**Empty state:** If no weeks are logged, print `No hours logged yet. Run 'hours add' to get started.`

### `hours summary`

Display progress toward licensure targets. See [summary-system.md](./summary-system.md) for calculation details.

**Output format:**

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

**Flags:**

- `--json` — Output as a JSON object (see [summary-system.md § JSON Output](./summary-system.md#json-output)).

**Empty state:** If no weeks are logged, show all targets at 0 / target (0.0%).

### `hours export`

Generate a PDF report. See [pdf-export.md](./pdf-export.md) for layout details.

**Behavior:**

1. Generate PDF report.
2. Save to `<data_dir>/exports/hours-report-YYYY-MM-DD.pdf`.
3. Print the file path to stdout.
4. Git commit and push (the `exports/` directory is gitignored; the commit captures any pending `hours.json` changes).

**Flags:**

- `--output PATH` — Override output file path.
- `--open` — Open the PDF after generation (macOS: `open`, Linux: `xdg-open`).

## Non-Interactive Mode

Every mutating command (`init`, `add`, `edit`) accepts a `--non-interactive` flag. When set:

- No terminal prompts are displayed.
- All required values must be provided via CLI flags.
- Missing required flags cause an error with a usage message.
- This mode is essential for integration testing (see [architecture.md § Testability](./architecture.md#testability)).

## Interactive Prompts

All interactive prompts are custom-built on the `crossterm` crate for full control over key handling and rendering. They live in `src/ui/prompts.rs`.

### Key Bindings

| Key | Action |
|-----|--------|
| `j` / `↓` | Move selection down |
| `k` / `↑` | Move selection up |
| `Enter` | Confirm selection |
| `Esc` / `q` | Cancel and exit the command |
| `g` | Jump to first item |
| `G` | Jump to last item |

### Week Selector

Displays a scrollable list of weeks, most recent first. The current week is pre-selected and marked. Weeks with existing data show their total hours.

```
Select week:
  > Jan 28 – Feb 03, 2025 (current)     23.5 hrs
    Jan 21 – Jan 27, 2025               18.5 hrs
    Jan 14 – Jan 20, 2025                0.0 hrs
    Jan 07 – Jan 13, 2025               22.0 hrs
    ...
```

The list includes all weeks from the licensure start date (see [config-system.md § `[licensure]`](./config-system.md#section-licensure)) through the current week.

### Category Selector

```
Select category:
  > Individual Supervision
    Group Supervision
    Direct (client contact)
    Indirect
```

### Number Input

Accepts decimal numbers. Validates on Enter. Shows current total for the category in the selected week (if any data exists).

```
Direct hours to add (current total: 9.0): 3.5
```

For `edit` mode, shows the current value as the default:

```
Direct [14.5]: _
```

Press Enter with no input to keep the current value.
