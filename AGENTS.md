# Agent Guidelines

Guidance for coding agents working in this repository. Hours is a single-binary
Rust CLI (`hours`) for tracking counseling licensure hours.

## After changing the package

After making changes to the crate, install the updated binary so the `hours`
command on your PATH reflects your work:

```bash
just
```

A bare `just` runs the default recipe, `install`, which is
`cargo install --path . --force`.

## Other recipes

```bash
just build   # cargo build
just test    # cargo test --workspace
just lint    # cargo clippy --workspace -- -D warnings
just fmt     # cargo fmt --all
```

Run `just --list` to see all recipes.

## Conventions

- Design documentation lives in [specs/](specs/README.md); keep specs in sync
  with code when you change behavior. Run `specs/validate` to check the library.
- See [README.md](README.md) for user-facing command documentation.
