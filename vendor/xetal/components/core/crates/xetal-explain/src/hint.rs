//! Why an element may not fit where the train puts it, from the
//! arities of built-ins.

use xetal_syntax::Fun;

use crate::say::Say;

/// How many arguments a function takes, when known (a built-in).
pub type Arity<'a> = &'a dyn Fn(&Fun) -> Option<usize>;

/// `f` given two arguments though it takes one.
pub(crate) fn given_two(say: &Say, arity: Arity, f: &Fun) -> Option<String> {
    (arity(f) == Some(1)).then(|| {
        format!(
            "`{}` takes one argument, but here it is given two",
            say.spell(f)
        )
    })
}

/// `f` given one argument (in a monadic train) though it takes two, so
/// its result is a function.
pub(crate) fn waiting(say: &Say, arity: Arity, f: &Fun) -> Option<String> {
    (!say.dyadic && arity(f) == Some(2)).then(|| {
        let s = say.spell(f);
        format!("`{s}` takes two arguments, so `{s} x` is a function waiting for the other")
    })
}
