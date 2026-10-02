use xetal_spec::*;

const SQUARE: &str = "\
# squaring via a monadic lambda
== SOURCE
square = { _r * _r }; square_ 7
== TYPE
Int
== RESULT
49
";

fn out(success: bool, stdout: &str, stderr: &str) -> StageOutput {
    StageOutput {
        success,
        stdout: stdout.into(),
        stderr: stderr.into(),
    }
}

#[test]
fn parses_sections_in_file_order_with_preamble() {
    let case = CaseFile::parse(SQUARE).unwrap();
    assert_eq!(case.preamble, vec!["# squaring via a monadic lambda"]);
    assert_eq!(case.source(), "square = { _r * _r }; square_ 7");
    assert_eq!(case.get(Section::Type), Some("Int"));
    assert_eq!(case.get(Section::Result), Some("49"));
    assert_eq!(case.get(Section::Error), None);
    assert_eq!(case.status(), Status::Active);
}

#[test]
fn keeps_multi_line_bodies_and_trims_trailing_blank_lines() {
    let case = CaseFile::parse("== SOURCE\nx\n== RESULT\n1 2\n\n3 4\n\n\n").unwrap();
    assert_eq!(case.get(Section::Result), Some("1 2\n\n3 4"));
}

#[test]
fn accepts_crlf_line_endings() {
    let case = CaseFile::parse("== SOURCE\r\n1 + 2\r\n== RESULT\r\n3\r\n").unwrap();
    assert_eq!(case.source(), "1 + 2");
    assert_eq!(case.get(Section::Result), Some("3"));
}

#[test]
fn status_pending_with_a_note() {
    let case =
        CaseFile::parse("== SOURCE\nx\n== RESULT\n1\n== STATUS\npending  waits for M8\n").unwrap();
    assert_eq!(case.status(), Status::Pending);
}

#[test]
fn rejects_unknown_section() {
    let err = CaseFile::parse("== SOURCE\nx\n== OUTPUT\n1\n").unwrap_err();
    assert_eq!(err.line, 3);
    assert!(err.message.contains("unknown section `OUTPUT`"));
}

#[test]
fn rejects_duplicate_section() {
    let err = CaseFile::parse("== SOURCE\nx\n== RESULT\n1\n== RESULT\n2\n").unwrap_err();
    assert_eq!(err.line, 5);
    assert!(err.message.contains("duplicate section `RESULT`"));
}

#[test]
fn rejects_missing_or_empty_source() {
    assert!(
        CaseFile::parse("== RESULT\n1\n")
            .unwrap_err()
            .message
            .contains("missing `== SOURCE`")
    );
    assert!(
        CaseFile::parse("== SOURCE\n\n== RESULT\n1\n")
            .unwrap_err()
            .message
            .contains("empty `== SOURCE`")
    );
}

#[test]
fn rejects_text_before_first_header() {
    let err = CaseFile::parse("stray\n== SOURCE\nx\n").unwrap_err();
    assert_eq!(err.line, 1);
}

#[test]
fn rejects_result_with_error() {
    let err = CaseFile::parse("== SOURCE\nx\n== RESULT\n1\n== ERROR\nboom\n").unwrap_err();
    assert!(err.message.contains("mutually exclusive"));
}

#[test]
fn rejects_unknown_status() {
    let err = CaseFile::parse("== SOURCE\nx\n== STATUS\nmaybe\n").unwrap_err();
    assert!(err.message.contains("`active` or `pending`"));
}

#[test]
fn render_round_trips() {
    let case = CaseFile::parse(SQUARE).unwrap();
    assert_eq!(case.render(), SQUARE);
    assert_eq!(CaseFile::parse(&case.render()).unwrap(), case);
}

#[test]
fn set_replaces_or_inserts_in_canonical_order() {
    let mut case = CaseFile::parse(SQUARE).unwrap();
    case.set(Section::Tokens, "Noun(square)".into());
    case.set(Section::Result, "50".into());
    let order: Vec<Section> = case.sections.iter().map(|(s, _)| *s).collect();
    assert_eq!(
        order,
        vec![
            Section::Source,
            Section::Tokens,
            Section::Type,
            Section::Result
        ]
    );
    assert_eq!(case.get(Section::Result), Some("50"));
}

#[test]
fn sections_map_to_cli_stages() {
    assert_eq!(Section::Tokens.stage(), Some("lex"));
    assert_eq!(Section::Render.stage(), Some("render"));
    assert_eq!(Section::Surface.stage(), Some("parse"));
    assert_eq!(Section::Canonical.stage(), Some("fmt"));
    assert_eq!(Section::Core.stage(), Some("core"));
    assert_eq!(Section::Type.stage(), Some("type"));
    assert_eq!(Section::Result.stage(), Some("eval"));
    assert_eq!(Section::Error.stage(), Some("eval"));
    assert_eq!(Section::Source.stage(), None);
    assert_eq!(Section::Status.stage(), None);
}

