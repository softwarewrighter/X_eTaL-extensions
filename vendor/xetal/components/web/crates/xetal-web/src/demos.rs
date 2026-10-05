//! What Open offers: the demos and the tour, built in; the standard
//! libraries, named as `u_se<` finds them (`Stats.xtl`); and the files
//! saved in the store (local storage, in the browser).

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Demo {
    pub name: &'static str,
    pub text: &'static str,
}

macro_rules! demos {
    ($($name:literal),* $(,)?) => {
        &[
            Demo { name: "tour.xtl", text: include_str!("../../../../../demos/tour.xtl") },
            Demo { name: "(empty)", text: "" },
            $(Demo {
                name: $name,
                text: include_str!(concat!("../../../../../demos/", $name)),
            }),*
        ]
    };
}

/// The first is shown when the page opens; the second is an empty
/// editor, to type into as at a REPL. The rest are listed here in
/// alphabetical order (a test keeps them so), and Open sorts them too.
pub const DEMOS: &[Demo] = demos![
    "arrays.xtl",
    "classics/automaton.xtl",
    "classics/bases.xtl",
    "classics/closure.xtl",
    "classics/collatz.xtl",
    "classics/duck.xtl",
    "classics/factorial.xtl",
    "classics/fibonacci.xtl",
    "classics/gcd.xtl",
    "classics/hanoi.xtl",
    "classics/histogram.xtl",
    "classics/life-drawn.xtl",
    "classics/magic.xtl",
    "classics/mandelbrot.xtl",
    "classics/mastermind-play.xtl",
    "classics/mastermind.xtl",
    "classics/matmul.xtl",
    "classics/mini-apl.xtl",
    "classics/pascal.xtl",
    "classics/primes.xtl",
    "classics/queens.xtl",
    "classics/quicksort.xtl",
    "classics/rle.xtl",
    "classics/roman.xtl",
    "classics/sequences.xtl",
    "classics/shortest.xtl",
    "classics/sieve.xtl",
    "classics/sorting.xtl",
    "classics/truth.xtl",
    "classics/turtle.xtl",
    "classics/wordfreq.xtl",
    "combinators.xtl",
    "factorial.xtl",
    "hello-library.xtl",
    "higher-order.xtl",
    "keys.xtl",
    "leetcode/numbers-in-string.xtl",
    "life.xtl",
    "macros.xtl",
    "magmas.xtl",
    "monads.xtl",
    "stats.xtl",
    "tttml-play.xtl",
    "tttml-train.xtl",
    "tttml.xtl",
];

/// The choices, as (group, value, label), group by group: Demos (the
/// tour, the empty editor, then the top folder's demos), Classics (shown
/// without their folder), Libraries, Misc (any other folder), Your files;
/// each group alphabetical. A value is `demo:N`, `lib:Name` or
/// `file:path`, and [`open`] reads it.
pub fn choices(saved: &[String]) -> Vec<(&'static str, String, String)> {
    let by_label = |mut group: Vec<(&'static str, String, String)>| {
        group.sort_by_key(|(_, _, label)| label.to_lowercase());
        group
    };
    let demo = |(i, d): (usize, &Demo)| match d.name.split_once('/') {
        None => ("Demos", format!("demo:{i}"), d.name.to_string()),
        Some(("classics", name)) => ("Classics", format!("demo:{i}"), name.to_string()),
        Some(_) => ("Misc", format!("demo:{i}"), d.name.to_string()),
    };
    let mut demos: Vec<_> = DEMOS.iter().enumerate().map(demo).collect();
    let rest = demos.split_off(2.min(demos.len()));
    let pick = |g: &str| by_label(rest.iter().filter(|c| c.0 == g).cloned().collect());
    let libs = xetal_libs::LIBRARIES.iter();
    let libs = by_label(
        libs.map(|(n, _)| ("Libraries", format!("lib:{n}"), format!("{n}.xtl")))
            .collect(),
    );
    let files = by_label(
        saved
            .iter()
            .map(|p| ("Your files", format!("file:{p}"), p.clone()))
            .collect(),
    );
    [
        demos,
        pick("Demos"),
        pick("Classics"),
        libs,
        pick("Misc"),
        files,
    ]
    .concat()
}

/// The name and text of a choice.
pub fn open(value: &str) -> Option<(String, String)> {
    match value.split_once(':')? {
        ("demo", i) => DEMOS
            .get(i.parse::<usize>().ok()?)
            .map(|d| (d.name.into(), d.text.into())),
        ("lib", n) => xetal_libs::standard(n).map(|t| (format!("{n}.xtl"), t.into())),
        ("file", p) => xetal_store::read(p).ok().map(|t| (p.to_string(), t)),
        _ => None,
    }
}

/// Put the demos' own libraries among your files, unless they are there
/// already (so your edits are kept): `hello-library.xtl` imports
/// `Hello.xtl` from there, as it would from beside it on disk.
pub fn seed() {
    for (path, text) in OWN_LIBRARIES {
        if xetal_store::read(path).is_err() {
            let _ = xetal_store::write(path, text);
        }
    }
}

/// The libraries the demos import that are not standard ones.
const OWN_LIBRARIES: &[(&str, &str)] = &[
    (
        "Hello.xtl",
        include_str!("../../../../../userlibs/Hello.xtl"),
    ),
    (
        "Greetings.xtl",
        include_str!("../../../../../userlibs/Greetings.xtl"),
    ),
];
