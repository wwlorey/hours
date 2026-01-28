# Configuration System

> **Spec:** `specs/config-system.md`
> **Code:** `src/config.rs`

## Overview

Configuration is stored in a TOML file at `~/.config/hours/config.toml`. Environment variables can override settings for testing and CI.

## Config File Location

| Priority | Source | Purpose |
|----------|--------|---------|
| 1 (highest) | `HOURS_CONFIG_DIR` env var | Testing override |
| 2 | `~/.config/hours/` | Default (XDG-compatible) |

The config file is always named `config.toml` within the config directory.

## Config File Format

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

### Section: `[data]`

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `directory` | `String` | `"~/Sync/.hours"` | Path to data directory. Tilde is expanded at runtime. |

### Section: `[git]`

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `remote` | `String` | `"origin"` | Git remote name for push operations |
| `auto_push` | `bool` | `true` | Whether to push after every commit |

### Section: `[licensure]`

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `start_date` | `String` (ISO 8601) | *(required)* | Licensure tracking start date. Must be a Tuesday. |
| `total_hours_target` | `u32` | `3000` | Total supervised hours required |
| `direct_hours_target` | `u32` | `1200` | Direct client contact hours required |
| `min_months` | `u32` | `24` | Minimum months of continuous experience |
| `min_weekly_average` | `f64` | `15.0` | Minimum average hours per week |

## Environment Variable Overrides

| Variable | Overrides | Purpose |
|----------|-----------|---------|
| `HOURS_CONFIG_DIR` | Config directory path | Test isolation — point to a temp dir |
| `HOURS_DATA_DIR` | `data.directory` | Test isolation — point to a temp dir |
| `HOURS_NO_GIT` | Forces `git.auto_push = false` and skips all git operations | Test isolation — no git side effects |

Environment variables take precedence over config file values. This is the primary mechanism for integration test isolation (see [architecture.md § Testability](./architecture.md#testability)).

## Rust Types

```rust
use chrono::NaiveDate;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub data: DataConfig,
    pub git: GitConfig,
    pub licensure: LicensureConfig,
}

#[derive(Debug, Deserialize)]
pub struct DataConfig {
    pub directory: String,
}

#[derive(Debug, Deserialize)]
pub struct GitConfig {
    pub remote: String,
    pub auto_push: bool,
}

#[derive(Debug, Deserialize)]
pub struct LicensureConfig {
    pub start_date: NaiveDate,
    pub total_hours_target: u32,
    pub direct_hours_target: u32,
    pub min_months: u32,
    pub min_weekly_average: f64,
}
```

## Loading Behavior

1. Determine config directory: `HOURS_CONFIG_DIR` env var, or `~/.config/hours/`.
2. Read `config.toml` from that directory.
3. Deserialize into `Config` struct.
4. Apply env var overrides (`HOURS_DATA_DIR`, `HOURS_NO_GIT`).
5. Expand tilde in `data.directory`.

If the config file does not exist, all commands except `hours init` print an error and exit:

```
Error: Configuration not found. Run `hours init` to set up.
```

## Initialization

The `hours init` command creates the config file (see [cli-system.md § `hours init`](./cli-system.md#hours-init)). If the file already exists, `init` warns and asks for confirmation before overwriting.