#[test]
fn check_passes_matching_stdout_and_runs_each_stage() {
    let case = CaseFile::parse(SQUARE).unwrap();
    let mut stages = Vec::new();
    let checks = check_case(&case, |stage, src| {
        stages.push(stage.to_string());
        assert_eq!(src, case.source());
        match stage {
            "type" => out(true, "Int\n", ""),
            _ => out(true, "49\n", ""),
        }
    });
    assert_eq!(stages, vec!["type", "eval"]);
    assert!(checks.iter().all(|c| c.outcome == Outcome::Pass));
    assert_eq!(verdict(Status::Active, &checks), Ok(()));
}

#[test]
fn error_section_compares_stderr_of_a_failing_run() {
    let case =
        CaseFile::parse("== SOURCE\n1 / 0\n== ERROR\nerror[div]: division by zero\n").unwrap();
    let pass = check_case(&case, |_, _| {
        out(false, "", "error[div]: division by zero\n")
    });
    assert_eq!(pass[0].outcome, Outcome::Pass);
    let succeeded = check_case(&case, |_, _| out(true, "0\n", ""));
    assert!(matches!(
        succeeded[0].outcome,
        Outcome::Mismatch {
            blessable: false,
            ..
        }
    ));
}

#[test]
fn unsupported_stage_is_reported_as_such() {
    let case = CaseFile::parse(SQUARE).unwrap();
    let checks = check_case(&case, |_, _| {
        out(
            false,
            "",
            "error[unsupported]: stage `eval` is not implemented\n",
        )
    });
    assert!(checks.iter().all(|c| c.outcome == Outcome::Unsupported));
    let err = verdict(Status::Active, &checks).unwrap_err();
    assert!(err.contains("unsupported"), "{err}");
}

#[test]
fn active_mismatch_fails_with_expected_and_actual() {
    let case = CaseFile::parse("== SOURCE\n1 + 2\n== RESULT\n3\n").unwrap();
    let checks = check_case(&case, |_, _| out(true, "4\n", ""));
    let err = verdict(Status::Active, &checks).unwrap_err();
    assert!(err.contains("--- expected\n3\n--- actual\n4"), "{err}");
}

#[test]
fn pending_must_fail_and_unexpected_pass_is_an_error() {
    let case = CaseFile::parse("== SOURCE\n1 + 2\n== RESULT\n3\n== STATUS\npending\n").unwrap();
    let failing = check_case(&case, |_, _| out(true, "4\n", ""));
    assert_eq!(verdict(Status::Pending, &failing), Ok(()));
    let passing = check_case(&case, |_, _| out(true, "3\n", ""));
    let err = verdict(Status::Pending, &passing).unwrap_err();
    assert!(err.contains("unexpectedly passes"), "{err}");
}

#[test]
fn a_case_that_checks_nothing_is_an_error() {
    let case = CaseFile::parse("== SOURCE\n1 + 2\n").unwrap();
    let checks = check_case(&case, |_, _| unreachable!("no stage sections"));
    assert!(verdict(Status::Active, &checks).is_err());
    assert!(verdict(Status::Pending, &checks).is_err());
}

#[test]
fn bless_rewrites_active_mismatches_only() {
    let mut case = CaseFile::parse("== SOURCE\n1 + 2\n== RESULT\n0\n").unwrap();
    let checks = check_case(&case, |_, _| out(true, "3\n", ""));
    assert!(bless(&mut case, &checks));
    assert_eq!(case.get(Section::Result), Some("3"));

    let mut pending =
        CaseFile::parse("== SOURCE\n1 + 2\n== RESULT\n0\n== STATUS\npending\n").unwrap();
    let checks = check_case(&pending, |_, _| out(true, "3\n", ""));
    assert!(!bless(&mut pending, &checks));
    assert_eq!(pending.get(Section::Result), Some("0"));
}

#[test]
fn bless_never_writes_unsupported_output() {
    let mut case = CaseFile::parse("== SOURCE\n1 + 2\n== RESULT\n3\n").unwrap();
    let checks = check_case(&case, |_, _| {
        out(
            false,
            "",
            "error[unsupported]: stage `eval` is not implemented\n",
        )
    });
    assert!(!bless(&mut case, &checks));
    assert_eq!(case.get(Section::Result), Some("3"));
}

#[test]
fn render_section_sorts_after_tokens() {
    let mut case = CaseFile::parse("== SOURCE\nr_\n== TYPE\nInt\n").unwrap();
    case.set(Section::Render, "r".into());
    case.set(Section::Tokens, "Func(r)".into());
    let order: Vec<Section> = case.sections.iter().map(|(s, _)| *s).collect();
    assert_eq!(
        order,
        vec![
            Section::Source,
            Section::Tokens,
            Section::Render,
            Section::Type
        ]
    );
}
