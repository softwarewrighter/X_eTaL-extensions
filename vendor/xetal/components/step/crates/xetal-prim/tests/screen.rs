//! Screen control (QD6): typed, pure text builders - their results are
//! text holding ANSI sequences, which a program prints and a terminal
//! (or the live demo's grid) interprets - and the built-in enumerated
//! types' constructors and Key's character.

use std::rc::Rc;

use xetal_arith::Rng;
use xetal_array::Array;
use xetal_base::Span;
use xetal_value::Value;

fn text(s: &str) -> Value<'static> {
    Value::Array(Rc::new(Array::vector(s.chars().map(Value::Char).collect())))
}

fn ints(v: &[i64]) -> Value<'static> {
    Value::Array(Rc::new(Array::vector(
        v.iter().map(|i| Value::Int(*i)).collect(),
    )))
}

/// What calling `name` gave back (or its error code); it writes nothing.
fn call(name: &str, args: &[Value<'static>]) -> String {
    let mut out = Vec::new();
    let result = xetal_prim::call(name, args, Span::default(), &mut out, &mut Rng::seeded(1));
    assert!(out.is_empty(), "{name} wrote output");
    match result {
        Ok(v) => v.to_string(),
        Err(e) => format!("error[{}]", e.code),
    }
}

#[test]
fn the_builders_give_text_with_its_codes() {
    assert_eq!(call("[]C_LS", &[Value::Unit]), "\x1b[2J\x1b[H");
    assert_eq!(call("[]A_T", &[ints(&[2, 5]), text("hi")]), "\x1b[2;5Hhi");
    assert_eq!(call("[]B_OLD", &[text("x")]), "\x1b[1mx\x1b[22m");
    let red = Value::Tag("Color", 1);
    assert_eq!(
        call("[]F_G", &[red.clone(), text("x")]),
        "\x1b[31mx\x1b[39m"
    );
    assert_eq!(call("[]B_G", &[red, text("x")]), "\x1b[41mx\x1b[49m");
}

#[test]
fn a_place_is_two_counts_from_one() {
    assert_eq!(
        call("[]A_T", &[ints(&[1, 2, 3]), text("x")]),
        "error[domain]"
    );
    assert_eq!(call("[]A_T", &[ints(&[0, 1]), text("x")]), "error[domain]");
}

#[test]
fn colours_and_keys_are_made_by_their_constructors() {
    assert_eq!(call("[]C_OLOR", &[Value::Int(2)]), "RED");
    assert_eq!(call("[]C_OLOR", &[Value::Int(9)]), "error[domain]");
    assert_eq!(call("[]K_NAMED", &[Value::Int(1)]), "UP");
    assert_eq!(call("[]K_NAMED", &[Value::Int(12)]), "error[domain]");
}

#[test]
fn a_keys_character() {
    assert_eq!(
        call("[]K_CHAR", &[Value::Tag("Key", 0x100 + 'a' as u32)]),
        "a"
    );
    assert_eq!(call("[]K_CHAR", &[Value::Tag("Key", 0)]), "");
}

#[test]
fn colours_and_keys_compare_with_equals() {
    let (red, blue) = (Value::Tag("Color", 1), Value::Tag("Color", 4));
    assert_eq!(call("=", &[red.clone(), red.clone()]), "1");
    assert_eq!(call("=", &[red.clone(), blue.clone()]), "0");
    assert_eq!(call("!=", &[red, blue]), "1");
}
