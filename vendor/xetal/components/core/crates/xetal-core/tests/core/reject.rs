//! Bindings and arguments that desugaring rejects.

use crate::reject;

#[test]
fn user_functions_are_named_with_u() {
    assert_eq!(reject("f_ := { _r }"), "bad-binding");
}

#[test]
fn plain_variables_take_no_prefix() {
    assert_eq!(reject("u:x := 3"), "bad-binding");
}

#[test]
fn imported_namespaces_are_read_only() {
    assert_eq!(reject("c:K_ := { x y -> x }"), "bad-binding");
    assert_eq!(reject("{ x -> u:g_ := x }"), "bad-binding");
}

#[test]
fn lambda_arguments_outside_lambdas() {
    assert_eq!(reject("_r + 1"), "arg-outside-lambda");
    assert_eq!(reject("x := _l_ 3"), "arg-outside-lambda");
}

#[test]
fn parse_errors_come_through() {
    assert_eq!(reject("- 3"), "symbol-needs-left");
}
