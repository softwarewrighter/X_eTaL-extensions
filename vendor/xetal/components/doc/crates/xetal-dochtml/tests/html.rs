//! The HTML pieces: code drawn decorated with links, numbered source
//! lines, prose with inline code, examples, anchors and page names.

use xetal_base::Span;
use xetal_dochtml::{anchor, code, escape, example, lined, page, prose};
use xetal_doclink::{Link, Target};

fn href(t: Target) -> String {
    match t {
        Target::Builtin(b) => format!("builtins.html#{}", anchor(b)),
        _ => "x.html".into(),
    }
}

#[test]
fn code_is_drawn_decorated_with_its_names_linked() {
    let link = Link {
        span: Span::new(0, 4),
        target: Target::Builtin("r_ev"),
    };
    let html = code("r_ev 1 2", &[link], &href);
    assert!(
        html.starts_with("<a href=\"builtins.html#r_ev\" class=\"c-builtin\">"),
        "{html}"
    );
    assert!(html.contains("r\u{332}ev</a>"), "{html}");
    assert!(html.contains("<span class=\"c-number\">1</span>"), "{html}");
}

#[test]
fn text_is_escaped() {
    assert_eq!(escape("a < b & \"c\""), "a &lt; b &amp; &quot;c&quot;");
    assert!(code("1 < 2", &[], &href).contains("&lt;"));
}

#[test]
fn source_lines_are_numbered_with_anchors() {
    let html = lined("x := 1\ny := 2\n", &[], &href);
    assert!(
        html.contains("<span class=\"line\" id=\"L1\"><a class=\"ln\" href=\"#L1\">1</a>"),
        "{html}"
    );
    assert!(html.contains("id=\"L2\""), "{html}");
    assert!(!html.contains("id=\"L3\""), "{html}");
}

#[test]
fn prose_has_paragraphs_and_decorated_inline_code() {
    let html = prose("The mean of `r_ev v`.\n\nAnother & more.", &|c| {
        format!("<b>{c}</b>")
    });
    assert_eq!(
        html,
        "<p>The mean of <code><b>r_ev v</b></code>.</p>\n<p>Another &amp; more.</p>\n"
    );
}

#[test]
fn an_indented_paragraph_is_a_code_block() {
    let html = prose("Read\n\n  s:u_se< := 1\n  x", &|c| format!("<b>{c}</b>"));
    assert!(
        html.ends_with("<pre class=\"code\"><b>s:u_se< := 1\nx</b></pre>\n"),
        "{html}"
    );
}

#[test]
fn an_example_is_a_session_transcript() {
    let html = example("<b>1 + 1</b>", "2");
    assert_eq!(
        html,
        "<pre class=\"example\"><span class=\"prompt\">      </span><b>1 + 1</b>\n2</pre>\n"
    );
}

#[test]
fn anchors_and_pages_are_plain_ascii() {
    assert_eq!(anchor("l:m_ean"), "l.m_ean");
    assert_eq!(anchor("s:i_f<"), "s.i_f-3c");
    assert_eq!(anchor("[]R_EJECT"), "-5b-5dR_EJECT");
    assert_eq!(page("std/System.xtlm"), "std-System.xtlm");
    assert_eq!(page("../lib/Stats.xtl"), "lib-Stats.xtl");
    assert_eq!(page("-e"), "program");
}
