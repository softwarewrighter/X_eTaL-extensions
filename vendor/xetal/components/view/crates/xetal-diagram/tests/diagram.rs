//! Annotated diagrams: callouts anchored to real tokens, or an error.

use xetal_diagram::diagram;

const NOTES: &str = "\
== SOURCE
u:s_q := { _r * _r }
== TITLE
Squares
== NOTE u:s_q
A name
The program's own function.
== NOTE _r #2
The argument
Its second use.
";

#[test]
fn the_decorated_line_and_every_note_are_drawn() {
    let svg = diagram(NOTES).unwrap();
    assert!(svg.starts_with("<svg"), "{svg}");
    for text in [
        "Squares",
        "A name",
        "The argument",
        "Its second use.",
        "\u{2375}",
    ] {
        assert!(svg.contains(text), "missing {text}");
    }
    assert!(svg.contains("u:s_q := { _r * _r }"), "the source as typed");
    assert_eq!(diagram(NOTES).unwrap(), svg, "deterministic");
}

#[test]
fn an_anchor_must_be_in_the_source() {
    let bad = NOTES.replace("== NOTE u:s_q", "== NOTE u:c_ube");
    assert_eq!(diagram(&bad).unwrap_err().code, "anchor-not-found");
    let third = NOTES.replace("_r #2", "_r #3");
    assert_eq!(diagram(&third).unwrap_err().code, "anchor-not-found");
}

#[test]
fn an_anchor_must_cover_whole_tokens() {
    let split = NOTES.replace("== NOTE u:s_q", "== NOTE s_q");
    assert_eq!(diagram(&split).unwrap_err().code, "anchor-splits-token");
}

#[test]
fn a_notes_file_needs_its_source() {
    assert_eq!(diagram("== TITLE\nx\n").unwrap_err().code, "bad-notes");
}

#[test]
fn backquoted_text_in_a_note_is_set_as_code() {
    let notes = NOTES.replace("Its second use.", "Like `f⍨` in Dyalog.");
    let svg = diagram(&notes).unwrap();
    assert!(svg.contains("<tspan class=\"inline\">f⍨</tspan>"), "{svg}");
    assert!(!svg.contains('`'), "the backquotes are not drawn");
}

#[test]
fn a_short_line_with_few_notes_gets_a_narrower_page() {
    let svg = diagram(NOTES).unwrap();
    assert!(!svg.contains("width=\"1500\""), "{}", &svg[..120]);
}
