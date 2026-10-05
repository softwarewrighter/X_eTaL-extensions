//! Typing a long strand of Int literals takes time linear in its length
//! (ask D1 from X_eTaL-demos: 8000 Ints took two seconds, quadratic).

use std::time::{Duration, Instant};

use xetal_types::check_source;

fn strand(n: usize) -> String {
    (0..n)
        .map(|i| (i % 97).to_string())
        .collect::<Vec<_>>()
        .join(" ")
}

#[test]
fn a_long_int_strand_types_quickly() {
    let src = strand(20_000);
    let start = Instant::now();
    let lines = check_source(&src).expect("a strand of Ints type-checks");
    assert_eq!(lines, vec!["Int".to_string()]);
    assert!(
        start.elapsed() < Duration::from_secs(2),
        "took {:?}",
        start.elapsed()
    );
}

#[test]
fn a_long_mixed_strand_is_still_float() {
    let src = format!("{} 2.5", strand(5_000));
    assert_eq!(
        check_source(&src).expect("type-checks"),
        vec!["Float".to_string()]
    );
}
