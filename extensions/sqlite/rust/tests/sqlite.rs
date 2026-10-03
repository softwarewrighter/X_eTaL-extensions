//! sqlite called as a host calls it (the loader, linked statically),
//! on databases in a temporary root.
#![allow(unsafe_code)]

use std::sync::OnceLock;

use xetal_ext_loader::{ArrayData, CallError, Registry, Value};

/// One temporary root for the whole test binary (the variable is
/// process-wide), set before any call.
fn sq() -> Registry {
    static ROOT: OnceLock<tempfile::TempDir> = OnceLock::new();
    let dir = ROOT.get_or_init(|| tempfile::tempdir().unwrap());
    // SAFETY: set once, to the same value, before any call reads it.
    unsafe { std::env::set_var("XETAL_SQLITE_ROOT", dir.path()) };
    let mut r = Registry::new();
    r.load_static(xetal_ext_sqlite::__xetal_extension::descriptor)
        .unwrap();
    r
}

fn t(s: &str) -> Value {
    Value::Text(s.into())
}

fn call(r: &Registry, f: &str, db: &str, sql: &str) -> Result<Value, CallError> {
    r.call("sqlite", f, &[t(db), t(sql)])
}

fn chars(v: &Value) -> (Vec<usize>, String) {
    match v {
        Value::Array(a) => match a.data() {
            ArrayData::Char(c) => (a.shape().to_vec(), c.iter().collect()),
            d => panic!("{d:?}"),
        },
        v => panic!("{v:?}"),
    }
}

fn floats(v: &Value) -> (Vec<usize>, Vec<f64>) {
    match v {
        Value::Array(a) => match a.data() {
            ArrayData::Float(x) => (a.shape().to_vec(), x.clone()),
            d => panic!("{d:?}"),
        },
        v => panic!("{v:?}"),
    }
}

#[test]
fn statements_numbers_texts_and_names() {
    let r = sq();
    let db = "t/basic.db";
    let made = call(
        &r,
        "exec",
        db,
        "drop table if exists p; create table p (name text, x real, n integer);
        insert into p values ('ann', 1.5, 3), ('bob', -2.0, NULL), ('cy''s', 0.25, 7);",
    );
    assert_eq!(made, Ok(Value::Int(3)));
    let (shape, x) = floats(&call(&r, "nums", db, "select x, n from p order by rowid").unwrap());
    assert_eq!(shape, [3, 2]);
    assert_eq!(&x[..3], &[1.5, 3.0, -2.0]);
    assert!(x[3].is_nan());
    assert_eq!(&x[4..], &[0.25, 7.0]);
    let (shape, s) = chars(&call(&r, "texts", db, "select name, n from p order by rowid").unwrap());
    assert_eq!(shape, [6, 4]);
    assert_eq!(s, "ann 3   bob     cy's7   ");
    let (shape, s) = chars(&call(&r, "cols", db, "select name, x as value from p").unwrap());
    assert_eq!((shape, s), (vec![2, 5], "name value".into()));
    assert_eq!(
        call(&r, "exec", db, "update p set n = 0 where n is null"),
        Ok(Value::Int(1))
    );
}

#[test]
fn empty_results_and_memory() {
    let r = sq();
    let (shape, x) = floats(&call(&r, "nums", ":memory:", "select 1 + 1, 2.5 * 2").unwrap());
    assert_eq!((shape, x), (vec![1, 2], vec![2.0, 5.0]));
    call(
        &r,
        "exec",
        "t/empty.db",
        "create table if not exists e (a int)",
    )
    .unwrap();
    let (shape, _) = floats(&call(&r, "nums", "t/empty.db", "select a from e").unwrap());
    assert_eq!(shape, [0, 1]);
    let (shape, _) = chars(&call(&r, "texts", "t/empty.db", "select a from e").unwrap());
    assert_eq!(shape, [0, 0]);
}

#[test]
fn quoting() {
    let r = sq();
    let q = |v: Value| r.call("sqlite", "quote", &[v]).unwrap();
    assert_eq!(q(t("O'Brien")), t("'O''Brien'"));
    assert_eq!(q(Value::Int(-4)), t("-4"));
    assert_eq!(q(Value::Float(2.5)), t("2.5"));
    // a quoted value is data, never SQL
    let evil = "x'); drop table p; --";
    let lit = match q(t(evil)) {
        Value::Text(s) => s,
        v => panic!("{v:?}"),
    };
    let (_, s) = chars(&call(&r, "texts", ":memory:", &format!("select {lit}")).unwrap());
    assert_eq!(s, evil);
}

#[test]
fn errors() {
    let r = sq();
    let err = |f: &str, db: &str, sql: &str| match call(&r, f, db, sql) {
        Err(CallError::Failed(m) | CallError::InvalidArgument(m)) => m,
        other => panic!("{other:?}"),
    };
    assert!(err("exec", "/etc/x.db", "select 1").contains("relative"));
    assert!(err("exec", "../x.db", "select 1").contains("may not leave"));
    assert!(err("exec", "a/../../x.db", "select 1").contains("may not leave"));
    assert!(err("exec", ":memory:", "selec 1").contains("syntax error"));
    assert!(err("nums", ":memory:", "select 'abc'").contains("is not a number"));
    assert!(err("nums", ":memory:", "select * from nowhere").contains("no such table"));
}
