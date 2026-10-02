//! Segments as HTML spans: the decorated form for a web page, colored
//! by the same classes as the live demo's Rendered pane.

use xetal_view::{html, view};

#[test]
fn each_class_is_a_span_and_plain_text_is_bare() {
    let out = html(&view("x := u:s_quare 7"));
    assert!(
        out.starts_with("x \u{2190} <span class=\"c-userfunc\">"),
        "{out}"
    );
    assert!(out.ends_with("<span class=\"c-number\">7</span>"), "{out}");
}

#[test]
fn text_is_escaped() {
    let out = html(&view("\"<a & b>\" c_at 1 < 2"));
    assert!(out.contains("&quot;&lt;a &amp; b&gt;&quot;"), "{out}");
    assert!(!out.contains("<a"), "{out}");
}

#[test]
fn a_run_of_one_class_is_one_span() {
    let out = html(&view("# a comment"));
    assert_eq!(out.matches("<span").count(), 1, "{out}");
}

#[test]
fn text_that_does_not_lex_is_shown_as_an_error_not_dropped() {
    let out = html(&view("1 $ 2"));
    assert!(out.contains("c-error"), "{out}");
    assert!(out.contains('$'), "{out}");
}
