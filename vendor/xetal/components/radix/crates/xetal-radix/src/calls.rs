//! Dispatch: encode and decode on runtime values, with APL's shapes.

use xetal_array::Array;
use xetal_base::{Diagnostic, Span};
use xetal_value::{Value, as_array, to_value};

use crate::digits::{decode, encode};
use crate::number::{Number, is_float};

type Out<'a> = Result<Value<'a>, Diagnostic>;

/// Call `e_ncode` or `d_ecode`, if `name` is one.
pub fn call<'a>(name: &str, args: &[Value<'a>], span: Span) -> Option<Out<'a>> {
    let result = match (name, args) {
        ("e_ncode", [r, x]) => encode_all(r, x),
        ("d_ecode", [r, x]) if is_float(r) || is_float(x) => decode_all::<f64>(r, x),
        ("d_ecode", [r, x]) => decode_all::<i64>(r, x),
        _ => return None,
    };
    Some(result.map_err(|d| match d.span {
        Some(_) => d,
        None => d.with_span(span),
    }))
}

/// Numbers of rank at most `most`, for `name`'s argument.
fn numbers<N: Number>(v: &Value<'_>, name: &str, most: usize) -> Result<Array<N>, Diagnostic> {
    let a = as_array(v).map(|x| {
        N::read(x).ok_or_else(|| {
            let message = format!("expected {}, got {x}", N::WHAT);
            Diagnostic::new("not-an-integer", message)
        })
    })?;
    if a.rank() > most {
        let dims: Vec<String> = a.shape().iter().map(ToString::to_string).collect();
        let what = if most == 1 {
            "a scalar or a vector"
        } else {
            "at most a matrix"
        };
        let message = format!("{name} needs {what}, got shape {}", dims.join(" "));
        return Err(Diagnostic::new("rank", message));
    }
    Ok(a)
}

/// Shape: the radix's, then the right argument's; the digits of item
/// `j` run down column `j`.
fn encode_all<'a>(r: &Value<'a>, x: &Value<'a>) -> Out<'a> {
    let (r, x) = (
        numbers::<i64>(r, "e_ncode", 1)?,
        numbers::<i64>(x, "e_ncode", 1)?,
    );
    let columns = x
        .data()
        .iter()
        .map(|n| encode(r.data(), *n))
        .collect::<Result<Vec<_>, _>>()?;
    let data = (0..r.data().len())
        .flat_map(|i| columns.iter().map(move |c| Value::Int(c[i])))
        .collect();
    Ok(to_value(Array::new([r.shape(), x.shape()].concat(), data)?))
}

/// One number per column of the digits (a vector is one column); a
/// scalar radix extends to every digit.
fn decode_all<'a, N: Number>(r: &Value<'a>, x: &Value<'a>) -> Out<'a> {
    let (r, x) = (
        numbers::<N>(r, "d_ecode", 1)?,
        numbers::<N>(x, "d_ecode", 2)?,
    );
    let k = x.shape().first().copied().unwrap_or(1);
    let radix = match (r.rank(), r.data()) {
        (0, [b]) => vec![*b; k],
        (_, rs) if rs.len() == k => rs.to_vec(),
        (_, rs) => {
            let message = format!("{} radix values for {k} digits", rs.len());
            return Err(Diagnostic::new("length-mismatch", message));
        }
    };
    let cols = x.shape().get(1).copied().unwrap_or(1);
    let numbers = (0..cols)
        .map(|j| {
            let digits: Vec<N> = (0..k).map(|i| x.data()[i * cols + j]).collect();
            decode(&radix, &digits).map(N::value)
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(match x.rank() {
        2 => to_value(Array::vector(numbers)),
        _ => numbers[0].clone(),
    })
}
