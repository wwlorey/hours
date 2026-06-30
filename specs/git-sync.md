---
status: implemented
refs: [config-system]
---

# Git Sync

> **Spec:** `specs/git-sync.md`
> **Code:** `src/git.rs`

## Overview

Every data mutation (`add`, `edit`) automatically commits and pushes the data file to a git remote. This provides version history, backup, and sync across machines. Git operations shell out to the `git` CLI.

## Architecture

Git integration lives in `src/git.rs` and shells out to the `git` CLI, always using `git -C <data_dir>` so operations target the data directory regardless of the working directory. Commit and (optionally) push run synchronously after each successful write to `hours.json` (see [Commit Behavior](#commit-behavior)).

## Dependencies

Requires the `git` CLI on `PATH` (see [Prerequisites](#prerequisites)); no git library is linked. The data directory and remote are established during `hours init`, and push behavior is governed by the `[git]` config section (see [config-system.md § `[git]`](./config-system.md#section-git)).

## Prerequisites

- `git` must be installed and available on `PATH`.
- The data directory must be an initialized git repository with a configured remote.
- Both are established during `hours init` (see [Initialization](#initialization)).

## Commit Behavior

After any successful write to `hours.json`:

1. `git -C <data_dir> add hours.json`
2. `git -C <data_dir> commit -m "<message>"`
3. If `auto_push` is enabled (see [config-system.md § `[git]`](./config-system.md#section-git)): `git -C <data_dir> push <remote> main`

All git commands use `-C <data_dir>` to operate on the data directory regardless of the user's current working directory.

### Commit Messages

| Operation | Message Format |
|-----------|---------------|
| `hours add` | `Add <hours> <category> hours for week of <start_date>` |
| `hours edit` | `Edit hours for week of <start_date>` |
| `hours init` | `Initialize hours tracking` |

Examples:

```
Add 3.5 direct hours for week of 2025-01-28
Edit hours for week of 2025-01-28
Initialize hours tracking
```

## Push Failure Handling

If `git push` fails (network unavailable, auth issue, etc.):

1. Print warning to stderr: `Warning: git push failed: <error>. Data saved locally.`
2. The local commit is preserved — no data is lost.
3. On the next mutating operation, `git push` is attempted again before the new commit. Git will push all unpushed commits.
4. The user can also manually run `git -C <data_dir> push` at any time.

The tool never fails or exits non-zero due to a push failure. Data integrity is always preserved locally.

## Error handling

| Scenario | Behavior |
|----------|----------|
| `git` not installed | `hours init` exits with error: `Error: git is not installed. Install git and try again.` |
| Data dir is not a git repo | Commands exit with error: `Error: Data directory is not a git repository. Run 'hours init' to set up.` |
| Push fails (network) | Warn on stderr, continue. Retry on next operation. |
| Commit fails (nothing to commit) | Silently skip. This can happen if `edit` sets values to the same values. |
| Remote not configured | Warn on stderr: `Warning: No git remote configured. Data is saved locally only.` |

## Initialization

`hours init` performs the following git setup in the data directory:

1. `git init` (if not already a repo).
2. `git remote add <remote_name> <remote_url>` (if remote doesn't exist).
3. Create `.gitignore` containing:
   ```
   *.tmp
   exports/
   ```
4. `git add .gitignore hours.json`
5. `git commit -m "Initialize hours tracking"`
6. `git push -u <remote> main`

If the remote repository does not exist, the push will fail. The user should create the repository on GitHub first. The warning will direct them:

```
Warning: git push failed. Ensure the remote repository exists: <remote_url>
```

## Disabling Git

Set `HOURS_NO_GIT=1` environment variable or pass `--no-git` on any mutating command to skip all git operations. This is the primary mechanism for test isolation (see [architecture.md § Testability](./architecture.md#testability)).

When git is disabled:
- No `git add`, `git commit`, or `git push` calls are made.
- Data is still saved to `hours.json` normally.
- No warnings about git status are printed.

## Testing

Setting `HOURS_NO_GIT=1` (or passing `--no-git`) disables all git operations so integration tests run with no git side effects (see [architecture.md § Testability](./architecture.md#testability)).
