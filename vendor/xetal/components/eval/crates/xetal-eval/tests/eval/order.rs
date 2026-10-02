//! Evaluation order (E4): the function, then arguments right to left.

use crate::run;

#[test]
fn arguments_right_to_left() {
    assert_eq!(run("(p_rint! 1) + p_rint! 2"), "2\n1\n3");
}

#[test]
fn statements_in_order() {
    assert_eq!(run("p_rint! 1; p_rint! 2"), "1\n1\n2\n2");
}
