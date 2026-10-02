//! Decorated Unicode -> raw ASCII. ASCII passes through unchanged, so
//! anything shown raw (e.g. `x^0.5`, `q:x`) inverts trivially. A string
//! or a comment is copied as it is (the drawing never touches either),
//! so both may hold any Unicode.

use xetal_base::{Diagnostic, Span};

use crate::glyphs::{
    LIGATURES, RAISED_POINT, UNDERLINE, from_subscript, from_superscript, from_superscript_letter,
};
use crate::lambda::from_lambda_glyph;

#[derive(Clone, Copy, PartialEq, Eq)]
enum After {
    Other,
    Sub,
    Super,
    Namespace,
}

struct Inverse {
    out: String,
    after: After,
    /// Inside a string, just after a `\` in one, and inside a comment:
    /// strings and comments are copied as they are (the drawing never
    /// touches them), so they may hold any Unicode.
    string: bool,
    escaped: bool,
    comment: bool,
}

/// Convert decorated text back to raw ASCII source.
pub fn undecorate(text: &str) -> Result<String, Diagnostic> {
    let mut inv = Inverse {
        out: String::new(),
        after: After::Other,
        string: false,
        escaped: false,
        comment: false,
    };
    let mut chars = text.char_indices().peekable();
    while let Some((i, c)) = chars.next() {
        if inv.string || inv.comment {
            inv.out.push(c);
            inv.comment = inv.comment && c != '\n';
            inv.string = inv.string && (inv.escaped || c != '"');
            inv.escaped = inv.string && !inv.escaped && c == '\\';
            continue;
        }
        let underlined = chars.peek().is_some_and(|&(_, n)| n == UNDERLINE);
        if underlined {
            chars.next();
        }
        let len = c.len_utf8() + if underlined { UNDERLINE.len_utf8() } else { 0 };
        let span = Span::new(i, i + len);
        inv.close_namespace(c, span)?;
        inv.push(c, underlined, span)?;
        inv.string = c == '"';
        inv.comment = inv.out.ends_with('#');
    }
    if inv.after == After::Namespace {
        return Err(bad_namespace(Span::new(text.len(), text.len())));
    }
    Ok(inv.out)
}

impl Inverse {
    /// Superscript letters are a namespace: they must precede a name.
    fn close_namespace(&mut self, c: char, span: Span) -> Result<(), Diagnostic> {
        if self.after == After::Namespace && from_superscript_letter(c).is_none() {
            if !c.is_ascii_alphabetic() {
                return Err(bad_namespace(span));
            }
            self.out.push(':');
            self.after = After::Other;
        }
        Ok(())
    }

    fn push(&mut self, c: char, underlined: bool, span: Span) -> Result<(), Diagnostic> {
        if let Some(side) = from_lambda_glyph(c) {
            self.out.push('_');
            self.out.push(side);
            if underlined {
                self.out.push('_');
            }
            self.after = After::Other;
        } else if underlined {
            if !c.is_ascii_alphabetic() {
                return Err(bad_underline(span));
            }
            self.out.push(c);
            self.out.push('_');
            self.after = After::Other;
        } else if c == RAISED_POINT && self.after == After::Super {
            self.out.push('.');
        } else if let Some(d) = from_subscript(c) {
            self.run('_', After::Sub, d);
        } else if let Some(d) = from_superscript(c) {
            self.run('^', After::Super, d);
        } else if let Some(l) = from_superscript_letter(c) {
            self.out.push(l);
            self.after = After::Namespace;
        } else if let Some((ascii, _)) = LIGATURES.iter().find(|(_, g)| *g == c) {
            self.out.push_str(ascii);
            self.after = After::Other;
        } else {
            self.out.push(plain(c, span)?);
            self.after = After::Other;
        }
        Ok(())
    }

    /// A run of subscript or superscript glyphs shares one `_` or `^`.
    fn run(&mut self, prefix: char, state: After, c: char) {
        if self.after != state {
            self.out.push(prefix);
        }
        self.out.push(c);
        self.after = state;
    }
}

fn plain(c: char, span: Span) -> Result<char, Diagnostic> {
    if c == UNDERLINE {
        return Err(bad_underline(span));
    }
    if !c.is_ascii() {
        return Err(
            Diagnostic::new("not-decorated", "not ASCII and not a decoration glyph")
                .with_span(span),
        );
    }
    Ok(c)
}

fn bad_underline(span: Span) -> Diagnostic {
    Diagnostic::new("bad-underline", "an underline must be under a letter").with_span(span)
}

fn bad_namespace(span: Span) -> Diagnostic {
    Diagnostic::new(
        "bad-namespace",
        "a superscript namespace must precede a name",
    )
    .with_span(span)
}
