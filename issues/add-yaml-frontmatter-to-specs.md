---
status: closed
priority: p3
type: chore
deps: []
---

# Add YAML frontmatter to spec library

All specs under `specs/` predate the frontmatter convention and lack a YAML
header (no `status:` / `refs:` fields). As a result `specs/validate` reports
"no frontmatter" library-wide, and spec lifecycle automation that keys off
`status:` (e.g. flipping `approved` → `implemented` after code lands) has no
field to operate on.

Surfaced while updating specs for the weekly-average-direct-only change: the
impl worker could not set `status: implemented` because no frontmatter exists.

## Proposed fix

Add a minimal YAML frontmatter block to each `specs/*.md` with at least
`status:` (best-effort: `implemented` for specs whose code exists, `approved`
otherwise) and a `refs:` list of neighbor stems. Re-run `specs/validate` to
confirm the library passes structural validation.

## Source refs

- specs/*.md — entire library (no file currently has frontmatter)

## Comments

Added YAML frontmatter (`status:` + `refs:`) to all 8 `specs/*.md` files. Every
spec was assigned `status: implemented` because each subsystem's described design
is fully shipped in code (verified against `src/`, with passing unit and
integration tests):

- specs/architecture.md — implemented; refs: [cli-system, config-system, data-model, git-sync]
- specs/cli-system.md — implemented; refs: [config-system, git-sync, pdf-export, summary-system]
- specs/config-system.md — implemented; refs: []
- specs/data-model.md — implemented; refs: [config-system, git-sync]
- specs/git-sync.md — implemented; refs: [config-system]
- specs/pdf-export.md — implemented; refs: [config-system, summary-system]
- specs/summary-system.md — implemented; refs: [config-system]
- specs/README.md — implemented; refs: [all seven specs]

`refs:` were derived from the genuine cross-references in each spec's prose and
oriented by dependency direction so the ref graph is an acyclic DAG (no cycle
warnings). The skill's canonical `specs/validate` script did not exist in the
repo, so it was installed from the `specs` skill; that validator also enforces
five required H2 sections (Overview, Architecture, Dependencies, Error handling,
Testing), so the missing required sections were added per spec (additive only —
existing prose and heading anchors were preserved; git-sync's `## Error Handling`
was case-normalized to `## Error handling`, anchor unchanged).

`specs/validate` now passes cleanly across the whole library (exit 0, no errors,
no warnings). Rust gates remain green and untouched: `cargo fmt --check`,
`cargo clippy -D warnings`, and `cargo test` (96 unit + 20 integration tests).
Code-review on the diff surfaced no findings.
