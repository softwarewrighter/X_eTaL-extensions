//! The view model: styled segments of decorated source.

use proptest::prelude::*;
use xetal_view::{Class, ansi, column, lines, view};

fn classes(src: &str) -> Vec<(String, Class)> {
    view(src)
        .into_iter()
        .filter(|s| s.class != Class::Space)
        .map(|s| (s.text, s.class))
        .collect()
}

#[test]
fn tokens_get_decorated_text_and_a_class() {
    let got = classes("u:s_quare := { _r * _r } # sq");
    let want = [
        ("\u{1d58}s\u{332}quare", Class::UserFunc),
        ("\u{2190}", Class::Punct),
        ("{", Class::Punct),
        ("\u{2375}", Class::LambdaArg),
        ("\u{d7}", Class::Symbol),
        ("\u{2375}", Class::LambdaArg),
        ("}", Class::Punct),
        ("\u{235d}", Class::Comment),
        (" sq", Class::Comment),
    ];
    let want: Vec<(String, Class)> = want.iter().map(|(t, c)| (t.to_string(), *c)).collect();
    assert_eq!(got, want);
}

#[test]
fn builtins_library_names_numbers_and_strings_differ() {
    let got = classes("'+ r_/ c:K_ x^2 \"hi\" 3.5 @");
    let kinds: Vec<Class> = got.iter().map(|(_, c)| *c).collect();
    assert_eq!(
        kinds,
        [
            Class::Symbol,
            Class::Symbol,
            Class::Builtin,
            Class::LibFunc,
            Class::Variable,
            Class::Exponent,
            Class::String,
            Class::Number,
            Class::Unit
        ]
    );
}

#[test]
fn a_quote_takes_the_class_of_the_function_it_quotes() {
    let got = classes("'+ r_/ 'r_/ 'u:p_lus 'c:K_ '{ _r } '[n_eg]");
    let kinds: Vec<Class> = got.iter().map(|(_, c)| *c).collect();
    use Class::*;
    let want = [
        Symbol, Symbol, Builtin, Builtin, Builtin, UserFunc, UserFunc, LibFunc, LibFunc, Quote,
        Punct, LambdaArg, Punct, Quote, Punct, Builtin, Punct,
    ];
    assert_eq!(kinds, want);
}

#[test]
fn a_power_takes_the_class_of_its_function() {
    let got = classes("n_eg^3 5 'u:d_^2 c:K_^0 x^2");
    let kinds: Vec<Class> = got.iter().map(|(_, c)| *c).collect();
    use Class::*;
    let want = [
        Builtin, Builtin, Number, UserFunc, UserFunc, UserFunc, LibFunc, LibFunc, Variable,
        Exponent,
    ];
    assert_eq!(kinds, want);
}

#[test]
fn symbols_and_their_quotes_are_colored() {
    let out = ansi(&view("'+"));
    assert!(out.starts_with("\u{1b}[94m'+\u{1b}[0m"), "{out:?}");
}

#[test]
fn macros_have_their_own_class_and_color() {
    let got = classes("\"c:\" u_se< \"Combinators\"");
    assert_eq!(got[1].1, Class::Macro);
    assert!(ansi(&view("u_se<")).contains("\u{1b}[1;33m"));
}

#[test]
fn an_import_is_drawn_as_its_alias_bound_to_the_macro() {
    assert_eq!(
        shown("\"s:\" u_se< \"Stats\""),
        "\u{2e2}\u{207c}u\u{332}se< \"Stats\""
    );
    let got = classes("\"s:\" u_se< \"Stats\"");
    assert_eq!(got[0].1, Class::LibFunc);
    assert_eq!(
        shown("\"s\" u_se< \"Stats\""),
        "\"s\" u\u{332}se< \"Stats\"",
        "not an alias: as typed"
    );
}

#[test]
fn invalid_text_is_kept_and_marked() {
    let got = classes("3-1 x");
    assert_eq!(got[1], ("-".to_string(), Class::Error));
    assert_eq!(got.last().unwrap(), &("x".to_string(), Class::Variable));
}

#[test]
fn columns_follow_the_rendered_width() {
    let segs = view("x^2 + y");
    assert_eq!(column(&segs, 4), 3);
    assert_eq!(column(&segs, 6), 5);
    assert_eq!(column(&view("r_ev x"), 5), 4);
}

#[test]
fn lines_split_at_newlines() {
    let ls = lines(&view("a := 1\nb := 2"));
    assert_eq!(ls.len(), 2);
    assert_eq!(ls[1][0].text, "b");
}

#[test]
fn ansi_colors_each_class_and_resets() {
    let out = ansi(&view("r_ev x"));
    assert!(out.contains("\u{1b}["), "{out:?}");
    assert!(out.ends_with("\u{1b}[0m"), "{out:?}");
}

fn shown(src: &str) -> String {
    view(src).into_iter().map(|s| s.text).collect()
}

#[test]
fn trailing_comments_keep_their_source_column() {
    let src = "x := 1        # one\nu:s_quare := { _r * _r }  # two";
    let out = shown(src);
    let lines: Vec<&str> = out.lines().collect();
    let at = |l: &str| {
        l.chars()
            .take_while(|c| *c != '\u{235d}')
            .filter(|c| *c != '\u{332}')
            .count()
    };
    assert_eq!(at(lines[0]), 14, "{out}");
    assert_eq!(at(lines[1]), 26, "{out}");
    assert_eq!(shown("x   # a"), "x   \u{235d} a");
}

#[test]
fn a_comment_keeps_its_column_after_code_drawn_shorter() {
    // x^0.5 is drawn one column shorter (a raised decimal exponent), so
    // one space is added to keep the comment where it was typed; a
    // comment typed touching the code stays touching.
    assert_eq!(shown("x^0.5 # c"), "x\u{2070}\u{b7}\u{2075}  \u{235d} c");
    assert_eq!(shown("x#c"), "x\u{235d}c");
}

#[test]
fn backquoted_code_in_comments_is_decorated() {
    assert_eq!(
        shown("# `:=` binds; `;` separates"),
        "\u{235d} \u{2190} binds; \u{25c6} separates"
    );
    let segs = view("# see `r_ev x`");
    assert!(
        segs.iter()
            .any(|s| s.class == Class::Builtin && s.text == "r\u{332}ev")
    );
    assert_eq!(shown("# a `lone backquote"), "\u{235d} a `lone backquote");
}

proptest! {
    #[test]
    fn segments_cover_any_text_in_order(src in "\\PC{0,40}") {
        let segs = view(&src);
        let mut at = 0;
        for s in &segs {
            prop_assert_eq!(s.raw.start, at);
            prop_assert!(s.raw.end > s.raw.start);
            at = s.raw.end;
        }
        prop_assert_eq!(at, src.len());
    }

    #[test]
    fn valid_source_renders_as_decorate(src in "[a-z_ :=0-9^+*{}();'\"-]{0,30}") {
        if let Ok(want) = xetal_render::decorate(&src) {
            let got: String = view(&src).into_iter().map(|s| s.text).collect();
            prop_assert_eq!(got, want);
        }
    }
}
