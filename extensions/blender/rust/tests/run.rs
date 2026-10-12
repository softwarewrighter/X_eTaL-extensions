//! Running a launcher: the exit status, the output, the timeout, and
//! paths kept under the working directory. A stand-in program (sh)
//! takes Blender's place; Blender itself is run by the goldens, when
//! it is present.

use std::time::Duration;

use xetal_ext_blender::{confine, errors, run_with, tail};

#[test]
fn a_run_reports_its_status_and_output() {
    let dir = std::env::temp_dir();
    let ran = run_with(
        "sh",
        &[
            "-c".into(),
            "echo one; echo two >&2; echo $XETAL_BLENDER; exit 3".into(),
        ],
        &dir,
        &[("XETAL_BLENDER", "/somewhere/Blender".into())],
        Duration::from_secs(20),
    )
    .unwrap();
    assert_eq!(ran.status, Some(3));
    assert!(ran.output.contains("one") && ran.output.contains("two"));
    assert!(ran.output.contains("/somewhere/Blender"));
}

#[test]
fn a_run_that_takes_too_long_is_stopped() {
    let dir = std::env::temp_dir();
    let ran = run_with(
        "sh",
        &["-c".into(), "sleep 30".into()],
        &dir,
        &[],
        Duration::from_millis(300),
    )
    .unwrap();
    assert_eq!(ran.status, None);
}

#[test]
fn paths_stay_under_the_working_directory() {
    assert!(confine("out/scene.py").is_ok());
    assert!(confine("../scene.py").is_err());
    assert!(confine("/tmp/scene.py").is_err());
    assert!(confine("").is_err());
}

#[test]
fn the_tail_is_the_last_lines() {
    assert_eq!(tail("a\nb\nc\nd", 2), "c\nd");
    assert_eq!(tail("a", 5), "a");
}

#[test]
fn error_lines_are_counted() {
    let log =
        "Blender 5.1\nTraceback (most recent call last):\n  x\nError: interpretation failed\nSaved";
    assert_eq!(errors(log), 2);
    assert_eq!(errors("Saved: still.png"), 0);
}

#[test]
fn python_is_the_virtual_environments_when_one_is_active() {
    // XETAL_PYTHON wins; otherwise VIRTUAL_ENV's python3 when it exists
    // (the repository's uv environment), never a guess
    let p = xetal_ext_blender::python();
    match (std::env::var("XETAL_PYTHON"), std::env::var("VIRTUAL_ENV")) {
        (Ok(x), _) if !x.is_empty() => assert_eq!(p, x),
        (_, Ok(v)) if std::path::Path::new(&v).join("bin/python3").is_file() => {
            assert_eq!(p, format!("{v}/bin/python3"));
        }
        _ => assert_eq!(p, "python3"),
    }
}
