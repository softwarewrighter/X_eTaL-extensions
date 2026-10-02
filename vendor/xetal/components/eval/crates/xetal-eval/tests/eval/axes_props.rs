//! Properties of rotate, reverse and axis subscripts (A2-A6, R1) on
//! generated Int matrices, through the checker and evaluator.

use proptest::prelude::*;

use crate::props::{lit, shown, typed};

/// A matrix literal `(r c r_eshape items)`.
fn mat(rows: usize, items: &[i64]) -> String {
    let cols = items.len() / rows;
    let body: Vec<String> = items.iter().map(ToString::to_string).collect();
    format!("({rows} {cols} r_eshape {})", body.join(" "))
}

fn matrices() -> impl Strategy<Value = (usize, Vec<i64>)> {
    (1usize..4, 1usize..4)
        .prop_flat_map(|(r, c)| (Just(r), prop::collection::vec(-9i64..10, r * c)))
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(40))]

    #[test]
    fn rotating_back_and_reversing_twice_restore(v in prop::collection::vec(-9i64..10, 0..7), n in -9i64..10) {
        prop_assert_eq!(typed(&format!("{n} o_- {} o_- {}", -n, lit(&v))), shown(&v));
        prop_assert_eq!(typed(&format!("r_ev r_ev {}", lit(&v))), shown(&v));
    }

    #[test]
    fn axis_one_is_the_plain_function((r, xs) in matrices(), n in -5i64..6) {
        let m = mat(r, &xs);
        prop_assert_eq!(typed(&format!("r_ev_1 {m}")), typed(&format!("r_ev {m}")));
        prop_assert_eq!(typed(&format!("{n} o_-_1 {m}")), typed(&format!("{n} o_- {m}")));
        prop_assert_eq!(typed(&format!("'+ r_/_1 {m}")), typed(&format!("'+ r_/ {m}")));
    }

    #[test]
    fn every_combination_holds_each_single_rotation((r, xs) in matrices(), i in 1usize..4, j in 1usize..4) {
        let (m, amounts) = (mat(r, &xs), [-1i64, 0, 1]);
        let picked = typed(&format!("{j} s_elect {i} s_elect -1 0 1 o_-_12 {m}"));
        let direct = typed(&format!("{} o_-_2 {} o_- {m}", amounts[j - 1], amounts[i - 1]));
        prop_assert_eq!(picked, direct);
    }

    #[test]
    fn axis_functions_match_a_model((r, xs) in matrices()) {
        let rows: Vec<i64> = xs.chunks(xs.len() / r).map(|row| row.iter().sum()).collect();
        let total: i64 = xs.iter().sum();
        let flipped: Vec<i64> = xs.chunks(xs.len() / r).flat_map(|row| row.iter().rev().copied()).collect();
        let scanned: Vec<i64> = xs
            .chunks(xs.len() / r)
            .flat_map(|row| row.iter().scan(0, |acc, x| { *acc += x; Some(*acc) }).collect::<Vec<_>>())
            .collect();
        let as_matrix = |v: &[i64]| typed(&mat(r, v));
        prop_assert_eq!(typed(&format!("r_ev_2 {}", mat(r, &xs))), as_matrix(&flipped));
        prop_assert_eq!(typed(&format!("'+ s_\\_2 {}", mat(r, &xs))), as_matrix(&scanned));
        prop_assert_eq!(typed(&format!("'+ r_/_2 {}", mat(r, &xs))), shown(&rows));
        prop_assert_eq!(typed(&format!("'+ r_/_12 {}", mat(r, &xs))), total.to_string());
    }
}
