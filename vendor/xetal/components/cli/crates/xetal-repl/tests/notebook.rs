//! A file as a notebook: each statement with the output it produced.

use xetal_repl::{Cell, notebook};

fn cells(src: &str) -> Vec<(String, String, String)> {
    notebook("-e", src, 7, false)
        .into_iter()
        .map(|Cell { source, out, err }| (source, out, err))
        .collect()
}

#[test]
fn each_line_is_followed_by_its_own_output() {
    let got = cells("# squares\nu:s_quare := { _r * _r }\nu:s_quare 3; u:s_quare 4");
    assert_eq!(got[0], ("# squares".into(), String::new(), String::new()));
    assert_eq!(got[1].1, "");
    assert_eq!(
        got[2],
        (
            "u:s_quare 3; u:s_quare 4".into(),
            "9\n16\n".into(),
            String::new()
        )
    );
}

#[test]
fn a_statement_over_several_lines_stays_together() {
    let got = cells("u:f_ := { n ->\n  n + 1\n}\nu:f_ 41");
    assert_eq!(got[0].0, "u:f_ := { n ->\n  n + 1\n}");
    assert_eq!(got[1].1, "42\n");
}

#[test]
fn an_error_shows_under_its_line_and_the_rest_runs() {
    let got = cells("1 / 0\n2 + 3");
    assert!(got[0].2.contains("division-by-zero"), "{got:?}");
    assert_eq!(got[1].1, "5\n");
}

#[test]
fn printing_and_rolling_are_not_repeated() {
    let got = cells("x := r_oll! 1000000\np_rint! 5\nx = x");
    assert_eq!(got[1].1, "5\n5\n");
    assert_eq!(got[2].1, "1\n");
}

#[test]
fn a_block_continues_after_earlier_ones_showing_only_its_own_output() {
    let context = "u:s_q := { _r * _r }\nu:s_q 3\np_rint! 1\n";
    let got = xetal_repl::continued("-e", context, "u:s_q 4\nu:s_q 5\n", 7);
    assert_eq!((got.out.as_str(), got.err.as_str()), ("16\n25\n", ""));
}

#[test]
fn a_failing_block_reports_its_error() {
    let got = xetal_repl::continued("-e", "x := 1\n", "x + y\n", 7);
    assert!(got.err.contains("error["), "{got:?}");
}

#[test]
fn a_notebook_can_import_a_library() {
    let got = cells("\"s:\" u_se< \"Stats\"\ns:r_ange 3 9 4");
    assert_eq!(got[1].1, "6\n");
}

#[test]
fn an_untyped_notebook_skips_the_checker() {
    let src = include_str!("../../../../../demos/fixed-point.xtl");
    let typed = notebook("-e", src, 7, false);
    assert!(
        typed
            .iter()
            .any(|c| c.err.starts_with("error[infinite-type]"))
    );
    let untyped = notebook("-e", src, 7, true);
    assert!(untyped.iter().all(|c| c.err.is_empty()), "{untyped:?}");
    assert!(untyped.iter().any(|c| c.out == "120\n"), "{untyped:?}");
}
