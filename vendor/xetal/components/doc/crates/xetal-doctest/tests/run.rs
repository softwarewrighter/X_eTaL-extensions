//! Running a file's examples (S10): each block a session, outputs and
//! expected errors compared, failures reported with what was printed.

use xetal_doctest::{Outcome, report, run};

fn outcomes(text: &str) -> Vec<Outcome> {
    run("-e", text).expect("runs")
}

const LIB: &str = "\
## Squares, for the doc tests.

## The square of each number.
## >> \"s:\" u_se< \"Stats\"
## >> l:s_q 1 2 3
## 1 4 9
l:s_q := { _r * _r }
";

#[test]
fn an_example_passes_when_it_prints_what_is_shown() {
    let text = "## Twice.\n## >> 2 * 21\n## 42\nl:t_wice := { 2 * _r }\n";
    let all = outcomes(text);
    assert_eq!(all.len(), 1);
    assert!(all[0].passed, "{all:?}");
    assert_eq!(all[0].item.as_deref(), Some("l:t_wice"));
}

#[test]
fn a_wrong_output_fails_and_shows_both() {
    let text = "## Twice.\n## >> 2 * 4\n## 9\nl:t_wice := { 2 * _r }\n";
    let all = outcomes(text);
    assert!(!all[0].passed);
    assert_eq!(
        (all[0].expected.as_str(), all[0].actual.as_str()),
        ("9", "8")
    );
}

#[test]
fn a_block_is_one_session() {
    let text = "## Defs.\n## >> d := 5\n## >> d + 1\n## 6\nl:x := 1\n";
    let all = outcomes(text);
    assert_eq!(all.len(), 2);
    assert!(all.iter().all(|o| o.passed), "{all:?}");
}

#[test]
fn a_library_example_imports_the_library_itself() {
    let text = "## Means.\n## >> \"s:\" u_se< \"Stats\"\n## >> s:m_ean 1 2 3 4\n## 2.5\nl:m := 1\n";
    assert!(outcomes(text).iter().all(|o| o.passed));
}

#[test]
fn library_names_are_not_in_scope_by_themselves() {
    let all = outcomes(LIB);
    assert!(!all[1].passed);
    assert!(all[1].actual.starts_with("error["), "{:?}", all[1]);
}

#[test]
fn an_expected_error_matches_by_its_code() {
    let text = "## Bad.\n## >> 1 + \"a\"\n## error[type-mismatch]\nl:x := 1\n";
    assert!(outcomes(text)[0].passed);
    let unexpected = "## Bad.\n## >> 1 + \"a\"\n## 2\nl:x := 1\n";
    assert!(!outcomes(unexpected)[0].passed);
}

#[test]
fn a_program_example_runs_after_the_program() {
    let text = "#!/usr/bin/env xetal\n## Uses n.\n## >> n + 1\n## 3\nn := 2\np_rint! n\n";
    let all = outcomes(text);
    assert!(all[0].passed, "{all:?}");
}

#[test]
fn the_report_counts_like_cargo_test() {
    let text = "## T.\n## >> 1\n## 1\n## >> 2\n## 3\nl:x := 1\n";
    let (text, ok) = report("t.xtl", &outcomes(text));
    assert!(!ok);
    assert!(
        text.ends_with("doc test result: FAILED. 1 passed; 1 failed\n"),
        "{text}"
    );
    assert!(text.contains("--- expected\n3\n--- actual\n2\n"), "{text}");
}
