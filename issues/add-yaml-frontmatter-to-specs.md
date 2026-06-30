---
status: open
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
