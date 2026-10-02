//! Comments in the view: backquoted code inside a comment is shown
//! decorated (the backquotes hidden), and a comment after code keeps
//! the column it has in the source, so comments stay lined up although
//! the code before them is drawn shorter. `xetal render` does neither,
//! so its output still converts back to the exact source.

use xetal_base::Span;

use crate::{Class, Segment, view, width};

fn seg(start: usize, end: usize, text: &str, class: Class) -> Segment {
    Segment {
        raw: Span::new(start, end),
        text: text.into(),
        class,
    }
}

/// The segments of comment `text` (from `#` to the end of its line),
/// which starts at byte `base`.
pub(crate) fn comment(text: &str, base: usize, out: &mut Vec<Segment>) {
    out.push(seg(base, base + 1, "\u{235d}", Class::Comment));
    let mut at = 1;
    while at < text.len() {
        let open = text[at..].find('`').map(|i| at + i);
        let close = open.and_then(|o| text[o + 1..].find('`').map(|i| o + 1 + i));
        let (Some(o), Some(c)) = (open, close) else {
            out.push(seg(
                base + at,
                base + text.len(),
                &text[at..],
                Class::Comment,
            ));
            return;
        };
        if o > at {
            out.push(seg(base + at, base + o, &text[at..o], Class::Comment));
        }
        out.push(seg(base + o, base + o + 1, "", Class::Comment));
        out.extend(view(&text[o + 1..c]).into_iter().map(|mut s| {
            s.raw = Span::new(s.raw.start + base + o + 1, s.raw.end + base + o + 1);
            s
        }));
        out.push(seg(base + c, base + c + 1, "", Class::Comment));
        at = c + 1;
    }
}

/// Pad or trim the space before each comment that follows code so the
/// comment starts at its source column (keeping at least one space).
pub(crate) fn align(segments: &mut [Segment], src: &str) {
    let mut col = 0;
    for i in 0..segments.len() {
        let s = &segments[i];
        if s.class == Class::Comment
            && s.text.starts_with('\u{235d}')
            && i > 0
            && trailing(&segments[i - 1])
        {
            let line = src[..s.raw.start].rfind('\n').map_or(0, |n| n + 1);
            let want = src[line..s.raw.start].chars().count();
            let gap = segments[i - 1].text.len();
            let len = (gap + want).saturating_sub(col).max(1);
            segments[i - 1].text = " ".repeat(len);
            col = col + len - gap;
        }
        let text = &segments[i].text;
        col = match text.rfind('\n') {
            Some(n) => width(&text[n + 1..]),
            None => col + width(text),
        };
    }
}

/// Spaces between code and a comment on the same line.
fn trailing(s: &Segment) -> bool {
    s.class == Class::Space && !s.text.contains('\n') && !s.text.is_empty()
}
