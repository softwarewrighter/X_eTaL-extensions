//! Scalars, symbols and the numeric rules T1, T2, T3, D-4, D-10.

use crate::run;

#[test]
fn arithmetic() {
    assert_eq!(run("1 + 2"), "3");
    assert_eq!(run("10 - 3"), "7");
    assert_eq!(run("2 * 3 + 4"), "14"); // right to left: 2 * (3 + 4)
    assert_eq!(run("7 / 2"), "3.5");
    assert_eq!(run("6 / 2"), "3.0");
    assert_eq!(run("7 d_iv 2"), "3");
    assert_eq!(run("7 m_od 3"), "1");
    assert_eq!(run("-7 d_iv 2"), "-4");
    assert_eq!(run("n_eg 3"), "-3");
    assert_eq!(run("a_bs -2.5"), "2.5");
    assert_eq!(run("f_loor 2.7"), "2");
    assert_eq!(run("c_eiling 2.1"), "3");
    assert_eq!(run("3 m_ax 5"), "5");
    assert_eq!(run("3 m_in 5"), "3");
    assert_eq!(run("f_loat 3"), "3.0");
}

#[test]
fn powers() {
    assert_eq!(run("2^10"), "1024");
    assert_eq!(run("x := 3; x^2 + 4^2"), "25");
    assert_eq!(run("2^-1"), "0.5");
    assert_eq!(run("4^0.5"), "2.0");
    assert_eq!(run("2 ^ 10"), "1024");
    assert_eq!(run("2.0 ^ -1"), "0.5");
}

#[test]
fn comparisons_and_bools() {
    assert_eq!(run("3 = 3"), "1");
    assert_eq!(run("3 = 3.0"), "1");
    assert_eq!(run("(0.1 + 0.2) = 0.3"), "0");
    assert_eq!(run("(0.1 + 0.2) e_q~ 0.3"), "1");
    assert_eq!(run("0.1 + 0.2 = 0.3"), "0.1"); // right to left: 0.1 + (0.2 = 0.3)
    assert_eq!(run("1 != 2"), "1");
    assert_eq!(run("2 <= 2"), "1");
    assert_eq!(run("(3 = 3) + 1"), "2"); // Bool -> Int in arithmetic
    assert_eq!(run("(1 < 2) & 2 > 1"), "1");
    assert_eq!(run("0 | 1"), "1"); // Int 0/1 -> Bool where a Bool is required
    assert_eq!(run("n_ot 1 = 2"), "1");
}

#[test]
fn values_print_as_input() {
    assert_eq!(run("42"), "42");
    assert_eq!(run("-3"), "-3");
    assert_eq!(run("2.5"), "2.5");
    assert_eq!(run("@"), "@");
    assert_eq!(run("{ _r }"), "<function>");
    assert_eq!(run("1\n2 + 2"), "1\n4");
}
