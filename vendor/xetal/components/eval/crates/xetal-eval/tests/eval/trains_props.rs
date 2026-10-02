//! Trains (TR1-TR4) are their desugaring: for any functions and any
//! arguments, a train gives what its written-out form gives.

use proptest::prelude::*;

use crate::props::{lit, typed};

/// Monadic functions on a vector: some keep its length, some give one
/// number, so forks mix vectors with scalars that extend.
const MONADIC: [&str; 8] = [
    "r_ev",
    "n_eg",
    "a_bs",
    "s_ort",
    "'+ s_\\",
    "'+ r_/",
    "t_ally",
    "'m_ax r_/",
];

/// Dyadic functions on two vectors of one length (or a vector and a
/// number).
const DYADIC: [&str; 5] = ["+", "-", "*", "m_ax", "m_in"];

fn pick<'a>(pool: &[&'a str], i: usize) -> &'a str {
    pool[i % pool.len()]
}

fn ints() -> impl Strategy<Value = Vec<i64>> {
    prop::collection::vec(-9i64..10, 1..6)
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(60))]

    #[test]
    fn a_fork_is_f_x_g_h_x(v in ints(), f in 0usize..8, g in 0usize..5, h in 0usize..8) {
        let (f, g, h) = (pick(&MONADIC, f), pick(&DYADIC, g), pick(&MONADIC, h));
        let x = lit(&v);
        let train = typed(&format!("[{f} {g} {h}] {x}"));
        let written = typed(&format!("({f} {x}) {g} ({h} {x})"));
        prop_assert_eq!(train, written);
    }

    #[test]
    fn an_atop_is_f_of_g_x(v in ints(), f in 0usize..8, g in 0usize..8) {
        let (f, g) = (pick(&MONADIC, f), pick(&MONADIC, g));
        let x = lit(&v);
        prop_assert_eq!(typed(&format!("[{f} {g}] {x}")), typed(&format!("{f} ({g} {x})")));
    }

    #[test]
    fn a_dyadic_fork_is_x_f_y_g_x_h_y(
        pairs in prop::collection::vec((-9i64..10, -9i64..10), 1..6),
        f in 0usize..5, g in 0usize..5, h in 0usize..5,
    ) {
        let (a, b): (Vec<i64>, Vec<i64>) = pairs.into_iter().unzip();
        let (f, g, h) = (pick(&DYADIC, f), pick(&DYADIC, g), pick(&DYADIC, h));
        let (x, y) = (lit(&a), lit(&b));
        let train = typed(&format!("({x}) [{f} {g} {h}] {y}"));
        let written = typed(&format!("(({x}) {f} {y}) {g} (({x}) {h} {y})"));
        prop_assert_eq!(train, written);
    }

    #[test]
    fn a_long_train_groups_from_the_right(
        v in ints(), a in 0usize..8, b in 0usize..5, c in 0usize..8, d in 0usize..5, e in 0usize..8,
    ) {
        let (a, b, c) = (pick(&MONADIC, a), pick(&DYADIC, b), pick(&MONADIC, c));
        let (d, e) = (pick(&DYADIC, d), pick(&MONADIC, e));
        let x = lit(&v);
        let train = typed(&format!("[{a} {b} {c} {d} {e}] {x}"));
        let written = typed(&format!("({a} {x}) {b} (({c} {x}) {d} ({e} {x}))"));
        prop_assert_eq!(train, written);
    }

    #[test]
    fn a_named_train_is_the_train(v in ints(), f in 0usize..8, g in 0usize..5, h in 0usize..8) {
        let (f, g, h) = (pick(&MONADIC, f), pick(&DYADIC, g), pick(&MONADIC, h));
        let x = lit(&v);
        let named = typed(&format!("u:t_ := [{f} {g} {h}]; u:t_ {x}"));
        prop_assert_eq!(named, typed(&format!("[{f} {g} {h}] {x}")));
    }

    #[test]
    fn a_fork_over_text_joins_like_its_written_form(
        w in "[a-e]{1,6}", f in 0usize..3, h in 0usize..3,
    ) {
        let text = ["r_ev", "s_ort", "u_nique"];
        let (f, h) = (pick(&text, f), pick(&text, h));
        let x = format!("{w:?}");
        let train = typed(&format!("[{f} c_at {h}] {x}"));
        prop_assert_eq!(train, typed(&format!("({f} {x}) c_at {h} {x}")));
    }
}
