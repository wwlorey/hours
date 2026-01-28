# Hours

CLI tool for tracking counseling licensure hours. Tracks supervised and direct client contact hours across Tuesday–Monday weeks, with automatic git-based backup, PDF reporting, and licensure target tracking.

## Installation

Requires Rust 1.70+ and (optionally) Git.

```bash
cargo install --path .
```

## Quick Start

```bash
# First-time setup
hours init

# Log hours
hours add

# Check progress
hours summary

# Generate PDF report
hours export
```

## Commands

### `hours init`

Creates configuration, data directory, and git repository.

```bash
# Interactive (prompts for all settings)
hours init

# Non-interactive
hours init \
  --data-dir ~/Sync/.hours \
  --remote git@github.com:user/hours-data.git \
  --start-date 2025-01-28 \
  --non-interactive
```

### `hours add`

Adds hours to a category for a given week. Hours accumulate (adding 3.5 then 2.0 gives 5.5).

```bash
# Interactive (select week, category, enter hours)
hours add

# Non-interactive
hours add --category direct --hours 3.5 --non-interactive
hours add --week 2025-01-28 --category individual_supervision --hours 1.0 --non-interactive
```

Categories: `individual_supervision`, `group_supervision`, `direct`, `indirect`

### `hours edit`

Sets absolute values for a week's categories. Only specified categories are updated; others are preserved.

```bash
# Interactive (select week, edit each category)
hours edit

# Non-interactive (set direct to 10.0, leave others unchanged)
hours edit --week 2025-01-28 --direct 10.0 --non-interactive
```

### `hours list`

Displays a table of all logged weeks.

```bash
hours list              # Terminal table
hours list --json       # JSON output
hours list --last 4     # Last 4 weeks only
```

### `hours summary`

Shows progress toward licensure targets.

```bash
hours summary           # Terminal display
hours summary --json    # JSON output
```

### `hours export`

Generates a PDF report with weekly hours table and progress summary.

```bash
hours export                          # Default path: <data_dir>/exports/hours-report-YYYY-MM-DD.pdf
hours export --output report.pdf      # Custom output path
hours export --open                   # Open after generating
```

### Global Flags

- `--no-git` — Disable git operations for any command

## Interactive Navigation

Interactive prompts use vim-style key bindings:

| Key | Action |
|-----|--------|
| `j` / `Down` | Move down |
| `k` / `Up` | Move up |
| `Enter` | Confirm |
| `Esc` / `q` | Cancel |
| `g` | Jump to first |
| `G` | Jump to last |

## Configuration

Config file: `~/.config/hours/config.toml`

```toml
[data]
directory = "~/Sync/.hours"

[git]
remote = "origin"
auto_push = true

[licensure]
start_date = "2025-01-28"
total_hours_target = 3000
direct_hours_target = 1200
min_months = 24
min_weekly_average = 15.0
```

### Environment Variable Overrides

| Variable | Overrides | Purpose |
|----------|-----------|---------|
| `HOURS_CONFIG_DIR` | Config directory path | Point to alternate config |
| `HOURS_DATA_DIR` | `data.directory` | Point to alternate data dir |
| `HOURS_NO_GIT` | Disables all git operations | Testing / offline use |

## Data Storage

Data is stored as JSON in the configured data directory (default: `~/Sync/.hours/hours.json`). Writes are atomic (write to temp file, fsync, rename) to prevent corruption.

### Git Sync

Every `add` and `edit` automatically commits and pushes changes. If push fails (network, auth), a warning is printed but data is saved locally. The next operation retries pushing all unpushed commits.

Disable git with `--no-git` or `HOURS_NO_GIT=1`.

### Week Calculation

Weeks run Tuesday through Monday. The `--week` flag accepts a Tuesday date in `YYYY-MM-DD` format. When omitted, the current week is used.

## Development

```bash
cargo build                              # Build
cargo test --workspace                   # Run all tests
cargo clippy --workspace -- -D warnings  # Lint
cargo fmt --all                          # Format
```

## Specifications

Detailed design documentation is available in [specs/README.md](specs/README.md).

## License

MIT License. See [LICENSE](LICENSE).

Bundled Liberation Sans fonts are licensed under the [SIL Open Font License 1.1](assets/fonts/LICENSE).
