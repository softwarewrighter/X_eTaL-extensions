//! A REPL session: definitions and types persist across lines, errors
//! do not end the session, unclosed brackets continue on the next line.

use xetal_repl::{Reply, Session};

fn say(s: &mut Session, line: &str) -> (String, String) {
    match s.feed(line) {
        Reply::Done { out, err } => (out, err),
        Reply::More => panic!("{line:?} should complete"),
    }
}

#[test]
fn definitions_persist_across_lines() {
    let mut s = Session::default();
    assert_eq!(
        say(&mut s, "u:s_quare := { _r * _r }"),
        (String::new(), String::new())
    );
    assert_eq!(say(&mut s, "u:s_quare 7").0, "49\n");
    assert_eq!(say(&mut s, "x := 3").0, "");
    assert_eq!(say(&mut s, "u:s_quare x").0, "9\n");
}

#[test]
fn an_error_is_reported_and_the_line_is_dropped() {
    let mut s = Session::default();
    say(&mut s, "x := 3");
    let (out, err) = say(&mut s, "x + 2.5");
    assert_eq!(out, "");
    assert!(err.starts_with("error[type-mismatch]"), "{err}");
    assert!(
        err.ends_with("at 0..7\n"),
        "spans are relative to the line: {err}"
    );
    let (out, err) = say(&mut s, "p_rint! 5; 1 / 0");
    assert_eq!(out, "5\n5\n");
    assert!(err.starts_with("error[division-by-zero]"), "{err}");
    assert_eq!(say(&mut s, "x * 10").0, "30\n");
}

#[test]
fn effects_of_earlier_lines_are_not_repeated() {
    let mut s = Session::default();
    assert_eq!(say(&mut s, "p_rint! 1").0, "1\n1\n");
    assert_eq!(say(&mut s, "2").0, "2\n");
}

#[test]
fn an_unclosed_bracket_continues() {
    let mut s = Session::default();
    assert_eq!(s.feed("u:f_ := { n ->"), Reply::More);
    assert_eq!(s.feed("  n + 1"), Reply::More);
    assert_eq!(say(&mut s, "}"), (String::new(), String::new()));
    assert_eq!(say(&mut s, "u:f_ 41").0, "42\n");
}

#[test]
fn a_rolled_value_keeps_its_value_across_lines() {
    let mut s = Session::default();
    assert_eq!(
        say(&mut s, "x := r_oll! 1000000000"),
        (String::new(), String::new())
    );
    let first = say(&mut s, "x");
    assert_eq!(say(&mut s, "x"), first);
    assert_eq!(say(&mut s, "x = x"), ("1\n".to_string(), String::new()));
}

#[test]
fn each_new_roll_is_fresh() {
    let mut s = Session::default();
    let rolls: Vec<(String, String)> = (0..5).map(|_| say(&mut s, "r_oll! 1000000000")).collect();
    assert!(
        rolls.iter().any(|r| r != &rolls[0]),
        "five equal rolls: {rolls:?}"
    );
}

#[test]
fn a_session_can_import_a_library() {
    let mut s = Session::default();
    assert_eq!(
        say(&mut s, "\"s:\" u_se< \"Stats\""),
        (String::new(), String::new())
    );
    assert_eq!(
        say(&mut s, "s:m_ean 1 2 3"),
        ("2.0\n".to_string(), String::new())
    );
    let (_, err) = say(&mut s, "s:m_ean y");
    assert!(
        err.contains("at 8..9"),
        "the error is placed in the line typed: {err}"
    );
}

#[test]
fn output_can_stream_to_a_sink_as_it_is_written() {
    let mut s = xetal_repl::Session::new("-e", 1);
    let mut sink = Vec::new();
    s.feed_to("1 + 1", &mut sink);
    s.feed_to("p_rint! 5 6", &mut sink);
    s.feed_to("1 / y", &mut sink);
    s.feed_to("7", &mut sink);
    assert_eq!(
        String::from_utf8(sink).unwrap(),
        "2\n5 6\n5 6\n7\n",
        "each line's new output once; a failing line adds none"
    );
}
