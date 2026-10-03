//! sqlite: SQLite databases for X_eTaL, through rusqlite with SQLite
//! compiled in. State is named by path (plan A15): each call opens the
//! database file, does its work and closes it, so there are no handles
//! to keep. Paths are confined (see `confine`).

pub mod csv;

use std::path::{Component, Path, PathBuf};

use rusqlite::types::ValueRef;
use rusqlite::{Connection, OpenFlags};
use xetal_ext_sdk::{Array, ArrayData, OwnedError, Value, text};

/// Where database (and CSV) paths are resolved: `XETAL_SQLITE_ROOT`,
/// else the working directory.
fn root() -> Result<PathBuf, OwnedError> {
    match std::env::var_os("XETAL_SQLITE_ROOT") {
        Some(r) => Ok(PathBuf::from(r)),
        None => std::env::current_dir().map_err(|e| OwnedError::failure(e.to_string())),
    }
}

/// A path a program names, resolved under the root: relative, with no
/// `..`, so a program reaches only files under its root.
pub fn confine(path: &str) -> Result<PathBuf, OwnedError> {
    let p = Path::new(path);
    if path.is_empty() || p.is_absolute() {
        return Err(OwnedError::invalid_argument(format!(
            "{path:?}: give a path relative to the working directory (or XETAL_SQLITE_ROOT)"
        )));
    }
    if p.components()
        .any(|c| !matches!(c, Component::Normal(_) | Component::CurDir))
    {
        return Err(OwnedError::invalid_argument(format!(
            "{path:?}: a path may not leave its root (..)"
        )));
    }
    Ok(root()?.join(p))
}

/// Opens the database a program names: a confined file (created when
/// missing), or `:memory:` for a scratch database that lasts one call.
pub fn open(name: &str) -> Result<Connection, OwnedError> {
    let conn = if name == ":memory:" {
        Connection::open_in_memory()
    } else {
        let path = confine(name)?;
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)
                .map_err(|e| OwnedError::failure(format!("{name}: {e}")))?;
        }
        Connection::open_with_flags(
            &path,
            OpenFlags::SQLITE_OPEN_READ_WRITE
                | OpenFlags::SQLITE_OPEN_CREATE
                | OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )
    };
    conn.map_err(|e| OwnedError::failure(format!("{name}: {e}")))
}

fn sql_error(e: &rusqlite::Error) -> OwnedError {
    OwnedError::failure(e.to_string())
}

/// db exec sql: run every statement; how many rows they changed.
fn exec(args: &[Value]) -> Result<Value, OwnedError> {
    let conn = open(&text(&args[0])?)?;
    let before = conn.total_changes();
    conn.execute_batch(&text(&args[1])?)
        .map_err(|e| sql_error(&e))?;
    let changed = conn.total_changes() - before;
    Ok(Value::Int(i64::try_from(changed).unwrap_or(i64::MAX)))
}

