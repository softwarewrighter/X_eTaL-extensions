//! The ANSI sequences the screen functions write (QD6): `ESC [ r ; c H`
//! places the cursor, `ESC [ 2 J` clears, `ESC [ ... m` sets bold and
//! colours; any other sequence is read and ignored.

use std::iter::Peekable;

use crate::cell::{Color, Style};

/// A sequence a grid obeys.
pub(crate) enum Sequence {
    Place(usize, usize),
    Clear,
    Style(Vec<u32>),
}

/// The sequence after an `ESC` (its `[`, numbers and final letter).
pub(crate) fn read(chars: &mut Peekable<impl Iterator<Item = char>>) -> Option<Sequence> {
    chars.next_if_eq(&'[')?;
    let mut body = String::new();
    let last = loop {
        match chars.next()? {
            c if c.is_ascii_alphabetic() => break c,
            c => body.push(c),
        }
    };
    let nums: Vec<u32> = body.split(';').map(|n| n.parse().unwrap_or(0)).collect();
    match last {
        'H' => Some(Sequence::Place(
            nums.first().copied().filter(|n| *n > 0).unwrap_or(1) as usize,
            nums.get(1).copied().filter(|n| *n > 0).unwrap_or(1) as usize,
        )),
        'J' => Some(Sequence::Clear),
        'm' => Some(Sequence::Style(nums)),
        _ => None,
    }
}

const PALETTE: [Color; 8] = [
    Color::Black,
    Color::Red,
    Color::Green,
    Color::Yellow,
    Color::Blue,
    Color::Magenta,
    Color::Cyan,
    Color::White,
];

/// `style` with the SGR codes applied (0 resets everything).
pub(crate) fn apply(mut style: Style, codes: &[u32]) -> Style {
    for &code in codes {
        match code {
            0 => style = Style::default(),
            1 => style.bold = true,
            22 => style.bold = false,
            30..=37 => style.fg = PALETTE[(code - 30) as usize],
            39 => style.fg = Color::Default,
            40..=47 => style.bg = PALETTE[(code - 40) as usize],
            49 => style.bg = Color::Default,
            _ => {}
        }
    }
    style
}
