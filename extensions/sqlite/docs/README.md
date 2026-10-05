# sqlite

SQLite databases for X_eTaL: run statements, and bring query results
back as X_eTaL arrays -- numbers as a matrix to compute on, text as a
Char matrix -- through rusqlite with SQLite compiled in.

- Package: `extensions/sqlite/` (`extension.toml`; library
  `xetal_ext_sqlite`: `libxetal_ext_sqlite.dylib` on macOS, `.so` on
  Linux)
- Facade: `lib/Sqlite.xtl`, recommended alias `sq:`
- Native crates: `rusqlite` 0.32 with `bundled` (SQLite itself, no
  system library); `tempfile` for tests

## From X_eTaL

```
"sq:" u_se< "Sqlite"
db := "work/pets.db"
db sq:e_xec "create table pets (name text, kg real, legs integer)"
db sq:e_xec "insert into pets values ('Rex', 31.5, 4), ('Polly', 0.4, 2)"
m := db sq:n_ums "select kg, legs from pets"     # a 2 by 2 Float matrix
(1 s_elect_2 m) / 2 s_elect_2 m                    # kg per leg, in X_eTaL
db sq:t_exts "select name from pets"               # Rex / Polly, one per row
```

| Export | Type | What |
| ------ | ---- | ---- |
| `db sq:e_xec sql` | `Char -> Char -> Int` | run the statements (several, separated by `;`); how many rows they changed |
| `db sq:n_ums sql` | `Char -> Char -> Float` | the result as a Float matrix, rows by columns; NULL is NaN; text that is not a number is an error |
| `db sq:t_exts sql` | `Char -> Char -> Char` | every cell as text: a Char matrix with one row per cell (the first row's cells, then the second's), padded with spaces; NULL is empty |
| `db sq:c_ols sql` | `Char -> Char -> Char` | the query's column names, one per row |
| `sq:q_uote t` | `Char -> Char` | the text as an SQL literal (`'O''Brien'`): put a value into a statement as data, never as SQL |
| `db sq:i_mport spec` | `Char -> Char -> Int` | a CSV file into a new table: `"path.csv"` (the table named after the file) or `"table=path.csv"`; how many rows |

Native functions: `exec`, `nums`, `texts`, `cols`, `quote`, `import`
(`just list`).

## CSV import

```
db sq:i_mport "tests/data/planets.csv"                     # 6 rows, table planets
m := db sq:n_ums "select au, days from planets order by au"
a := 1 s_elect_2 m
t := 2 s_elect_2 m
f_loor 0.5 + (t * t) / a * a * a                           # Kepler: about 365.25 squared for each
```

The first record names the columns. A column is INTEGER if every
non-empty field is an integer, else REAL if every one is a number, else
TEXT; an empty field is NULL. Fields follow RFC 4180: quoted fields may
hold commas, doubled quotes and newlines; CRLF or LF line ends. The
table must not exist yet (drop it first with `sq:e_xec`); a record with
the wrong number of fields is an error naming it; the whole import is
one transaction. The file's path is confined as a database's is.

## The data notebook

`demos/notebook.xtl` (`just demo notebook`; pictures in `work/draw/`):
atmospheric CO2 at Mauna Loa, 1959-2025 (NOAA GML and Scripps;
`demos/data/PROVENANCE.txt`), loaded from CSV into SQLite and analyzed
by SQL and X_eTaL together -- each doing what it is good at:

| Step | Who | What it finds |
| ---- | --- | ------------- |
| what is there | SQL | 67 years, 315.98 to 427.35 ppm |
| decade means | SQL `group by` | 316.0 in the 1950s to 420.4 in the 2020s |
| the rise each year | X_eTaL: the series minus itself shifted | 1.69 ppm a year on average; 0.86 in the first decade, 2.63 in the last |
| a least-squares line | X_eTaL, with the standard `Stats` library | 1.67 ppm a year; at 2025 the line says 416.43, the air says 427.35 |
| what the line misses | X_eTaL: residuals by thirds | +2.1, -4.14, +2.2 ppm: a curve bending up -- the rise is speeding up |
| a histogram of the rises | X_eTaL: a table of comparisons | half-ppm bins 1 13 12 18 13 7 1 1, drawn as an APL bar chart (`[]G_RID`) |
| the curve | X_eTaL: `[]P_ATH` | the Keeling curve, as SVG |

One program uses a native extension (`Sqlite`), a standard X_eTaL
library (`Stats`) and X_eTaL's own pictures. Its reg-rs test pins the
output and both pictures.

## Databases are paths

There are no handles (ABI V1 has none; plan A15): `db` names a file,
which each call opens (creating it and its directories when missing)
and closes. `":memory:"` is a scratch database that lasts one call.

Paths are confined: relative, without `..`, resolved under the working
directory -- or under `XETAL_SQLITE_ROOT` when it is set. A path that
leaves its root is an X_eTaL error (`tests/confine.xtl`).

## Errors

An SQL error, a path outside the root, or a non-number asked for as a
number is an X_eTaL error with SQLite's message:

```
error[io]: []N_GET: ext:sqlite/nums: near "selec": syntax error in selec 1 at offset 0 at ./lib/Sqlite.xtl:14:1
```

(The location is the function's signature line in the facade; X_eTaL does not yet also name the
program line that called it -- ask E5.)

## Recording

`videos/notebook.webm` (and `.webp`) record the notebook at the
command line (`just videos sqlite`).

## Live

None, by decision (2026-10-03): sqlite runs on the command line
(`xetal-x`). For the record, it would have been within reach: rusqlite 0.40 with `bundled` builds SQLite for the
browser (`wasm32-unknown-unknown`) through `sqlite-wasm-rs`, which
compiles SQLite's C source with a clang that can target WebAssembly.
Apple's clang cannot (`--target=wasm32-unknown-unknown` fails), so a
live notebook needs LLVM's clang (`brew install llvm`, then
`CC_wasm32_unknown_unknown` and `AR_wasm32_unknown_unknown` pointing at
it) on the machine that builds the pages, and rusqlite moved from 0.32
to 0.40. Probed 2026-10-03 (`sqlite-wasm-rs` 0.5.5 has no precompiled
option). In the browser a database would live in memory (or the
browser's private file system), the notebook's CSV bundled into the
page.

## Build and test

```sh
just build      # the native library and xetal-x
just test       # Rust tests (rust/tests/sqlite.rs) and reg-rs tests (tests/)
just list       # the functions as xetal-x sees them
```

The Rust tests call the extension through the loader on databases in a
temporary root: numbers and NULL, text cells, column names, empty
results, `:memory:`, quoting (an injection attempt stays data),
confinement, SQL errors; the CSV reader (quotes, newlines in fields,
CRLF, malformed input), column types, and imports (a typed table, a
named table, a repeated or ragged import refused). The reg-rs tests run `tests/*.xtl` through
`xetal-x`, pin the facade's types, and check the error messages.
