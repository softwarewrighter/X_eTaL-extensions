//! Type foundations: unification, the Num constraint, schemes, display.

use xetal_base::Span;
use xetal_ty::{Type, Unifier};

fn span() -> Span {
    Span::new(0, 1)
}

fn fun(a: Type, b: Type) -> Type {
    Type::Fn(Box::new(a), Box::new(b))
}

#[test]
fn equal_types_unify_and_different_ones_do_not() {
    let mut u = Unifier::default();
    assert!(u.unify(&Type::Int, &Type::Int, span()).is_ok());
    let e = u.unify(&Type::Int, &Type::Bool, span()).unwrap_err();
    assert_eq!(e.code, "type-mismatch");
    assert_eq!(e.message, "expected Int, found Bool");
    assert_eq!(e.span, Some(span()));
}

#[test]
fn variables_are_solved() {
    let mut u = Unifier::default();
    let a = u.fresh();
    let b = u.fresh();
    u.unify(
        &fun(a.clone(), b.clone()),
        &fun(Type::Int, Type::Float),
        span(),
    )
    .unwrap();
    assert_eq!(u.resolve(&a), Type::Int);
    assert_eq!(u.resolve(&fun(a, b)).to_string(), "Int -> Float");
}

#[test]
fn occurs_check() {
    let mut u = Unifier::default();
    let a = u.fresh();
    let e = u.unify(&a, &fun(a.clone(), Type::Int), span()).unwrap_err();
    assert_eq!(e.code, "infinite-type");
}

#[test]
fn num_constraint() {
    let mut u = Unifier::default();
    let n = u.fresh_num();
    assert!(u.unify(&n, &Type::Float, span()).is_ok());
    let m = u.fresh_num();
    let e = u.unify(&m, &Type::Unit, span()).unwrap_err();
    assert_eq!(e.message, "expected a number, found Unit");
    let k = u.fresh_num();
    let e = u.unify(&Type::Unit, &k, span()).unwrap_err();
    assert_eq!(e.message, "expected Unit, found a number");
    // the constraint moves to a variable it is unified with
    let p = u.fresh_num();
    let q = u.fresh();
    u.unify(&p, &q, span()).unwrap();
    assert!(u.unify(&q, &Type::Unit, span()).is_err());
}

#[test]
fn truthy_constraint_and_defaulting() {
    let mut u = Unifier::default();
    let t = u.fresh_truthy();
    assert!(u.unify(&t, &Type::Bool, span()).is_ok());
    let t2 = u.fresh_truthy();
    let e = u.unify(&t2, &Type::Float, span()).unwrap_err();
    assert_eq!(e.message, "expected Bool or Int, found Float");
    // a truthy value used as a number is an Int
    let (c, n) = (u.fresh_truthy(), u.fresh_num());
    u.unify(&c, &n, span()).unwrap();
    assert!(u.unify(&c, &Type::Float, span()).is_err());
    assert_eq!(u.defaulted(&c), Type::Int);
    let (b, n) = (u.fresh_truthy(), u.fresh_num());
    assert_eq!(u.defaulted(&b), Type::Bool);
    assert_eq!(u.defaulted(&n), Type::Int);
}

#[test]
fn generalize_and_instantiate() {
    let mut u = Unifier::default();
    let a = u.fresh();
    let identity = fun(a.clone(), a);
    let scheme = u.generalize(&identity, &[]);
    assert_eq!(scheme.to_string(), "a -> a");
    let (one, _) = u.instantiate(&scheme);
    let (two, _) = u.instantiate(&scheme);
    u.unify(&one, &fun(Type::Int, Type::Int), span()).unwrap();
    u.unify(&two, &fun(Type::Bool, Type::Bool), span()).unwrap();
}

#[test]
fn free_variables_in_the_environment_stay_monomorphic() {
    let mut u = Unifier::default();
    let a = u.fresh();
    let scheme = u.generalize(&fun(a.clone(), a.clone()), std::slice::from_ref(&a));
    let (inst, _) = u.instantiate(&scheme);
    u.unify(&inst, &fun(Type::Int, Type::Int), span()).unwrap();
    assert_eq!(u.resolve(&a), Type::Int);
}

#[test]
fn display() {
    let mut u = Unifier::default();
    let (a, b) = (u.fresh(), u.fresh());
    let k = fun(a.clone(), fun(b, a));
    assert_eq!(u.generalize(&k, &[]).to_string(), "a -> b -> a");
    let (f, x) = (u.fresh(), u.fresh());
    let apply = fun(fun(x.clone(), f.clone()), fun(x, f));
    assert_eq!(u.generalize(&apply, &[]).to_string(), "(a -> b) -> a -> b");
    let n = u.fresh_num();
    let add = fun(n.clone(), fun(n.clone(), n));
    assert_eq!(u.generalize(&add, &[]).to_string(), "Num a => a -> a -> a");
    let (m, t) = (u.fresh_num(), u.fresh_truthy());
    let eq = fun(m.clone(), fun(m, t));
    assert_eq!(
        u.generalize(&eq, &[]).to_string(),
        "(Num a, Truthy b) => a -> a -> b"
    );
    assert_eq!(fun(Type::Unit, Type::Int).to_string(), "Unit -> Int");
}

