---
status: closed
priority: p3
type: feature
deps: []
---

# Repeat the list table column header below the TOTALS row

When `hours list` renders a tall table, the column header row scrolled off the
top of the terminal, leaving data rows without visible labels. Repeat the six
column headers as a final row below the bottom (TOTALS) row so the labels
("Week", "Ind Sv", "Grp Sv", "Direct", "Indirect", "Total") appear at both the
top and the bottom of the table.

The repeated bottom header is rendered plain (no `Attribute::Bold`), identical to
the top header — explicitly confirmed. `UTF8_FULL` already draws a horizontal
rule between rows, fencing the repeated header off from the TOTALS row above and
the bottom border below, so no separator work was needed.

## Source refs

- src/cli/list.rs — factored the six column-header labels into a single shared
  `headers: [&'static str; 6]` binding (single source of truth so top and bottom
  cannot drift); feeds both `set_header(headers.to_vec())` and a new trailing
  `add_row(headers.to_vec())` after the TOTALS row. Confined to the human-table
  `else` branch; the `--json` branch is untouched.
- tests/integration.rs — extended `list_output_table` to capture stdout and
  assert a distinctive header label (`Ind Sv`) appears exactly twice, guarding
  against the header being dropped from either position.

## Doc refs

- specs/cli-system.md — `hours list` output-format fenced sample now shows the
  header row repeated below TOTALS, with a note explaining the plain repeated
  header stays visible on tall tables.

## Comments

### 2026-07-07 — close

Implemented per the approved plan. Design decisions (locked): single-source-of-truth
`headers` binding shared by top and bottom; plain (non-bold) styling for the
repeated bottom header; no separator work (UTF8_FULL supplies the rule).

Backpressure: cargo test, cargo clippy (-D warnings), and cargo fmt --check all
pass. Verify gate: ran the built binary against seeded data — `Ind Sv` appears
above the first data row AND below the TOTALS row; `hours list | grep -c 'Ind Sv'`
returns 2.
