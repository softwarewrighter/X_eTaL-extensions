//! Graphics (QD5): []G_RID draws an array as SVG text, []S_HOW hands a
//! picture to the store in use (here one in memory).

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

fn array(shape: Vec<usize>, items: Vec<Value<'static>>) -> Value<'static> {
    Value::Array(Rc::new(Array::new(shape, items).unwrap()))
}

fn ints(shape: Vec<usize>, v: &[i64]) -> Value<'static> {
    array(shape, v.iter().map(|&i| Value::Int(i)).collect())
}

fn run(name: &str, args: &[Value<'static>]) -> Result<String, String> {
    match call(name, args, Span::new(0, 1)) {
        Some(Ok(v)) => Ok(v.to_string()),
        Some(Err(d)) => Err(d.code),
        None => Err("not a system built-in".into()),
    }
}

#[test]
fn grid_gives_the_svg_as_text() {
    let svg = run("[]G_RID", &[ints(vec![2, 2], &[1, 0, 0, 1])]).unwrap();
    assert!(svg.starts_with("<svg xmlns=") && svg.trim_end().ends_with("</svg>"));
    assert_eq!(svg.matches("fill=\"#1f2937\"").count(), 2);
}

#[test]
fn grid_draws_bools_floats_and_chars() {
    let bools = array(vec![2], vec![Value::Bool(true), Value::Bool(false)]);
    assert_eq!(
        run("[]G_RID", &[bools]).unwrap().matches("#1f2937").count(),
        1
    );
    let floats = array(vec![2], vec![Value::Float(0.5), Value::Float(2.5)]);
    assert!(run("[]G_RID", &[floats]).unwrap().contains("#fde725"));
    let chars = array(vec![1], vec![Value::Char('Q')]);
    assert!(run("[]G_RID", &[chars]).unwrap().contains(">Q</text>"));
    assert!(
        run("[]G_RID", &[Value::Int(1)])
            .unwrap()
            .contains("width=\"24\" height=\"24\" viewBox")
    );
}

#[test]
fn grid_rejects_what_it_cannot_draw() {
    assert_eq!(
        run("[]G_RID", &[ints(vec![1, 1, 1, 1], &[1])]).unwrap_err(),
        "rank"
    );
    assert_eq!(run("[]G_RID", &[ints(vec![0], &[])]).unwrap_err(), "empty");
    assert_eq!(run("[]G_RID", &[Value::Unit]).unwrap_err(), "domain");
}

#[test]
fn show_hands_the_picture_to_the_store_and_returns_it() {
    let store = store();
    let svg = "<svg/>";
    let text = array(vec![svg.len()], svg.chars().map(Value::Char).collect());
    assert_eq!(run("[]S_HOW", &[text]).unwrap(), svg);
    assert_eq!(store.pictures(), [svg]);
    assert_eq!(run("[]S_HOW", &[Value::Int(3)]).unwrap_err(), "domain");
}

#[test]
fn path_draws_a_two_row_matrix_of_points() {
    let svg = run("[]P_ATH", &[ints(vec![2, 3], &[0, 1, 2, 0, 1, 0])]).unwrap();
    assert!(svg.contains("<polyline points=\"12,212 212,12 412,212\""));
    assert_eq!(
        run("[]P_ATH", &[ints(vec![3, 2], &[0; 6])]).unwrap_err(),
        "shape-mismatch"
    );
    let chars = array(vec![2, 2], "abcd".chars().map(Value::Char).collect());
    assert_eq!(run("[]P_ATH", &[chars]).unwrap_err(), "domain");
}
