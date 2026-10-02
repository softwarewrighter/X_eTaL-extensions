//! Evaluator tests (one test binary so helpers are shared).

mod axes_props;
mod errors;
mod events;
mod lambdas;
mod order;
mod power_props;
mod props;
mod scalar;
mod trains_props;

use xetal_eval::eval_source;

/// The printed output of `src`, which must evaluate without error.
pub fn run(src: &str) -> String {
    let mut out = Vec::new();
    let (_, result) = eval_source(src, &mut out);
    if let Err(e) = result {
        panic!(
            "{src:?} should evaluate, got {e:?}; output so far {:?}",
            String::from_utf8_lossy(&out)
        );
    }
    String::from_utf8(out)
        .expect("utf-8")
        .trim_end()
        .to_string()
}

/// The error code for `src`, which must fail.
pub fn fails(src: &str) -> String {
    let mut out = Vec::new();
    match eval_source(src, &mut out).1 {
        Ok(()) => panic!(
            "{src:?} should fail, printed {:?}",
            String::from_utf8_lossy(&out)
        ),
        Err(e) => e.code,
    }
}

/// Warning codes reported for `src`.
pub fn warnings(src: &str) -> Vec<String> {
    let mut out = Vec::new();
    eval_source(src, &mut out)
        .0
        .into_iter()
        .map(|w| w.code)
        .collect()
}
