//! The page's program, run as the page runs it (natively here, where
//! nothing is drawn): the city of two seeds, each the same number of
//! calls; and the film's seed gives the calls the command line gives.

use three_web::{about, build, install, library, program, rendered};
use xetal_ext_shell::run::run;

#[test]
fn a_seed_builds_a_city() {
    assert_eq!(install(), 3);
    let o = run(&program(7));
    assert_eq!(o.err, "");
    assert_eq!(o.out.trim(), "84");
    assert!(build(7).unwrap().starts_with("seed 7: 84 calls"));
    assert!(build(8).unwrap().starts_with("seed 8: 84 calls"));
    // drawn decorated: the underlined letter of a function, in spans
    let shown = rendered(7);
    assert!(shown.contains("<span class=\"t-"));
    assert!(!shown.contains("u_se"));
    // the About dialog's facts and sources
    let facts = about();
    assert!(facts.contains("MIT License") && facts.contains("class=\"stamp\""));
    assert!(!facts.contains("build") && !facts.contains("sha "));
    assert!(library("NeonCity").contains("<span class=\"t-"));
    assert_eq!(library("nothing"), "");
}
