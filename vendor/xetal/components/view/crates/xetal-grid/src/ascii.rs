//! Pictures in plain ASCII, as APL2's DISPLAY drew them on terminals
//! without box characters.

use std::sync::atomic::{AtomicBool, Ordering};

/// Whether pictures are drawn in plain ASCII (`xetal --ascii`), for
/// places that take no box characters.
static ASCII: AtomicBool = AtomicBool::new(false);

/// Draw every later picture in ASCII (true) or with box characters.
pub fn set_ascii(on: bool) {
    ASCII.store(on, Ordering::Relaxed);
}

/// Whether every printed array is drawn boxed, flat ones too
/// (`xetal --box`, the live demo's Boxed): APL2's DISPLAY for all.
static BOXED: AtomicBool = AtomicBool::new(false);

/// Print every later array boxed (true), or only nested ones.
pub fn set_boxed(on: bool) {
    BOXED.store(on, Ordering::Relaxed);
}

pub fn boxed() -> bool {
    BOXED.load(Ordering::Relaxed)
}

/// A line of a picture in ASCII, as APL2's DISPLAY drew it on terminals
/// without box characters: `.` and `'` corners, `-` and `|` sides, `>`
/// and `v` arrows, `e` for boxes, `O` for an empty axis. Each character
/// becomes one, so the picture keeps its shape.
pub fn to_ascii(line: &str) -> String {
    line.chars()
        .map(|c| match c {
            '┌' | '┐' => '.',
            '└' | '┘' => '\'',
            '─' => '-',
            '│' => '|',
            '→' => '>',
            '↓' => 'v',
            '∊' => 'e',
            '⊖' | '⌽' => 'O',
            other => other,
        })
        .collect()
}

/// Whether pictures are drawn in ASCII now.
pub(crate) fn ascii() -> bool {
    ASCII.load(Ordering::Relaxed)
}
