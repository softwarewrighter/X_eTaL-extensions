//! CSV into a table: a small RFC 4180 reader (quoted fields, doubled
//! quotes, newlines inside quotes, CRLF), column types inferred.

use rusqlite::{Connection, params_from_iter, types::Value as Sql};
use xetal_ext_sdk::OwnedError;

/// The records of CSV text: fields, as written (quotes removed).
pub fn records(text: &str) -> Result<Vec<Vec<String>>, OwnedError> {
    let mut out = Vec::new();
    let mut record = Vec::new();
    let mut field = String::new();
    let mut quoted = false; // inside a quoted field
    let mut was_quoted = false; // this field began with a quote
    let mut chars = text.chars().peekable();
    let mut line = 1;
    while let Some(c) = chars.next() {
        if quoted {
            match c {
                '"' if chars.peek() == Some(&'"') => {
                    chars.next();
                    field.push('"');
                }
                '"' => quoted = false,
                c => {
                    if c == '\n' {
                        line += 1;
                    }
                    field.push(c);
                }
            }
            continue;
        }
        match c {
            '"' if field.is_empty() && !was_quoted => {
                quoted = true;
                was_quoted = true;
            }
            '"' => {
                return Err(OwnedError::failure(format!(
                    "line {line}: a quote inside an unquoted field"
                )));
            }
            ',' => {
                record.push(std::mem::take(&mut field));
                was_quoted = false;
            }
            '\r' if chars.peek() == Some(&'\n') => {}
            '\n' => {
                record.push(std::mem::take(&mut field));
                was_quoted = false;
                out.push(std::mem::take(&mut record));
                line += 1;
            }
            c => {
                if was_quoted {
                    return Err(OwnedError::failure(format!(
                        "line {line}: text after a closing quote"
                    )));
                }
                field.push(c);
            }
        }
    }
    if quoted {
        return Err(OwnedError::failure("the file ends inside a quoted field"));
    }
    if !field.is_empty() || !record.is_empty() || was_quoted {
        record.push(field);
        out.push(record);
    }
    Ok(out)
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Kind {
    Integer,
    Real,
    Text,
}

impl Kind {
    fn sql(self) -> &'static str {
        match self {
            Self::Integer => "INTEGER",
            Self::Real => "REAL",
            Self::Text => "TEXT",
        }
    }
}

/// A column's type: INTEGER if every non-empty field is one, else REAL
/// if every one is a number, else TEXT. An empty field is NULL.
pub fn kind<'a>(fields: impl Iterator<Item = &'a str>) -> Kind {
    let mut k = Kind::Integer;
    for f in fields.map(str::trim).filter(|f| !f.is_empty()) {
        if k == Kind::Integer && f.parse::<i64>().is_err() {
            k = Kind::Real;
        }
        if k == Kind::Real && f.parse::<f64>().is_err() {
            return Kind::Text;
        }
    }
    k
}

/// An SQL identifier, quoted (`"a ""b"""`).
fn ident(name: &str) -> String {
    format!("\"{}\"", name.replace('"', "\"\""))
}

/// Creates `table` from the CSV text (the first record names the
/// columns) and inserts every record, in one transaction. The number of
/// rows inserted.
pub fn import(conn: &mut Connection, table: &str, text: &str) -> Result<usize, OwnedError> {
    let mut recs = records(text)?;
    if recs.is_empty() {
        return Err(OwnedError::failure("the file is empty: no header"));
    }
    let header = recs.remove(0);
    let n = header.len();
    if header.iter().any(|h| h.trim().is_empty()) {
        return Err(OwnedError::failure("a column has no name in the header"));
    }
    for (i, r) in recs.iter().enumerate() {
        if r.len() != n {
            return Err(OwnedError::failure(format!(
                "record {} has {} fields; the header has {n}",
                i + 2,
                r.len()
            )));
        }
    }
    let kinds: Vec<Kind> = (0..n)
        .map(|c| kind(recs.iter().map(|r| r[c].as_str())))
        .collect();
    let cols: Vec<String> = header
        .iter()
        .zip(&kinds)
        .map(|(h, k)| format!("{} {}", ident(h.trim()), k.sql()))
        .collect();
    let fail = |e: rusqlite::Error| OwnedError::failure(e.to_string());
    let tx = conn.transaction().map_err(fail)?;
    tx.execute(
        &format!("CREATE TABLE {} ({})", ident(table), cols.join(", ")),
        [],
    )
    .map_err(fail)?;
    {
        let marks = vec!["?"; n].join(", ");
        let mut stmt = tx
            .prepare(&format!("INSERT INTO {} VALUES ({marks})", ident(table)))
            .map_err(fail)?;
        for r in &recs {
            let values = r.iter().zip(&kinds).map(|(f, k)| {
                let f = f.trim();
                match (f.is_empty(), k) {
                    (true, _) => Sql::Null,
                    (false, Kind::Integer) => Sql::Integer(f.parse().unwrap_or_default()),
                    (false, Kind::Real) => Sql::Real(f.parse().unwrap_or_default()),
                    (false, Kind::Text) => Sql::Text(f.to_owned()),
                }
            });
            stmt.execute(params_from_iter(values)).map_err(fail)?;
        }
    }
    tx.commit().map_err(fail)?;
    Ok(recs.len())
}

/// The table a file names by default: its stem, letters, digits and
/// underscores kept, anything else as an underscore.
pub fn table_name(path: &str) -> String {
    let stem = std::path::Path::new(path)
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_default();
    stem.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect()
}
