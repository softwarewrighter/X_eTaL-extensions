//! xetal-x as a program: the vendored CLI's behavior, its own options,
//! and ext: paths reaching the extension store.

use std::path::{Path, PathBuf};
use std::process::Command;

fn xx() -> Command {
    let mut c = Command::new(env!("CARGO_BIN_EXE_xetal-x"));
    c.env_remove("XETAL_EXT_PATH").env_remove("XETAL_PATH");
    c
}

fn extensions() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../extensions")
}

fn run(c: &mut Command) -> (String, String, bool) {
    let out = c.output().unwrap();
    (
        String::from_utf8(out.stdout).unwrap(),
        String::from_utf8(out.stderr).unwrap(),
        out.status.success(),
    )
}

#[test]
fn evaluates_as_xetal_does() {
    let (out, _, ok) = run(xx().args(["eval", "-e", "'+ r_/_2 2 3 r_eshape r_ange 6"]));
    assert!(ok);
    assert_eq!(out, "6 15\n");
}

#[test]
fn errors_as_xetal_does() {
    let (_, err, ok) = run(xx().args(["eval", "-e", "1 +"]));
    assert!(!ok);
    assert!(err.starts_with("error["), "{err}");
}

#[test]
fn lists_extensions() {
    let (out, _, ok) = run(xx().arg("--ext").arg(extensions()).arg("--ext-list"));
    assert!(ok);
    assert!(
        out.contains("\nhello 0.1.0\n") || out.starts_with("hello 0.1.0\n"),
        "{out}"
    );
    assert!(out.contains("  hello/add : Num a => a -> a -> a    # Left plus right"));
    let (out, _, _) = run(xx()
        .env("XETAL_EXT_PATH", extensions().join("hello"))
        .arg("--ext-list"));
    assert!(out.starts_with("hello 0.1.0\n"), "{out}");
}

#[test]
fn ext_paths_reach_the_store() {
    let program = r#""hi" []N_PUT "ext:hello/nope""#;
    let (_, err, ok) = run(xx()
        .arg("--ext")
        .arg(extensions())
        .args(["eval", "-e", program]));
    assert!(!ok);
    assert!(
        err.contains("ext:hello/nope: extension hello has no function nope"),
        "{err}"
    );
    let (_, err, _) = run(xx().args(["eval", "-e", program]));
    assert!(err.contains("no extension hello is loaded"), "{err}");
    let (_, err, _) = run(xx().args(["eval", "-e", r#"[]N_GET "ext:hello""#]));
    assert!(err.contains("expected ext:EXTENSION/FUNCTION"), "{err}");
}

#[test]
fn a_bad_ext_dir_is_reported() {
    let (_, err, ok) = run(xx().args(["--ext", "/no/such/dir", "eval", "-e", "1"]));
    assert!(!ok);
    assert!(err.starts_with("error[ext]"), "{err}");
    let (_, err, ok) = run(xx().arg("--ext"));
    assert!(!ok);
    assert!(err.contains("--ext needs a directory"), "{err}");
}

#[test]
fn facades_are_on_the_library_path() {
    let program = "\"hx:\" u_se< \"Hello\"\nhx:s_hout \"x_etal\"";
    let (out, err, ok) = run(xx()
        .arg("--ext")
        .arg(extensions())
        .args(["eval", "-e", program]));
    assert!(ok, "{err}");
    assert_eq!(out, "X_ETAL\n");
}