mod props {
    use proptest::prelude::*;
    use xetal_base::Span;
    use xetal_ty::{Type, TypeVar, Unifier};

    fn arb_type() -> impl Strategy<Value = Type> {
        let leaf = prop_oneof![
            Just(Type::Unit),
            Just(Type::Bool),
            Just(Type::Int),
            Just(Type::Float),
            (1u32..5).prop_map(|v| Type::Var(TypeVar(v))),
        ];
        leaf.prop_recursive(4, 16, 2, |inner| {
            prop_oneof![
                (inner.clone(), inner.clone())
                    .prop_map(|(a, b)| Type::Fn(Box::new(a), Box::new(b))),
            ]
        })
    }

    proptest! {
        #[test]
        fn a_type_unifies_with_itself(t in arb_type()) {
            let mut u = Unifier::default();
            prop_assert!(u.unify(&t, &t, Span::default()).is_ok());
        }

        #[test]
        fn unification_never_panics_and_resolve_is_idempotent(a in arb_type(), b in arb_type()) {
            let mut u = Unifier::default();
            let _ = u.unify(&a, &b, Span::default());
            let once = u.resolve(&a);
            prop_assert_eq!(u.resolve(&once), once);
        }
    }
}

#[test]
fn instantiation_reports_the_num_instances_in_order() {
    let mut u = Unifier::default();
    let (a, b, c) = (u.fresh_num(), u.fresh(), u.fresh_num());
    let scheme = u.generalize(&fun(a, fun(b, c)), &[]);
    let (t, nums) = u.instantiate(&scheme);
    let Type::Fn(first, rest) = t else {
        panic!("a function")
    };
    let Type::Fn(_, last) = *rest else {
        panic!("a function")
    };
    assert_eq!(nums, vec![*first, *last]);
}

#[test]
fn defaulting_skips_quantified_variables() {
    let mut u = Unifier::default();
    let mark = u.mark();
    let (a, b) = (u.fresh_num(), u.fresh_num());
    let Type::Var(va) = a.clone() else {
        panic!("a variable")
    };
    let skip = std::collections::HashSet::from([va]);
    u.default_since(mark, span(), &skip).unwrap();
    assert_eq!(u.resolve(&a), a);
    assert_eq!(u.resolve(&b), Type::Int);
}

#[test]
fn defaulting_skips_variables_bound_to_quantified_ones() {
    let mut u = Unifier::default();
    let root = u.fresh_num();
    let mark = u.mark();
    let alias = u.fresh_num();
    u.unify(&alias, &root, span()).unwrap();
    let Type::Var(q) = u.resolve(&root) else {
        panic!("a variable")
    };
    let skip = std::collections::HashSet::from([q]);
    u.default_since(mark, span(), &skip).unwrap();
    assert!(matches!(u.resolve(&alias), Type::Var(_)));
}

#[test]
fn defaulting_skips_quantified_variables_inside_structures() {
    let mut u = Unifier::default();
    let q = u.fresh_num();
    let mark = u.mark();
    let f = u.fresh();
    u.unify(&f, &fun(Type::Unit, q.clone()), span()).unwrap();
    let Type::Var(qv) = q.clone() else {
        panic!("a variable")
    };
    let skip = std::collections::HashSet::from([qv]);
    u.default_since(mark, span(), &skip).unwrap();
    assert_eq!(u.resolve(&q), q);
}

#[test]
fn eq_and_ord_classes() {
    // T8: `=` on any scalar type, the orderings on numbers and Char.
    use xetal_ty::Classes;
    let (eq, ord) = (
        Classes::named("Eq").unwrap(),
        Classes::named("Ord").unwrap(),
    );
    let mut u = Unifier::default();
    let a = u.fresh_in(eq);
    assert!(u.unify(&a, &Type::Char, span()).is_ok());
    let b = u.fresh_in(ord);
    let e = u.unify(&b, &Type::Bool, span()).unwrap_err();
    assert_eq!(e.message, "expected a number or Char, found Bool");
    let c = u.fresh_in(ord);
    let d = u.fresh_num();
    u.unify(&c, &d, span()).unwrap();
    let e = u.unify(&c, &Type::Char, span()).unwrap_err();
    assert_eq!(e.message, "expected a number, found Char");
    assert!(Classes::named("Show").is_none());
}
