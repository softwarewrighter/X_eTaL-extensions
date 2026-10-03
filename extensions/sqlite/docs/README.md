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
error[io]: []N_GET: ext:sqlite/nums: near "selec": syntax error in selec 1 at offset 0 at ./lib/Sqlite.xtl:12:17
```

(The location is the facade's; X_eTaL does not yet also name the
program line that called it -- ask E5.)

## Live

None: rusqlite compiles SQLite's C source, which does not build for the
browser's WebAssembly target without more work. The extension runs on
the command line (`xetal-x`).

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
