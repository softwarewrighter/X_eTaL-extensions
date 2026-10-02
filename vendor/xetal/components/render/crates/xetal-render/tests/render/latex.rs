//! Raw ASCII -> LaTeX math (one way, for post-processing).

use xetal_render::latex;

fn tex(src: &str) -> String {
    latex(src).unwrap_or_else(|e| panic!("{src:?}: {e:?}"))
}

#[test]
fn names() {
    assert_eq!(tex("x"), r"{\mathrm{x}}");
    assert_eq!(tex("count!"), r"{\mathrm{count}!}");
    assert_eq!(tex("m:pi"), r"{{}^{\mathrm{m}}\mathrm{pi}}");
    assert_eq!(tex("r_ev"), r"{\mathrm{\underline{r}ev}}");
    assert_eq!(
        tex("u:s_quare"),
        r"{{}^{\mathrm{u}}\mathrm{\underline{s}quare}}"
    );
    assert_eq!(tex("r_/"), r"{\mathrm{\underline{r}}{/}}");
}

/// Axes subscript the whole name, never a bare mark: `-_{12}` is
/// rejected by KaTeX and LaTeX checkers.
#[test]
fn axes_anchor_on_the_whole_name() {
    assert_eq!(tex("o_-_12"), r"{{\mathrm{\underline{o}}{-}}_{12}}");
    assert_eq!(tex("r_/_2"), r"{{\mathrm{\underline{r}}{/}}_{2}}");
    assert_eq!(tex("s_\\_1"), r"{{\mathrm{\underline{s}}{\backslash}}_{1}}");
    assert_eq!(tex("r_ev_2"), r"{{\mathrm{\underline{r}ev}}_{2}}");
}

/// An exponent attaches to the token it touches, never to an empty
/// group.
#[test]
fn exponents_anchor_on_what_they_touch() {
    assert_eq!(
        tex("o_-_12 x^2"),
        r"{{\mathrm{\underline{o}}{-}}_{12}}\ {\mathrm{x}}^{2}"
    );
    assert_eq!(tex("(1 9)^0.5"), r"{(}{1}\ {9}{)}^{0.5}");
    assert_eq!(tex("n_eg^3 5"), r"{\mathrm{\underline{n}eg}}^{3}\ {5}");
}

#[test]
fn exponents_and_symbols() {
    assert_eq!(tex("x^0.5"), r"{\mathrm{x}}^{0.5}");
    assert_eq!(
        tex("* / | & != <= >="),
        r"{\times}\ {\div}\ {\vee}\ {\wedge}\ {\neq}\ {\leq}\ {\geq}"
    );
    assert_eq!(tex("3 -1"), r"{3}\ {-1}");
}

#[test]
fn punctuation_and_whitespace() {
    assert_eq!(
        tex("{ _l ; _r_ }"),
        r"{\{}\ {\_\mathrm{l}}\ {\diamond}\ {\_\mathrm{r}\_}\ {\}}"
    );
    assert_eq!(tex("x := y"), r"{\mathrm{x}}\ {\leftarrow}\ {\mathrm{y}}");
    assert_eq!(tex("a\nb"), "{\\mathrm{a}}\\\\\n{\\mathrm{b}}");
}

/// Space before a comment (which is dropped) or the end of a line sets
/// nothing, so none is written: no line ends in a lone `\ `.
#[test]
fn no_space_before_a_dropped_comment_or_a_line_end() {
    assert_eq!(
        tex("x := 5        # five"),
        r"{\mathrm{x}}\ {\leftarrow}\ {5}"
    );
    assert_eq!(tex("a   \nb # c"), "{\\mathrm{a}}\\\\\n{\\mathrm{b}}");
    assert_eq!(tex("  a"), r"\ \ {\mathrm{a}}");
}

/// A string is set as text as it is spelled in the source (`\\` is two
/// backslashes), its TeX specials escaped, and an underlined
/// letter (a combining underline) as `\underline`, which KaTeX accepts
/// where it rejects the combining character.
#[test]
fn strings_are_escaped_text() {
    assert_eq!(
        tex("\"a_b #1 {x} $ % & ~ ^ \\\\\""),
        r#"{\text{"a\_b \#1 \{x\} \$ \% \& \textasciitilde{} \textasciicircum{} \textbackslash{}\textbackslash{}"}}"#
    );
    assert_eq!(
        tex("\"hi X\u{332}\u{1d49}T\""),
        "{\\text{\"hi \\underline{X}\u{1d49}T\"}}"
    );
}
