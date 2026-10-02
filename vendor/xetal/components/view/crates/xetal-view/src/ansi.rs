//! Segments as ANSI-colored terminal text (`xetal render --color`).

use crate::{Class, Segment};

const RESET: &str = "\u{1b}[0m";

/// The SGR color of a class; whitespace is left plain.
fn color(class: Class) -> Option<&'static str> {
    Some(match class {
        Class::Builtin => "34",
        Class::Macro => "1;33",
        Class::UserFunc => "32",
        Class::LibFunc => "36",
        Class::LambdaArg => "35",
        Class::Number | Class::Exponent => "33",
        Class::String => "93",
        Class::Symbol => "94",
        Class::Quote => "1",
        Class::Comment => "2",
        Class::Error => "31;4",
        Class::Variable | Class::Punct | Class::Unit | Class::Space => return None,
    })
}

/// The segments with each class colored, ending in a reset; runs of
/// one class share one color code.
pub fn ansi(segments: &[Segment]) -> String {
    let mut out = String::new();
    let mut i = 0;
    while i < segments.len() {
        let class = segments[i].class;
        let run: String = segments[i..]
            .iter()
            .take_while(|s| s.class == class)
            .map(|s| s.text.as_str())
            .collect();
        i += segments[i..]
            .iter()
            .take_while(|s| s.class == class)
            .count();
        match color(class) {
            Some(c) if !run.is_empty() => out.push_str(&format!("\u{1b}[{c}m{run}{RESET}")),
            _ => out.push_str(&run),
        }
    }
    if !out.ends_with(RESET) {
        out.push_str(RESET);
    }
    out
}
