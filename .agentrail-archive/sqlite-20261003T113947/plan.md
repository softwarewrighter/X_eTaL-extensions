# sqlite

Saga 4 of X_eTaL-extensions (docs/plan.md): SQLite for X_eTaL through
rusqlite (bundled SQLite), and the data notebook demo -- a CSV loaded
into SQLite, SQL selects, X_eTaL arrays compute the analysis, an SVG
chart. State is named by path (A15): each call opens the database file
and closes it; files are confined to the working directory (or a root
given in XETAL_SQLITE_ROOT).

Rules as before (CLAUDE.md). Every step gated, documented, committed
with .agentrail/, completed, pushed.

## Steps

1. sqlite -- the extension: exec, numeric and text results, column
   names, SQL quoting, confinement, errors; Rust, reg-rs tests, docs.
2. csv-import -- a CSV into a table (header names, inferred column
   types), row count; tests.
3. notebook -- demos/notebook: a bundled dataset, SQL and array
   analytics, an SVG chart; reg-rs golden; docs.
