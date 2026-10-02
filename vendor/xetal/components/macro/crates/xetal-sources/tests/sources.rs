//! A combined program text and the way back to where it was written.

use xetal_base::{Diagnostic, Span};
use xetal_sources::Sources;

fn two_files() -> Sources {
    let mut s = Sources::default();
    let lib = s.add("lib/Stats.xtl", "l:m_ean := 1\nl:s_d := 2\n");
    let main = s.add("main.xtl", "x := 3\ny\n");
    s.copy(lib, 0..13);
    s.replace(lib, 13..18, "LA:s_d");
    s.copy(lib, 18..24);
    s.copy(main, 0..9);
    s
}

#[test]
fn pieces_are_joined_into_one_text() {
    assert_eq!(
        two_files().combined(),
        "l:m_ean := 1\nLA:s_d := 2\nx := 3\ny\n"
    );
}

#[test]
fn an_offset_is_found_in_its_file_line_and_column() {
    let s = two_files();
    let at = s.locate(26);
    assert_eq!((at.file, at.line, at.col, at.offset), ("main.xtl", 1, 2, 1));
    let at = s.locate(13);
    assert_eq!(
        (at.file, at.line, at.col, at.offset),
        ("lib/Stats.xtl", 2, 1, 13)
    );
}

#[test]
fn a_replaced_piece_maps_to_its_original_token() {
    let s = two_files();
    let at = s.locate(16);
    assert_eq!((at.file, at.offset), ("lib/Stats.xtl", 13));
}

#[test]
fn diagnostics_name_their_file_when_there_are_several() {
    let s = two_files();
    let d = Diagnostic::new("undefined-name", "y is not defined").with_span(Span::new(32, 33));
    assert_eq!(
        s.describe(&d),
        "error[undefined-name]: y is not defined at main.xtl:2:1"
    );
}

#[test]
fn one_file_keeps_the_plain_report() {
    let mut s = Sources::default();
    let f = s.add("-e", "1 / 0");
    s.copy(f, 0..5);
    let d = Diagnostic::new("division-by-zero", "division by zero").with_span(Span::new(0, 5));
    assert_eq!(s.describe(&d), d.to_string());
}

#[test]
fn a_diagnostic_without_a_span_is_unchanged() {
    let d = Diagnostic::new("io", "cannot read");
    assert_eq!(two_files().describe(&d), d.to_string());
}

proptest::proptest! {
    #[test]
    fn a_copied_file_maps_every_offset_to_itself(text in "[a-z \n]{1,40}", at in 0usize..40) {
        let mut s = Sources::default();
        let f = s.add("f.xtl", &text);
        s.copy(f, 0..text.len());
        let at = at.min(text.len() - 1);
        let loc = s.locate(at);
        proptest::prop_assert_eq!(loc.offset, at);
        proptest::prop_assert_eq!(loc.line, text[..at].matches('\n').count() + 1);
    }
}
