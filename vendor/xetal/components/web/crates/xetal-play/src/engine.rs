//! Checking and running a program as the editor's panes show it.

use xetal_base::Diagnostic;
use xetal_macro::StoreLibraries;
use xetal_program::{
    Loaded, in_program, is_library, library_types, load_library_with, load_with, program_types,
};

/// What a run printed, its warnings and error (one per line), and the
/// pictures it showed (SVG documents, `[]S_HOW`), in order.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Run {
    pub out: String,
    pub err: String,
    pub pictures: Vec<String>,
}

/// The name the program is reported by.
const NAME: &str = "main.xtl";

fn loaded(src: &str) -> Result<Loaded, Diagnostic> {
    load_with(NAME, src, &StoreLibraries)
}

/// The type of each top-level item, or the first error; for a library
/// (a file naming `l:`), each export's type, as `xetal type` gives.
pub fn check(src: &str) -> Vec<String> {
    let library = is_library(src);
    let loaded = match library {
        true => load_library_with(NAME, src, &StoreLibraries),
        false => loaded(src),
    };
    let checked = loaded.and_then(|mut l| {
        let lines = xetal_types::check_program(&mut l.program);
        let lines = lines.map_err(|d| in_program(&l.sources, d))?;
        Ok(match library {
            true => library_types(&l.sources, lines),
            false => program_types(lines),
        })
    });
    checked.unwrap_or_else(|d| vec![d.to_string()])
}

/// Check and run `src`, rolling from `seed`; a library has nothing to
/// run, so its exports' types are its output.
pub fn run(src: &str, seed: u64) -> Run {
    let mut out = Vec::new();
    let run = run_to(src, seed, &mut out);
    Run {
        out: String::from_utf8_lossy(&out).into_owned() + &run.out,
        ..run
    }
}

/// Run `src`, writing its output to `out` as it is printed (the Run's
/// `out` holds only a library's types, which are not printed).
pub fn run_to(src: &str, seed: u64, out: &mut (dyn std::io::Write + Send)) -> Run {
    if is_library(src) {
        let lines = check(src);
        return Run {
            out: lines.iter().map(|l| format!("{l}\n")).collect(),
            ..Run::default()
        };
    }
    let loaded = match ready(src) {
        Ok(l) => l,
        Err(run) => return run,
    };
    xetal_store::take_shown();
    let (warnings, result) = xetal_eval::eval_program(&loaded.program, out, Some(seed));
    finish(&loaded, warnings, result)
}

/// The program loaded and type-checked, or the run that failed doing so.
pub(crate) fn ready(src: &str) -> Result<Loaded, Run> {
    let mut loaded = loaded(src).map_err(failed)?;
    match xetal_types::check_program(&mut loaded.program) {
        Ok(_) => Ok(loaded),
        Err(d) => Err(failed(in_program(&loaded.sources, d))),
    }
}

/// A run's end: its warnings and error located in the program, and the
/// pictures it showed.
pub(crate) fn finish(
    loaded: &Loaded,
    warnings: Vec<Diagnostic>,
    result: Result<(), Diagnostic>,
) -> Run {
    let mut err: Vec<String> = warnings.into_iter().map(|w| w.to_string()).collect();
    err.extend(
        result
            .err()
            .map(|e| in_program(&loaded.sources, e).to_string()),
    );
    Run {
        err: err.iter().map(|l| format!("{l}\n")).collect(),
        pictures: xetal_store::take_shown(),
        ..Run::default()
    }
}

fn failed(d: Diagnostic) -> Run {
    Run {
        err: format!("{d}\n"),
        ..Run::default()
    }
}
