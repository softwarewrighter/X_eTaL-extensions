//! Running a launcher: the exit status, the output, the timeout, and
//! paths kept under the working directory. A stand-in program (sh)
//! takes Blender's place; Blender itself is run by the goldens, when
//! it is present.

use std::time::Duration;

use xetal_ext_blender::{confine, run_with, tail};

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
