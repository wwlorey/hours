---
status: open
priority: p3
type: security
deps: []
---

# Validate resolved browse URL is `https://` before opening

Severity: LOW (INTRODUCED-BY-SLATE, commit 29cf527)

## Problem

`normalize_remote_url` (`src/git.rs`) returns the raw git remote **verbatim**
through its final `else` branch whenever the remote contains no `:` — e.g. a
local-path remote (`/some/path`, `../repo`) or, if `.git/config` is edited
directly, a `-`-leading token. `remote_web_url` returns that string unchecked,
and `hours browse` hands it to the OS opener in `src/cli/browse.rs` (`open` /
`xdg-open`) as the first positional argv with no `--` separator and no scheme
validation.

Consequences:
- A `-`-leading value is parsed as a **flag** by `open`/`xdg-open` (argv-level
  flag injection — not shell injection; the value is passed as a discrete argv
  element, so metacharacters are safe).
- A local-path value causes the OS to open an arbitrary local file/dir instead
  of a web page.

Exploitability is low: it requires the attacker to control the user's own
data-directory git config. Colon-bearing malicious schemes (`javascript:`,
`file:`) are already neutralized into `https://host/...` by the scp-shorthand
branch, so they are not affected.

## Proposed fix

In `remote_web_url` (`src/git.rs`, after computing the normalized URL, before
returning `Ok(...)`), assert the result begins with `https://`:

```rust
let url = normalize_remote_url(&raw);
if !url.starts_with("https://") {
    bail!("Refusing to open non-HTTPS remote URL: {url}");
}
Ok(url)
```

This single check rejects both `-`-leading tokens (they can't start with
`https://`) and any non-http(s) scheme or local path. Add a unit test covering a
colon-less remote (e.g. a local path and a `-n` value) asserting `remote_web_url`
errors.

## Citation

- `src/git.rs` — `normalize_remote_url` colon-less passthrough
- `src/git.rs` — `remote_web_url` returns unchecked
- `src/cli/browse.rs` — value handed to `open`/`xdg-open`
