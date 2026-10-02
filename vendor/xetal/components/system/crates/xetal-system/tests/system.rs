//! The system built-ins on runtime values: numbers as text, and files
//! in the store in use (here one in memory).

use std::rc::Rc;
use std::sync::{Arc, OnceLock};

use xetal_array::Array;
use xetal_base::Span;
use xetal_system::call;
use xetal_value::Value;

/// The store every test here shares: the store is global and tests run
/// at once, so each installing its own would race.
fn store() -> Arc<xetal_store::Memory> {
    static STORE: OnceLock<Arc<xetal_store::Memory>> = OnceLock::new();
    STORE
        .get_or_init(|| {
            let store = Arc::new(xetal_store::Memory::default());
            xetal_store::install(store.clone());
            store
        })
        .clone()
}

fn text(s: &str) -> Value<'static> {
    let chars: Vec<Value<'static>> = s.chars().map(Value::Char).collect();
    Value::Array(Rc::new(Array::new(vec![chars.len()], chars).unwrap()))
}

fn run(name: &str, args: &[Value<'static>]) -> Result<String, String> {
    match call(name, args, Span::new(0, 1)) {
        Some(Ok(v)) => Ok(v.to_string()),
        Some(Err(d)) => Err(d.code),
        None => Err("not a system built-in".into()),
    }
}

#[test]
fn format_gives_the_printed_text_and_numbers_read_it_back() {
    let v = Value::Array(Rc::new(
        Array::new(
            vec![3],
            vec![Value::Int(1), Value::Float(2.5), Value::Int(-3)],
        )
        .unwrap(),
    ));
    assert_eq!(run("f_ormat", &[v]).unwrap(), "1 2.5 -3");
    assert_eq!(
        run("n_umbers", &[text(" 1 2.5\n-3 ")]).unwrap(),
        "1.0 2.5 -3.0"
    );
}

#[test]
fn numbers_rejects_what_is_not_a_number() {
    assert_eq!(run("n_umbers", &[text("1 two")]).unwrap_err(), "domain");
    assert_eq!(run("n_umbers", &[Value::Int(5)]).unwrap_err(), "domain");
}

#[test]
fn files_round_trip_through_the_store_in_use() {
    store();
    let path = text("work/t.txt");
    assert_eq!(run("[]N_PUT", &[text("abc"), path.clone()]).unwrap(), "3");
    assert_eq!(run("[]N_GET", &[path]).unwrap(), "abc");
    assert_eq!(run("[]N_GET", &[text("missing.txt")]).unwrap_err(), "io");
}

#[test]
fn other_names_are_not_system_built_ins() {
    assert_eq!(
        run("r_ev", &[Value::Int(1)]).unwrap_err(),
        "not a system built-in"
    );
}
