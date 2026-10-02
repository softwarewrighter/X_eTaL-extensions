//! Lowering a train leaves notes for errors at its elements (D47).

use xetal_base::{Diagnostic, Span};
use xetal_core::lower;

/// The notes an error at `start..end` of `src` gets.
fn notes_at(src: &str, start: usize, end: usize) -> Vec<String> {
    let program = lower(src).expect("lowers");
    let d = Diagnostic::new("test", "message").with_span(Span::new(start, end));
    program.annotate(d).notes
}

#[test]
fn an_atop_explains_its_outer_element() {
    let notes = notes_at("[n_ot +] x", 1, 5);
    assert_eq!(
        notes[0],
        "in the train `[n_ot +]`, `n_ot` is applied as `n_ot (+ x)`, where x is the train's argument"
    );
    assert_eq!(
        notes[1],
        "`+` takes two arguments, so `+ x` is a function waiting for the other"
    );
}

#[test]
fn an_error_elsewhere_gets_no_notes() {
    assert!(notes_at("[n_ot +] x", 0, 10).is_empty());
    assert!(notes_at("(n_ot 1) + 2", 1, 5).is_empty());
}

#[test]
fn a_local_name_has_no_built_in_arity() {
    let notes = notes_at("{ n_ot -> [r_ev n_ot t_ally] 1 2 }", 16, 20);
    assert_eq!(notes.len(), 1, "{notes:?}");
}

#[test]
fn a_dyadic_tine_given_two_says_so() {
    let notes = notes_at("1 2 [r_ev + a_bs] 3 4", 5, 9);
    assert_eq!(
        notes[1],
        "`r_ev` takes one argument, but here it is given two"
    );
}
