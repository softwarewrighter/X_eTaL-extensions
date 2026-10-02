//! Text values (Char vectors), and numbers as text.

use std::rc::Rc;

use xetal_array::Array;
use xetal_base::Diagnostic;
use xetal_value::{Value, as_array, to_value};

/// A Char vector holding `s`.
pub(crate) fn text<'a>(s: &str) -> Value<'a> {
    let chars: Vec<Value<'a>> = s.chars().map(Value::Char).collect();
    match Array::new(vec![chars.len()], chars) {
        Ok(a) => Value::Array(Rc::new(a)),
        Err(_) => Value::Unit,
    }
}

/// The characters of a text value.
pub(crate) fn chars(v: &Value<'_>) -> Result<String, Diagnostic> {
    as_array(v)
        .data()
        .iter()
        .map(|c| match c {
            Value::Char(c) => Ok(*c),
            other => Err(Diagnostic::new(
                "domain",
                format!("expected text, got {other}"),
            )),
        })
        .collect()
}

/// `f_ormat v`: v as the text it prints as.
pub(crate) fn format<'a>(v: &Value<'a>) -> Value<'a> {
    text(&v.to_string())
}

/// `n_umbers t`: the numbers in t, separated by spaces or newlines.
pub(crate) fn numbers<'a>(t: &Value<'a>) -> Result<Value<'a>, Diagnostic> {
    let s = chars(t)?;
    let parsed: Result<Vec<Value<'a>>, Diagnostic> = s
        .split_whitespace()
        .map(|w| {
            w.parse::<f64>()
                .map(Value::Float)
                .map_err(|_| Diagnostic::new("domain", format!("n_umbers: {w:?} is not a number")))
        })
        .collect();
    let parsed = parsed?;
    let n = parsed.len();
    Array::new(vec![n], parsed)
        .map(to_value)
        .map_err(|e| Diagnostic::new("internal", format!("{e:?}")))
}
