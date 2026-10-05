//! The terminal a program runs on (QD6): key names as programs see
//! them, and the plain terminal's facts.

use xetal_tty::{Plain, Tty, key_name};

#[test]
fn keys_are_named_as_programs_see_them() {
    assert_eq!(key_name('a'), "a");
    assert_eq!(key_name('\n'), "Enter");
    assert_eq!(key_name('\r'), "Enter");
    assert_eq!(key_name('\x1b'), "Escape");
    assert_eq!(key_name('\x7f'), "Backspace");
    assert_eq!(key_name('\t'), "Tab");
}

#[test]
fn the_plain_terminal_has_a_size() {
    let [rows, cols, _, _] = Plain.facts();
    assert!(rows > 0 && cols > 0);
}
