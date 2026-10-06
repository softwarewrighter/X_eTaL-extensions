//! Known vectors, and files read in pieces.
#![allow(unsafe_code)]

use xetal_ext_loader::{Registry, Value};

fn call(f: &str, s: &str) -> Value {
    let mut r = Registry::new();
    r.load_static(xetal_ext_digest::__xetal_extension::descriptor)
        .unwrap();
    r.call("digest", f, &[Value::Text(s.into())]).unwrap()
}

fn text(v: Value) -> String {
    match v {
        Value::Text(s) => s,
        v => panic!("{v:?}"),
    }
}

#[test]
fn known_vectors() {
    assert_eq!(
        text(call("sha256", "")),
        "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
    );
    assert_eq!(
        text(call("sha256", "abc")),
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
    assert_eq!(call("crc32", "123456789"), Value::Int(0xCBF4_3926));
    assert_eq!(call("crc32", ""), Value::Int(0));
    // text is hashed as UTF-8: e with an acute accent is c3 a9
    assert_eq!(
        text(call("sha256", "\u{e9}")),
        "4a99557e4033c3539de2eb65472017cad5f9557f7a0625a09f1c3f6e2ba69c4c"
    );
    assert_eq!(call("crc32", "\u{e9}"), Value::Int(235_179_326));
}

#[test]
fn files_match_their_text_and_are_confined() {
    let dir = tempfile::tempdir().unwrap();
    // SAFETY: the only test in this binary that reads the variable.
    unsafe { std::env::set_var("XETAL_DIGEST_ROOT", dir.path()) };
    // bigger than one piece
    let body: String = (0..200_000)
        .map(|i| char::from(b'a' + (i % 26) as u8))
        .collect();
    std::fs::write(dir.path().join("big.txt"), &body).unwrap();
    assert_eq!(call("sha256_file", "big.txt"), call("sha256", &body));
    assert_eq!(call("crc32_file", "big.txt"), call("crc32", &body));
    let mut r = Registry::new();
    r.load_static(xetal_ext_digest::__xetal_extension::descriptor)
        .unwrap();
    for bad in ["../x", "/etc/hosts", ""] {
        let e = r
            .call("digest", "sha256_file", &[Value::Text(bad.into())])
            .unwrap_err()
            .to_string();
        assert!(e.contains("no .."), "{bad}: {e}");
    }
    let e = r
        .call("digest", "crc32_file", &[Value::Text("missing.txt".into())])
        .unwrap_err()
        .to_string();
    assert!(e.contains("missing.txt"), "{e}");
}
