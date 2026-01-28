# PDF Export

> **Spec:** `specs/pdf-export.md`
> **Code:** `src/pdf.rs`

## Overview

The `hours export` command generates a clean PDF report containing all tracked weeks and a progress summary. The report is designed to serve as supporting documentation for a licensing board.

## Report Layout

### Header

Centered at the top of the first page:

```
Counseling Licensure Hours Report
Generated: January 27, 2026
Tracking period: January 28, 2025 – January 26, 2026
```

- "Generated" date is the current date at time of export.
- "Tracking period" runs from the configured `start_date` (see [config-system.md § `[licensure]`](./config-system.md#section-licensure)) through the end date of the most recent logged week.

### Hours Table

A table with one row per logged week, sorted by start date ascending. Only weeks with at least one non-zero value are included.

| Column | Content | Alignment |
|--------|---------|-----------|
| Week | `Mon DD – Mon DD, YYYY` (e.g., `Jan 28 – Feb 03, 2025`) | Left |
| Ind. Supervision | Hours as decimal with 1 decimal place | Right |
| Grp. Supervision | Hours as decimal with 1 decimal place | Right |
| Direct | Hours as decimal with 1 decimal place | Right |
| Indirect | Hours as decimal with 1 decimal place | Right |
| Total | Row sum with 1 decimal place | Right |

The final row is a **Totals** row with bold text showing column sums.

### Progress Summary

Below the table, separated by a horizontal rule:

```
Licensure Progress Summary
───────────────────────────────────────
Total supervised hours:    247.0 / 3000  ( 8.2%)
Direct client hours:       156.0 / 1200  (13.0%)
Months of experience:        2  /   24   ( 8.3%)
Weekly average:             15.4 hrs/week (target: 15.0)
Weeks logged:               16
```

Calculations are identical to `hours summary` (see [summary-system.md](./summary-system.md)).

## File Output

- **Default path:** `<data_dir>/exports/hours-report-YYYY-MM-DD.pdf`
  - `YYYY-MM-DD` is the date of export.
  - The `exports/` subdirectory is created automatically if it doesn't exist.
- **Custom path:** The `--output PATH` flag overrides the default.
- The `exports/` directory is listed in `.gitignore` (see [git-sync.md § Initialization](./git-sync.md#initialization)) — PDF files are generated artifacts, not tracked in git.

## Page Format

| Property | Value |
|----------|-------|
| Paper size | US Letter (8.5" × 11") |
| Margins | 1" on all sides |
| Font family | Helvetica (built-in PDF font) |
| Title font size | 16pt bold |
| Header font size | 10pt regular |
| Table header font size | 9pt bold |
| Table body font size | 9pt regular |
| Summary font size | 10pt regular |

## PDF Generation

Uses the `genpdf` crate (wraps `printpdf`). The `genpdf` crate supports:

- Document metadata (title, author).
- Styled text (bold, regular, font sizes).
- Tables with cell alignment and borders.
- Automatic page breaks when the table exceeds one page.

Font handling: `genpdf` requires font files to be loaded. The project bundles the Liberation Sans font family (SIL Open Font License) in an `assets/fonts/` directory. These are embedded at compile time or loaded at runtime from a known path.

**Alternative:** If bundling fonts proves complex, fall back to the `printpdf` built-in PDF fonts (Helvetica, Courier, Times) used directly, with manual table layout. This avoids external font dependencies at the cost of more layout code.

## Empty State

If no hours are logged, the PDF contains only the header and a note:

```
No hours have been logged yet.
```
