//! The built-in enumerated types' values (QD6), by index: `Color` (the
//! eight colours) and `Key` (the named keys, then a printing key as
//! `PRINTING` plus its code point).

/// The colours, index 0 to 7 (ANSI colour order).
pub const COLORS: [&str; 8] = [
    "BLACK", "RED", "GREEN", "YELLOW", "BLUE", "MAGENTA", "CYAN", "WHITE",
];

/// The named keys, index 0 to 10.
pub const KEYS: [&str; 11] = [
    "UP",
    "DOWN",
    "LEFT",
    "RIGHT",
    "ENTER",
    "ESCAPE",
    "BACKSPACE",
    "TAB",
    "DELETE",
    "HOME",
    "END",
];

/// A printing key's index: this plus its character's code point.
const PRINTING: u32 = 0x100;

/// The name a value of `ty` with index `i` prints as: its name, or a
/// printing key's character.
pub fn name(ty: &str, i: u32) -> String {
    let table: &[&str] = if ty == "Color" { &COLORS } else { &KEYS };
    match table.get(i as usize) {
        Some(n) => (*n).to_string(),
        None if ty == "Key" => {
            char::from_u32(i.saturating_sub(PRINTING)).map_or_else(String::new, String::from)
        }
        None => format!("{ty}?{i}"),
    }
}

/// The Key index for a key as a terminal names it (`Up`, `Enter`, or
/// one character), if it is one.
pub fn key_named(name: &str) -> Option<u32> {
    if let Some(i) = KEYS.iter().position(|k| k.eq_ignore_ascii_case(name)) {
        return Some(i as u32);
    }
    let mut chars = name.chars();
    match (chars.next(), chars.next()) {
        (Some(c), None) => Some(PRINTING + c as u32),
        _ => None,
    }
}
