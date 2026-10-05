//! The report, as `cargo test` prints one: a line per example, each
//! failure with what was shown and what was printed, then the counts.

use crate::Outcome;

/// The report for the examples of `file`, and whether all passed.
pub fn report(file: &str, outcomes: &[Outcome]) -> (String, bool) {
    let noun = if outcomes.len() == 1 {
        "example"
    } else {
        "examples"
    };
    let mut text = format!("doc tests in {file}: {} {noun}\n", outcomes.len());
    for o in outcomes {
        let verdict = if o.passed { "ok" } else { "FAILED" };
        text += &format!("{} >> {} ... {verdict}\n", place(o), o.code);
    }
    for o in outcomes.iter().filter(|o| !o.passed) {
        text += &format!("\n---- {} >> {}\n", place(o), o.code);
        text += &format!("--- expected\n{}\n--- actual\n{}\n", o.expected, o.actual);
    }
    let failed = outcomes.iter().filter(|o| !o.passed).count();
    let verdict = if failed == 0 { "ok" } else { "FAILED" };
    let passed = outcomes.len() - failed;
    text += &format!("doc test result: {verdict}. {passed} passed; {failed} failed\n");
    (text, failed == 0)
}

/// Where an example is: its item, or the file's own doc.
fn place(o: &Outcome) -> &str {
    o.item.as_deref().unwrap_or("(file)")
}
