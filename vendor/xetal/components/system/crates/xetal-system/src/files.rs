//! Text files and the keyboard. These are effects, so a program runs
//! them only when it is run (never on a keystroke in the editor).

use xetal_base::Diagnostic;
use xetal_value::Value;

use crate::text::{chars, text};

fn io(e: impl std::fmt::Display, what: &str) -> Diagnostic {
    Diagnostic::new("io", format!("{what}: {e}"))
}

/// `t []N_PUT path`: write text t to the file (made, with its
/// directories, if missing; replaced if there); how many characters.
/// Files are in the store the host installed (`xetal-store`): the disk,
/// or the browser's local storage in the live demo.
pub(crate) fn put<'a>(t: &Value<'a>, path: &Value<'a>) -> Result<Value<'a>, Diagnostic> {
    let (t, path) = (chars(t)?, chars(path)?);
    xetal_store::write(&path, &t).map_err(|e| io(e, "[]N_PUT"))?;
    Ok(Value::Int(t.chars().count() as i64))
}

/// `[]N_GET path`: the file's text.
pub(crate) fn get<'a>(path: &Value<'a>) -> Result<Value<'a>, Diagnostic> {
    let path = chars(path)?;
    xetal_store::read(&path)
        .map(|s| text(&s))
        .map_err(|e| io(e, "[]N_GET"))
}

/// `[]R_EAD @`: a line typed at the keyboard, without its newline
/// (standard input on the command line; the browser asks for it).
pub(crate) fn read<'a>() -> Result<Value<'a>, Diagnostic> {
    xetal_store::read_line()
        .map(|line| text(&line))
        .map_err(|e| io(e, "[]R_EAD"))
}
