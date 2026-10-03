//! The CSV reader, and an import through the loader.
#![allow(unsafe_code)]

use std::sync::OnceLock;

use xetal_ext_loader::{ArrayData, CallError, Registry, Value};
use xetal_ext_sqlite::csv::{Kind, kind, records, table_name};

#[test]
fn records_follow_rfc_4180() {
    let r = records("a,b,c\r\n1,\"x, \"\"y\"\"\",\n\"two\nlines\",2,3").unwrap();
    assert_eq!(
        r,
        [
            vec!["a", "b", "c"],
            vec!["1", "x, \"y\"", ""],
            vec!["two\nlines", "2", "3"]
        ]
    );
    assert_eq!(records("a\n1\n").unwrap(), [vec!["a"], vec!["1"]]);
    assert!(records("a\n\"open").is_err());
    assert!(records("a\nx\"y").is_err());
    assert!(records("a\n\"x\"y").is_err());
}

#[test]
fn kinds_and_names() {
    assert_eq!(kind(["1", " 2", ""].into_iter()), Kind::Integer);
    assert_eq!(kind(["1", "2.5", "-3e2"].into_iter()), Kind::Real);
    assert_eq!(kind(["1", "two"].into_iter()), Kind::Text);
    assert_eq!(table_name("data/old-faithful.csv"), "old_faithful");
}

fn sq() -> Registry {
    static ROOT: OnceLock<tempfile::TempDir> = OnceLock::new();
    let dir = ROOT.get_or_init(|| {
        let d = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(d.path().join("data")).unwrap();
        std::fs::write(
            d.path().join("data/old-faithful.csv"),
            "eruption,waiting,note\n3.6,79,\n1.8,54,\"short, \"\"quiet\"\"\"\n3.333,74,\n",
        )
        .unwrap();
        std::fs::write(d.path().join("data/ragged.csv"), "a,b\n1,2\n3\n").unwrap();
        d
    });
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

#[test]
fn import_makes_a_typed_table() {
    let r = sq();
    let call = |f: &str, a: &str, b: &str| r.call("sqlite", f, &[t(a), t(b)]);
    assert_eq!(
        call("import", "csv.db", "data/old-faithful.csv"),
        Ok(Value::Int(3))
    );
    let types = call(
        "texts",
        "csv.db",
        "select type from pragma_table_info('old_faithful') order by cid",
    )
    .unwrap();
    let Value::Array(a) = types else { panic!() };
    let ArrayData::Char(c) = a.data() else {
        panic!()
    };
    assert_eq!(c.iter().collect::<String>(), "REAL   INTEGERTEXT   ");
    let Value::Array(a) = call(
        "nums",
        "csv.db",
        "select sum(waiting), count(note) from old_faithful",
    )
    .unwrap() else {
        panic!()
    };
    assert_eq!(a.data(), &ArrayData::Float(vec![207.0, 1.0]));
    assert_eq!(
        call("import", "csv.db", "geyser=data/old-faithful.csv"),
        Ok(Value::Int(3))
    );

    // a second import of the same table is refused; so is a ragged file
    assert!(
        matches!(call("import", "csv.db", "data/old-faithful.csv"), Err(CallError::Failed(m)) if m.contains("already exists"))
    );
    assert!(
        matches!(call("import", "csv.db", "data/ragged.csv"), Err(CallError::Failed(m)) if m.contains("record 3 has 1 fields"))
    );
    assert!(matches!(
        call("import", "csv.db", "../x.csv"),
        Err(CallError::InvalidArgument(_))
    ));
    assert!(matches!(
        call("import", "csv.db", "data/none.csv"),
        Err(CallError::Failed(_))
    ));
}
