---
status: closed
priority: p3
type: feature
deps: []
---

# Add a `hours browse` subcommand to open the git remote

`hours` had no way to jump from the CLI to the data repo's web page. Added a
`hours browse` subcommand that resolves the data-directory repo's git remote and
opens it in the default browser (macOS `open`, Linux `xdg-open`, matching
`export --open`). A `--print` flag prints the resolved HTTPS URL and exits
without launching a browser.

## What changed

- `src/git.rs` — new `remote_web_url(data_dir, remote_name)` runs a read-only
  `git -C <data_dir> remote get-url <remote>` and normalizes the result. New
  pure `normalize_remote_url` converts scp shorthand (`git@host:user/repo.git`)
  and `ssh://git@host/user/repo.git` to `https://host/user/repo`, strips a single
  trailing `.git` and any trailing slash, and leaves clean `https://` URLs
  unchanged. Host-agnostic.
- `src/cli/browse.rs` — new `BrowseArgs` (`--print`) and `run` handler.
- `src/cli/mod.rs` — registered `mod browse;`, `Browse` variant, dispatch arm.
- `tests/integration.rs` — `browse_print_resolves_remote_web_url` inits a repo
  with SSH remote `git@github.com:test/test.git` and asserts `browse --print`
  emits exactly `https://github.com/test/test`.
- Unit tests in `src/git.rs` cover all four URL forms plus `remote_web_url`
  success/error paths.

## Design decisions

- Remote resolved from the DATA directory repo (not the tool's own repo), reusing
  the existing `git -C <data_dir>` model.
- macOS/Linux only — no Windows arm, matching `export.rs`.
- No `HOURS_BROWSER` env override — surface kept minimal.
- Errors (git missing, dir not a repo, remote unconfigured) exit non-zero with
  wording mirroring `git-sync.md` (e.g. `Error: No git remote 'origin' configured.`).

## Specs updated

- `specs/cli-system.md` — new `### hours browse` section; "six"→"seven" commands;
  `browse.rs` added to the module list.
- `specs/git-sync.md` — note that `browse` is a read-only `git remote get-url`
  against the data dir.
- `README.md` — added `hours browse` to the command list.

## Verification

Backpressure green: `cargo fmt --all --check`, `cargo clippy --workspace
--all-targets -- -D warnings`, `cargo test --workspace` (104 unit + 22
integration). Verify gate (live run): built `hours`, initialized a config with
SSH remote `git@github.com:test/test.git`, and `hours browse --print` printed
exactly `https://github.com/test/test`; `hours browse --help` renders; the
no-remote case exits 1 with `Error: No git remote 'origin' configured.`.
