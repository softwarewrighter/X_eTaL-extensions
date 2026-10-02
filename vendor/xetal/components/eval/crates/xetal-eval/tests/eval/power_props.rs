//! Function power (D-7): the sugar `f_^k x` means `k 'f_ p_ower x`,
//! and both apply f k times.

use proptest::prelude::*;

use crate::props::{lit, shown, typed};

proptest! {
    #![proptest_config(ProptestConfig::with_cases(40))]

    #[test]
    fn the_sugar_is_p_ower(v in prop::collection::vec(-9i64..10, 1..6), k in 0i64..6) {
        for f in ["n_eg", "r_ev", "u:i_nc"] {
            let def = "u:i_nc := { _r + 1 }; ";
            let sugar = typed(&format!("{def}{f}^{k} {}", lit(&v)));
            let builtin = typed(&format!("{def}{k} '{f} p_ower {}", lit(&v)));
            prop_assert_eq!(sugar, builtin);
        }
    }

    #[test]
    fn a_power_applies_the_function_k_times(v in prop::collection::vec(-9i64..10, 1..6), k in 0i64..6) {
        let added: Vec<i64> = v.iter().map(|x| x + k).collect();
        prop_assert_eq!(typed(&format!("u:i_nc := {{ _r + 1 }}; u:i_nc^{k} {}", lit(&v))), shown(&added));
    }
}
