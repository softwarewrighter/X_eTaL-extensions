use xetal_base::{Diagnostic, LANG_NAME, Span};

#[test]
fn lang_name_is_the_display_name() {
    assert_eq!(LANG_NAME, "X_eTaL");
}

#[test]
fn span_join_covers_both() {
    let joined = Span::new(4, 6).join(Span::new(1, 3));
    assert_eq!(joined, Span::new(1, 6));
    assert_eq!(joined.len(), 5);
    assert!(Span::new(3, 3).is_empty());
}

#[test]
fn diagnostic_display_includes_code_span_and_notes() {
    let d = Diagnostic::new("E1", "bad token")
        .with_span(Span::new(2, 4))
        .with_note("try spaces");
    assert_eq!(
        d.to_string(),
        "error[E1]: bad token at 2..4\n  note: try spaces"
    );
}

#[test]
fn unsupported_names_the_stage() {
    let d = Diagnostic::unsupported("lex");
    assert_eq!(d.code, "unsupported");
    assert_eq!(
        d.to_string(),
        "error[unsupported]: stage `lex` is not implemented"
    );
}

#[test]
fn warnings_print_as_warnings() {
    let w = Diagnostic::warning("shadows-builtin", "parameter r_ev shadows a built-in");
    assert_eq!(
        w.to_string(),
        "warning[shadows-builtin]: parameter r_ev shadows a built-in"
    );
}
