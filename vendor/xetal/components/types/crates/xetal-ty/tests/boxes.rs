//! Box a (A7, B14): boxes unify only with boxes, print as `Box Char`,
//! and are in Eq (when what they hold is) but in no other class.

use xetal_base::Span;
use xetal_ty::{Classes, Type, Unifier};

fn span() -> Span {
    Span::new(0, 1)
}

fn boxed(t: Type) -> Type {
    Type::Box(Box::new(t))
}

#[test]
fn a_box_unifies_only_with_a_box() {
    let mut u = Unifier::default();
    assert!(
        u.unify(&boxed(Type::Char), &boxed(Type::Char), span())
            .is_ok()
    );
    let e = u
        .unify(&Type::Char, &boxed(Type::Char), span())
        .unwrap_err();
    assert_eq!(e.message, "expected Char, found Box Char");
    let a = u.fresh();
    u.unify(&boxed(a.clone()), &boxed(Type::Int), span())
        .unwrap();
    assert_eq!(u.resolve(&a), Type::Int);
}

#[test]
fn boxes_print_with_parentheses_when_nested() {
    assert_eq!(boxed(Type::Char).to_string(), "Box Char");
    assert_eq!(boxed(boxed(Type::Int)).to_string(), "Box (Box Int)");
    let f = Type::Fn(Box::new(Type::Int), Box::new(Type::Int));
    assert_eq!(boxed(f).to_string(), "Box (Int -> Int)");
}

#[test]
fn a_box_is_eq_when_its_item_is() {
    let mut u = Unifier::default();
    let eq = u.fresh_in(Classes::named("Eq").unwrap());
    let item = u.fresh();
    u.unify(&eq, &boxed(item.clone()), span()).unwrap();
    let e = u.unify(
        &item,
        &Type::Fn(Box::new(Type::Int), Box::new(Type::Int)),
        span(),
    );
    assert!(e.is_err(), "a box of functions is not Eq");
}

#[test]
fn a_box_is_not_a_number_or_ordered() {
    for class in ["Num", "Ord", "Truthy"] {
        let mut u = Unifier::default();
        let v = u.fresh_in(Classes::named(class).unwrap());
        assert!(u.unify(&v, &boxed(Type::Int), span()).is_err(), "{class}");
    }
}
