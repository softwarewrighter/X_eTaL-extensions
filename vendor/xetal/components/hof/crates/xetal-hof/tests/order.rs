//! Reduce and scan call their operand in the order of the right fold
//! (B6), one call at a time, whether f is associative (a running scan)
//! or not (each prefix folded leftwards).

use std::rc::Rc;

use xetal_array::Array;
use xetal_base::{Diagnostic, Span};
use xetal_kernel::{Never, drive};
use xetal_value::{Prim, Value};

/// Runs `name` with operand `f` on 1 2 3, recording each argument; f
/// fixes its first argument, then gives its second.
fn run(name: &str, f: &str) -> (String, Vec<String>) {
    let f = Value::Prim(Rc::new(Prim {
        name: leak(f),
        arity: 2,
        args: Vec::new(),
    }));
    let x = Value::Array(Rc::new(Array::vector((1..=3).map(Value::Int).collect())));
    let kernel = xetal_hof::call(name, &[f, x], Span::new(0, 0), &mut Never)
        .unwrap()
        .unwrap();
    let mut log = Vec::new();
    let out = drive(kernel, |f, x| -> Result<Value<'_>, Diagnostic> {
        log.push(x.to_string());
        Ok(match &f {
            Value::Prim(p) if p.args.is_empty() => Value::Prim(Rc::new(Prim {
                name: p.name,
                arity: 2,
                args: vec![x],
            })),
            _ => x,
        })
    })
    .unwrap();
    (out.to_string(), log)
}

fn leak(s: &str) -> &'static str {
    Box::leak(s.to_string().into_boxed_str())
}

#[test]
fn reduce_folds_from_the_right() {
    let (out, log) = run("r_/", "g");
    assert_eq!(out, "3");
    assert_eq!(log, ["2", "3", "1", "3"]);
}

#[test]
fn scan_folds_each_prefix_from_the_right() {
    let (out, log) = run("s_\\", "g");
    assert_eq!(out, "1 2 3");
    assert_eq!(log, ["1", "2", "2", "3", "1", "3"]);
}

#[test]
fn associative_scan_runs_on_the_previous_item() {
    let (out, log) = run("s_\\", "+");
    assert_eq!(out, "1 2 3");
    assert_eq!(log, ["1", "2", "2", "3"]);
}
