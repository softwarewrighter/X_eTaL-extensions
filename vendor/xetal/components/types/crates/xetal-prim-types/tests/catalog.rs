//! Every implemented catalog built-in has the type its signature says.

use xetal_base::Span;
use xetal_catalog::BUILTINS;
use xetal_prim_types::prim_type;
use xetal_ty::Unifier;

#[test]
fn signatures_round_trip() {
    for b in BUILTINS.iter().filter(|b| b.implemented) {
        let mut u = Unifier::default();
        let t = prim_type(b.name, &mut u, Span::new(0, 1)).unwrap();
        assert_eq!(u.generalize(&t, &[]).to_string(), b.sig, "{}", b.name);
    }
}

#[test]
fn unknown_names_are_errors() {
    let mut u = Unifier::default();
    assert!(prim_type("r_ev", &mut u, Span::new(0, 1)).is_ok());
    let err = prim_type("q_uux", &mut u, Span::new(0, 1)).unwrap_err();
    assert_eq!(err.code, "unknown-builtin");
}
