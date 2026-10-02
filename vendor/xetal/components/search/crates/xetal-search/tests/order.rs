//! Sorting through the public dispatch: stable, exact across Int and
//! Float, and lexicographic on rows.

use std::rc::Rc;

use xetal_array::Array;
use xetal_base::Span;
use xetal_search::call;
use xetal_value::Value;

fn run(name: &str, v: Value<'static>) -> String {
    call(name, &[v], Span::default())
        .unwrap()
        .unwrap()
        .to_string()
}

fn array(shape: Vec<usize>, items: Vec<Value<'static>>) -> Value<'static> {
    Value::Array(Rc::new(Array::new(shape, items).unwrap()))
}

#[test]
fn grade_is_stable() {
    let v = array(vec![4], [2, 1, 2, 1].map(Value::Int).to_vec());
    assert_eq!(run("g_rade", v), "2 4 1 3");
}

#[test]
fn int_and_float_order_exactly() {
    let v = array(
        vec![3],
        vec![Value::Float(2.5), Value::Int(2), Value::Int(3)],
    );
    assert_eq!(run("s_ort", v), "2 2.5 3");
}

#[test]
fn rows_order_item_by_item() {
    let v = array(vec![3, 2], [2, 1, 1, 9, 1, 3].map(Value::Int).to_vec());
    assert_eq!(run("g_rade", v), "3 2 1");
}

#[test]
fn other_names_are_not_search_builtins() {
    assert!(call("r_/", &[Value::Int(1)], Span::default()).is_none());
}
