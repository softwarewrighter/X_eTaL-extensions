//! Slicing, joining and unescaping keep each byte's origin.

use xetal_base::Span;
use xetal_mapped::Mapped;

#[test]
fn a_slice_keeps_its_place() {
    let m = Mapped::new("abcdef").slice(2..5);
    assert_eq!(m.text(), "cde");
    assert_eq!(m.span(Span::new(1, 2)), Span::new(3, 4));
}

#[test]
fn glue_maps_to_the_whole_call() {
    let mut m = Mapped::new("ab");
    m.glue("XYZ", 10..20);
    m.push(&Mapped::new("0123456789cd").slice(10..12));
    assert_eq!(m.text(), "abXYZcd");
    assert_eq!(m.span(Span::new(3, 4)), Span::new(10, 20));
    assert_eq!(m.span(Span::new(5, 7)), Span::new(10, 12));
    assert_eq!(m.span(Span::new(0, 1)), Span::new(0, 1));
}

#[test]
fn unescaping_maps_each_escape_to_its_backslash() {
    let raw = r#"x\"y\\z\nw"#;
    let m = Mapped::new(raw).unescape();
    assert_eq!(m.text(), "x\"y\\z\nw");
    assert_eq!(m.span(Span::new(1, 2)), Span::new(1, 2));
    assert_eq!(m.span(Span::new(2, 3)), Span::new(3, 4));
    assert_eq!(m.span(Span::new(6, 7)), Span::new(9, 10));
}
