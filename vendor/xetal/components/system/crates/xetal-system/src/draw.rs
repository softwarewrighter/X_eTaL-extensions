//! Graphics (QD5): `[]G_RID` draws an array as a grid and `[]P_ATH`
//! points as a path, each an SVG document returned as text, and
//! `[]S_HOW` hands a picture to the host. Drawing is pure;
//! showing is the one effect.

use xetal_base::Diagnostic;
use xetal_draw::Cells;
use xetal_value::{Value, as_array};

use crate::text::{chars, text};

/// `[]G_RID a`: a scalar, vector or matrix as a grid of cells, a rank-3
/// array as frames shown in turn; the SVG as a Char vector.
pub(crate) fn grid<'a>(v: &Value<'a>) -> Result<Value<'a>, Diagnostic> {
    let a = as_array(v);
    let cells = cells(a.data())?;
    xetal_draw::grid(a.shape(), &cells)
        .map(|svg| text(&svg))
        .map_err(|e| Diagnostic::new(e.code(), format!("[]G_RID: {}", e.message())))
}

/// `[]P_ATH xy`: points, 2 rows (x over y), joined in order and fitted
/// into the picture; a rank-3 array is frames of such paths.
pub(crate) fn path<'a>(v: &Value<'a>) -> Result<Value<'a>, Diagnostic> {
    let a = as_array(v);
    let coords: Vec<f64> = a.data().iter().map(number_of).collect::<Result<_, _>>()?;
    xetal_draw::path(a.shape(), &coords)
        .map(|svg| text(&svg))
        .map_err(|e| Diagnostic::new(e.code(), format!("[]P_ATH: {}", e.message())))
}

/// `[]S_HOW svg`: show the picture (numbered files on the command line,
/// the Draw pane in the browser) and return it.
pub(crate) fn show<'a>(v: &Value<'a>) -> Result<Value<'a>, Diagnostic> {
    let svg = chars(v)?;
    xetal_store::show(&svg).map_err(|e| Diagnostic::new("io", format!("[]S_HOW: {e}")))?;
    Ok(v.clone())
}

/// The items as numbers (Bool and Int too) or as characters.
fn cells(items: &[Value<'_>]) -> Result<Cells, Diagnostic> {
    if let Some(Value::Char(_)) = items.first() {
        return items
            .iter()
            .map(char_of)
            .collect::<Result<_, _>>()
            .map(Cells::Chars);
    }
    items
        .iter()
        .map(number_of)
        .collect::<Result<_, _>>()
        .map(Cells::Numbers)
}

fn number_of(v: &Value<'_>) -> Result<f64, Diagnostic> {
    match v {
        Value::Int(i) => Ok(*i as f64),
        Value::Float(x) => Ok(*x),
        Value::Bool(b) => Ok(f64::from(u8::from(*b))),
        other => Err(undrawable(other)),
    }
}

fn char_of(v: &Value<'_>) -> Result<char, Diagnostic> {
    match v {
        Value::Char(c) => Ok(*c),
        other => Err(undrawable(other)),
    }
}

fn undrawable(v: &Value<'_>) -> Diagnostic {
    Diagnostic::new(
        "domain",
        format!("[]G_RID draws numbers or characters, not {v}"),
    )
}
