//! The steppable evaluator (D50): a run taken a few transitions at a
//! time prints exactly what one uninterrupted run prints, whatever the
//! slice size, and a runaway recursion is still an error.

use xetal_eval::{eval_in_slices, eval_source};

const PROGRAMS: &[&str] = &[
    "1 + 2; 1 2 3 * 10",
    "u:c_ount := { n -> n = 0 ? 0; 1 + u:c_ount n - 1 }; u:c_ount 2000",
    "x := 3; y := x * 2; p_rint! y; x + y",
    "count! := 0; count! := count! + 1; count!",
    "u:f_ := { a ~b -> a = 0 ? 0; b }; u:f_ 0 (1 / 0)",
    "'+ r_/ 1 2 3 4; '{ _r * 2 } e_ach 1 2 3",
    "u:a_vg := ['+ r_/ / t_ally]; u:a_vg 1 2 3 4",
    "u:f_ := { @ -> 42 }; u:f_ @",
    "1 o_- 1 2 3",
    "'+ s_\\ 1 2 3 4; '- s_\\ 5 3 1",
    "1 2 3 '* t_able 4 5",
    "M := 2 3 r_eshape r_ange 6; M '+ '* i_nner o_\\ M",
    "'+ r_/_2 2 3 r_eshape r_ange 6; '+ r_/_12 2 3 r_eshape r_ange 6",
    "(2 2 r_eshape 1 2 3 4) c_at_2 2 2 r_eshape 5 6 7 8",
    "'{ _r r_eshape _r } m_ap 1 2 3",
    "1 2 3 '+ e_ach 10 20 30",
    "u:i_nc := { _r + 1 }; 'u:i_nc 'n_eg c_ompose 5; 2 '- s_wap 10",
    "3 '{ p_rint! _r; _r * 2 } p_ower 1",
];

fn whole(src: &str) -> String {
    let mut out = Vec::new();
    let (_, result) = eval_source(src, &mut out);
    format!("{}{result:?}", String::from_utf8_lossy(&out))
}

fn sliced(src: &str, budget: usize) -> String {
    let mut out = Vec::new();
    let (_, result) = eval_in_slices(src, budget, &mut out);
    format!("{}{result:?}", String::from_utf8_lossy(&out))
}

#[test]
fn slices_print_what_one_run_prints() {
    for src in PROGRAMS {
        for budget in [1, 2, 7, 1000] {
            assert_eq!(
                sliced(src, budget),
                whole(src),
                "{src} in slices of {budget}"
            );
        }
    }
}

#[test]
fn runaway_recursion_in_slices_is_still_an_error() {
    let out = sliced("u:l_oop := { n -> u:l_oop n + 1 }; u:l_oop 0", 1000);
    assert!(out.contains("stack-overflow"), "{out}");
}
