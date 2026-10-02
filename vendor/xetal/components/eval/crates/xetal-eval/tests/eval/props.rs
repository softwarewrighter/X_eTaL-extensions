//! Properties of the higher-order and search built-ins (B6, B7), run
//! through the checker and evaluator on generated Int vectors, against
//! a Rust model where there is one.

use proptest::prelude::*;

/// Printed output of a type-checked program.
pub(crate) fn typed(src: &str) -> String {
    let mut program = xetal_core::lower(src).expect("lowers");
    xetal_types::check_program(&mut program).unwrap_or_else(|e| panic!("{src:?}: {e:?}"));
    let mut out = Vec::new();
    if let Err(e) = xetal_eval::eval_program(&program, &mut out, None).1 {
        panic!("{src:?} should evaluate, got {e:?}");
    }
    String::from_utf8(out)
        .expect("utf-8")
        .trim_end()
        .to_string()
}

/// A vector literal of any length (a one-item strand would be a scalar).
pub(crate) fn lit(v: &[i64]) -> String {
    let items: Vec<String> = v.iter().map(ToString::to_string).collect();
    match v.len() {
        0 => "(0 t_ake 0)".into(),
        n => format!("({n} t_ake {})", items.join(" ")),
    }
}

pub(crate) fn shown(v: &[i64]) -> String {
    v.iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(" ")
}

fn ints(max: usize) -> impl Strategy<Value = Vec<i64>> {
    prop::collection::vec(-9i64..10, 0..max)
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(48))]

    #[test]
    fn reduce_distributes_over_cat(a in ints(6), b in ints(6)) {
        let joined = typed(&format!("'+ r_/ {} c_at {}", lit(&a), lit(&b)));
        let split = typed(&format!("('+ r_/ {}) + '+ r_/ {}", lit(&a), lit(&b)));
        prop_assert_eq!(joined, split);
    }

    #[test]
    fn last_of_scan_is_reduce(v in ints(7).prop_filter("non-empty", |v| !v.is_empty()),
                              f in prop::sample::select(vec!["'+", "'-", "'*", "'m_ax", "'m_in", "'{ (2 * _l) - _r }"])) {
        let last = typed(&format!("-1 t_ake {f} s_\\ {}", lit(&v)));
        prop_assert_eq!(last, typed(&format!("{f} r_/ {}", lit(&v))));
    }

    #[test]
    fn one_pass_scan_matches_the_definition(v in ints(8)) {
        let fast = typed(&format!("'+ s_\\ {}", lit(&v)));
        prop_assert_eq!(fast, typed(&format!("'{{ _l + _r }} s_\\ {}", lit(&v))));
    }

    #[test]
    fn each_of_identity_is_identity(v in ints(8)) {
        prop_assert_eq!(typed(&format!("'i_d e_ach {}", lit(&v))), shown(&v));
    }

    #[test]
    fn table_shape_joins_shapes(a in ints(5), b in ints(5)) {
        let shape = typed(&format!("s_hape {} '+ t_able {}", lit(&a), lit(&b)));
        prop_assert_eq!(shape, format!("{} {}", a.len(), b.len()));
    }

    #[test]
    fn sort_and_grade_match_a_stable_sort(v in ints(9)) {
        let mut sorted = v.clone();
        sorted.sort();
        let mut order: Vec<usize> = (0..v.len()).collect();
        order.sort_by_key(|i| v[*i]);
        let grade: Vec<i64> = order.iter().map(|i| *i as i64 + 1).collect();
        prop_assert_eq!(typed(&format!("s_ort {}", lit(&v))), shown(&sorted));
        prop_assert_eq!(typed(&format!("g_rade {}", lit(&v))), shown(&grade));
        let selected = typed(&format!("w := {}\n(g_rade w) s_elect w", lit(&v)));
        prop_assert_eq!(selected, shown(&sorted));
    }

    #[test]
    fn unique_and_index_of_match_a_model(v in ints(9), w in ints(4)) {
        let mut seen: Vec<i64> = Vec::new();
        for x in &v {
            if !seen.contains(x) {
                seen.push(*x);
            }
        }
        let at: Vec<i64> = w
            .iter()
            .map(|x| v.iter().position(|y| y == x).unwrap_or(v.len()) as i64 + 1)
            .collect();
        prop_assert_eq!(typed(&format!("u_nique {}", lit(&v))), shown(&seen));
        prop_assert_eq!(typed(&format!("{} i_ndexOf {}", lit(&v), lit(&w))), shown(&at));
    }
}