/// The rows of a query, each cell mapped by `cell`, with the column count.
fn rows<T>(
    args: &[Value],
    mut cell: impl FnMut(ValueRef<'_>) -> Result<T, OwnedError>,
) -> Result<(usize, Vec<T>, usize), OwnedError> {
    let conn = open(&text(&args[0])?)?;
    let mut stmt = conn.prepare(&text(&args[1])?).map_err(|e| sql_error(&e))?;
    let cols = stmt.column_count();
    let mut out = Vec::new();
    let mut n = 0;
    let mut rows = stmt.query([]).map_err(|e| sql_error(&e))?;
    while let Some(row) = rows.next().map_err(|e| sql_error(&e))? {
        for i in 0..cols {
            out.push(cell(row.get_ref(i).map_err(|e| sql_error(&e))?)?);
        }
        n += 1;
    }
    Ok((n, out, cols))
}

/// db nums sql: the result as a Float matrix (rows by columns); NULL
/// is NaN; text that is not a number is an error.
#[allow(clippy::cast_precision_loss)] // Int to Float, as X_eTaL widens
fn nums(args: &[Value]) -> Result<Value, OwnedError> {
    let (n, cells, cols) = rows(args, |v| match v {
        ValueRef::Null => Ok(f64::NAN),
        ValueRef::Integer(i) => Ok(i as f64),
        ValueRef::Real(x) => Ok(x),
        ValueRef::Text(t) => {
            let t = String::from_utf8_lossy(t);
            t.trim()
                .parse::<f64>()
                .map_err(|_| OwnedError::failure(format!("{t:?} is not a number (use sq:t_exts)")))
        }
        ValueRef::Blob(_) => Err(OwnedError::failure("a BLOB is not a number")),
    })?;
    Array::new(vec![n, cols], ArrayData::Float(cells))
        .map(Value::Array)
        .map_err(|e| OwnedError::failure(e.to_string()))
}

/// The cells as text: SQL text as it is, numbers as SQLite writes
/// them, NULL as the empty text.
fn cell_text(v: ValueRef<'_>) -> Result<String, OwnedError> {
    Ok(match v {
        ValueRef::Null => String::new(),
        ValueRef::Integer(i) => i.to_string(),
        ValueRef::Real(x) => format!("{x}"),
        ValueRef::Text(t) => String::from_utf8_lossy(t).into_owned(),
        ValueRef::Blob(_) => return Err(OwnedError::failure("a BLOB is not text")),
    })
}

/// Text cells as a Char matrix, one row per cell, padded with spaces.
fn char_matrix(cells: &[String]) -> Result<Value, OwnedError> {
    let width = cells.iter().map(|c| c.chars().count()).max().unwrap_or(0);
    let mut data = Vec::with_capacity(cells.len() * width);
    for c in cells {
        let n = c.chars().count();
        data.extend(c.chars());
        data.extend(std::iter::repeat_n(' ', width - n));
    }
    Array::new(vec![cells.len(), width], ArrayData::Char(data))
        .map(Value::Array)
        .map_err(|e| OwnedError::failure(e.to_string()))
}

/// db texts sql: every cell as text, a Char matrix with one row per
/// cell (row-major: the first row's cells, then the second's).
fn texts(args: &[Value]) -> Result<Value, OwnedError> {
    let (_, cells, _) = rows(args, cell_text)?;
    char_matrix(&cells)
}

/// db cols sql: the query's column names, a Char matrix, one per row.
fn cols(args: &[Value]) -> Result<Value, OwnedError> {
    let conn = open(&text(&args[0])?)?;
    let stmt = conn.prepare(&text(&args[1])?).map_err(|e| sql_error(&e))?;
    let names: Vec<String> = stmt
        .column_names()
        .iter()
        .map(|s| (*s).to_owned())
        .collect();
    char_matrix(&names)
}

/// quote x: x as an SQL literal -- text in single quotes with quotes
/// doubled, a number as itself -- for building statements safely.
fn quote(args: &[Value]) -> Result<Value, OwnedError> {
    let lit = match &args[0] {
        Value::Int(i) => i.to_string(),
        Value::Float(x) if x.is_finite() => format!("{x:?}"),
        Value::Float(_) => "NULL".into(),
        Value::Bool(b) => u8::from(*b).to_string(),
        v => format!("'{}'", text(v)?.replace('\'', "''")),
    };
    Ok(Value::Text(lit))
}

/// db import spec: a CSV file into a new table. `spec` is `path.csv`
/// (the table named after the file) or `table=path.csv`; the path is
/// confined as a database's is. The number of rows imported.
fn import(args: &[Value]) -> Result<Value, OwnedError> {
    let mut conn = open(&text(&args[0])?)?;
    let spec = text(&args[1])?;
    let (table, path) = match spec.split_once('=') {
        Some((t, p)) => (t.trim().to_owned(), p.trim().to_owned()),
        None => (csv::table_name(&spec), spec.trim().to_owned()),
    };
    if table.is_empty() {
        return Err(OwnedError::invalid_argument(format!(
            "{spec:?}: no table name"
        )));
    }
    let file = confine(&path)?;
    let body =
        std::fs::read_to_string(&file).map_err(|e| OwnedError::failure(format!("{path}: {e}")))?;
    let n = csv::import(&mut conn, &table, &body)
        .map_err(|e| OwnedError::new(e.code(), format!("{path}: {}", e.message())))?;
    Ok(Value::Int(i64::try_from(n).unwrap_or(i64::MAX)))
}

xetal_ext_sdk::xetal_extension! {
    name: "sqlite",
    version: env!("CARGO_PKG_VERSION"),
    functions: {
        exec: 2, "Char -> Char -> Int", "db exec sql: run the statements; rows changed.";
        nums: 2, "Char -> Char -> Float", "db nums sql: the result as a Float matrix (NULL is NaN).";
        texts: 2, "Char -> Char -> Char", "db texts sql: every cell as text, one row per cell.";
        cols: 2, "Char -> Char -> Char", "db cols sql: the column names, one per row.";
        quote: 1, "a -> Char", "x as an SQL literal (text quoted, quotes doubled).";
        import: 2, "Char -> Char -> Int", "db import path.csv (or table=path.csv): a CSV into a new table; rows.";
    }
}
