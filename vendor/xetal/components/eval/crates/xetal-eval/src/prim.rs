//! Calling built-ins; which exist, and their arity, come from the
//! catalog (`xetal-catalog`).

use std::io::Write;

use xetal_base::{Diagnostic, Span};
use xetal_catalog::find;

use crate::run::err;
use xetal_arith::{Rng, binary, compare, compare_chars, lift1, lift2, num, truth};
use xetal_value::Value;
use xetal_value::{as_array, to_value};

/// The arity of an implemented built-in, or an error.
pub fn arity(name: &str, span: Span) -> Result<(&'static str, usize), Diagnostic> {
    match find(name) {
        Some(b) if b.implemented => Ok((b.name, b.arity)),
        Some(b) => Err(err(
            "unsupported",
            span,
            format!("the built-in {name} arrives with a later saga ({})", b.rule),
        )),
        None => Err(err(
            "unknown-builtin",
            span,
            format!("there is no built-in {name}"),
        )),
    }
}

/// Call a fully applied built-in.
pub fn call<'a>(
    name: &str,
    args: &[Value<'a>],
    span: Span,
    out: &mut dyn Write,
    rng: &mut Rng,
) -> Result<Value<'a>, Diagnostic> {
    if let Some(result) = xetal_struct::call(name, args, span)
        .or_else(|| xetal_search::call(name, args, span))
        .or_else(|| xetal_radix::call(name, args, span))
        .or_else(|| xetal_rotate::call(name, args, span))
        .or_else(|| xetal_system::call(name, args, span))
    {
        return result;
    }
    match (name, args) {
        ("p_rint!", [v]) => {
            writeln!(out, "{}", xetal_value::printed(v))
                .map_err(|e| err("io", span, e.to_string()))?;
            Ok(v.clone())
        }
        ("r_oll!", [n]) => roll(n, rng, span),
        ("p_i", [Value::Unit]) => Ok(Value::Float(std::f64::consts::PI)),
        ("p_i", [_]) => Err(err("not-unit", span, "p_i takes @ (it is niladic)")),
        ("i_d", [a]) | ("l_eft", [a, _]) | ("r_ight", [_, a]) => Ok(a.clone()),
        (_, [a, b]) => lift2(a, b, span, |x, y| scalar2(name, x, y, span)),
        (_, [a]) => lift1(a, |x| unary(name, x, span)),
        _ => Err(err("unknown-builtin", span, format!("bad call of {name}"))),
    }
}

/// `r_oll! n`: one roll in `1..=k` for every item k of n, in order.
fn roll<'a>(n: &Value<'a>, rng: &mut Rng, span: Span) -> Result<Value<'a>, Diagnostic> {
    let rolled = as_array(n).map(|k| match k {
        Value::Int(k) if *k >= 1 => Ok(Value::Int(rng.roll(*k as u64) as i64)),
        other => Err(err(
            "domain",
            span,
            format!("r_oll! needs a positive count, got {other}"),
        )),
    })?;
    Ok(to_value(rolled))
}

/// A dyadic scalar built-in on two scalars.
fn scalar2<'a>(
    name: &str,
    a: &Value<'a>,
    b: &Value<'a>,
    span: Span,
) -> Result<Value<'a>, Diagnostic> {
    match name {
        "&" | "|" => {
            let (a, b) = (truth(a, span)?, truth(b, span)?);
            Ok(Value::Bool(if name == "&" { a && b } else { a || b }))
        }
        "=" | "!=" | "<" | ">" | "<=" | ">=" | "e_q~" => match (a, b) {
            (Value::Char(x), Value::Char(y)) if name != "e_q~" => Ok(compare_chars(name, *x, *y)),
            (Value::Boxed(_), Value::Boxed(_)) if name == "=" || name == "!=" => {
                Ok(Value::Bool(xetal_search::equal(a, b) == (name == "=")))
            }
            _ => Ok(compare(name, num(a, span)?, num(b, span)?)),
        },
        _ => binary(name, num(a, span)?, num(b, span)?, span),
    }
}

fn unary<'a>(name: &str, a: &Value<'a>, span: Span) -> Result<Value<'a>, Diagnostic> {
    use xetal_arith::Num::{F, I};
    let overflow = || err("integer-overflow", span, "integer overflow");
    let whole = |x: f64| {
        if x.is_finite() && x.abs() < 9.0e18 {
            Ok(Value::Int(x as i64))
        } else {
            Err(overflow())
        }
    };
    if name == "n_ot" {
        return Ok(Value::Bool(!truth(a, span)?));
    }
    match (name, num(a, span)?) {
        ("n_eg", I(i)) => i.checked_neg().map(Value::Int).ok_or_else(overflow),
        ("n_eg", F(x)) => Ok(Value::Float(-x)),
        ("a_bs", I(i)) => i.checked_abs().map(Value::Int).ok_or_else(overflow),
        ("a_bs", F(x)) => Ok(Value::Float(x.abs())),
        ("f_loor", I(i)) | ("c_eiling", I(i)) => Ok(Value::Int(i)),
        ("f_loor", F(x)) => whole(x.floor()),
        ("c_eiling", F(x)) => whole(x.ceil()),
        ("e_xp", n) => Ok(Value::Float(n.f().exp())),
        ("s_in", n) => Ok(Value::Float(n.f().sin())),
        ("c_os", n) => Ok(Value::Float(n.f().cos())),
        ("a_tan", n) => Ok(Value::Float(n.f().atan())),
        ("f_loat", n) => Ok(Value::Float(n.f())),
        ("l_og", n) if n.f() <= 0.0 => Err(err("domain", span, "l_og needs a positive number")),
        ("l_og", n) => Ok(Value::Float(n.f().ln())),
        _ => Err(err("unknown-builtin", span, format!("bad call of {name}"))),
    }
}
