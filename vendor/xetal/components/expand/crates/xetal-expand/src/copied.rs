//! Where the text a macro gives was written: a run of it copied from
//! one of the call's arguments maps back into that argument (so an
//! error in code written inside a string is reported there); the rest,
//! the macro's own text, maps to the whole call.

use xetal_base::Span;
use xetal_mapped::Mapped;

/// Shortest run of an argument taken as copied from it.
const RUN: usize = 2;

/// `text` mapped: runs copied from `args` (longest first, left to
/// right) to where they were written, the rest to the call `whole`.
pub(crate) fn copied(text: &str, args: &[&Mapped], whole: Span) -> Mapped {
    let (mut out, mut at, mut glue) = (Mapped::default(), 0, 0);
    while at < text.len() {
        match longest(&text[at..], args) {
            Some((arg, start, len)) => {
                if glue < at {
                    out.glue(&text[glue..at], whole.start..whole.end);
                }
                out.push(&args[arg].slice(start..start + len));
                at += len;
                glue = at;
            }
            None => at += text[at..].chars().next().map_or(1, char::len_utf8),
        }
    }
    if glue < text.len() {
        out.glue(&text[glue..], whole.start..whole.end);
    }
    out
}

/// The longest prefix of `text` found in an argument, at least [`RUN`]
/// bytes: which argument, where, how long.
fn longest(text: &str, args: &[&Mapped]) -> Option<(usize, usize, usize)> {
    let mut best: Option<(usize, usize, usize)> = None;
    for (k, arg) in args.iter().enumerate() {
        let source = arg.text();
        for (start, _) in source.char_indices() {
            let len = common(text, &source[start..]);
            if len >= RUN && best.is_none_or(|(_, _, b)| len > b) {
                best = Some((k, start, len));
            }
        }
    }
    best
}

/// The length of the common prefix of `a` and `b`, at a character
/// boundary of both.
fn common(a: &str, b: &str) -> usize {
    let n = a.bytes().zip(b.bytes()).take_while(|(x, y)| x == y).count();
    (0..=n)
        .rev()
        .find(|&i| a.is_char_boundary(i) && b.is_char_boundary(i))
        .unwrap_or(0)
}
