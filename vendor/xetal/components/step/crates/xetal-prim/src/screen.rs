//! Screen control (QD6): typed, pure text builders whose results hold
//! ANSI sequences (a program prints them; a terminal or the live demo's
//! grid interprets them), the built-in enumerated types' constructors
//! for libraries (`[]C_OLOR`, `[]K_NAMED`), and a key's character.

use std::rc::Rc;

use xetal_array::Array;
use xetal_base::{Diagnostic, Span};
use xetal_value::{COLORS, KEYS, Value, as_array};

type Out<'a> = Result<Value<'a>, Diagnostic>;

/// The screen built-in `name`, if it is one.
pub(crate) fn call<'a>(name: &str, args: &[Value<'a>], span: Span) -> Option<Out<'a>> {
    let wrap = |on: String, t: &Value<'a>, off: &str| Ok(text(&format!("{on}{}{off}", chars(t))));
    Some(match (name, args) {
        ("[]C_LS", [_]) => Ok(text("\x1b[2J\x1b[H")),
        ("[]B_OLD", [t]) => wrap("\x1b[1m".into(), t, "\x1b[22m"),
        ("[]F_G", [Value::Tag(_, c), t]) => wrap(format!("\x1b[{}m", 30 + c), t, "\x1b[39m"),
        ("[]B_G", [Value::Tag(_, c), t]) => wrap(format!("\x1b[{}m", 40 + c), t, "\x1b[49m"),
        ("[]A_T", [rc, t]) => {
            place(rc, span).and_then(|(r, c)| wrap(format!("\x1b[{r};{c}H"), t, ""))
        }
        ("[]C_OLOR", [n]) => tag("Color", COLORS.len(), n, span),
        ("[]K_NAMED", [n]) => tag("Key", KEYS.len(), n, span),
        ("[]K_CHAR", [Value::Tag(_, k)]) => Ok(text(&key_char(*k))),
        _ => return None,
    })
}

/// A key's character, or nothing for a named key.
fn key_char(k: u32) -> String {
    match (k as usize) < KEYS.len() {
        true => String::new(),
        false => xetal_value::tag_name("Key", k),
    }
}

fn text<'a>(s: &str) -> Value<'a> {
    Value::Array(Rc::new(Array::vector(s.chars().map(Value::Char).collect())))
}

fn chars(v: &Value<'_>) -> String {
    let char_of = |c: &Value<'_>| match c {
        Value::Char(c) => *c,
        _ => '?',
    };
    as_array(v).data().iter().map(char_of).collect()
}

/// A row and a column, both 1 or more.
fn place(rc: &Value<'_>, span: Span) -> Result<(i64, i64), Diagnostic> {
    match as_array(rc).data() {
        [Value::Int(r), Value::Int(c)] if *r >= 1 && *c >= 1 => Ok((*r, *c)),
        _ => Err(domain(
            span,
            "[]A_T needs a row and a column, each 1 or more",
        )),
    }
}

/// Value n (1-origin) of the enumerated type `ty` with `count` values.
fn tag<'a>(ty: &'static str, count: usize, n: &Value<'a>, span: Span) -> Out<'a> {
    match n {
        Value::Int(i) if *i >= 1 && (*i as usize) <= count => Ok(Value::Tag(ty, (*i - 1) as u32)),
        other => Err(domain(
            span,
            &format!("{ty} has values 1 to {count}, not {other}"),
        )),
    }
}

fn domain(span: Span, message: &str) -> Diagnostic {
    Diagnostic::new("domain", message).with_span(span)
}
