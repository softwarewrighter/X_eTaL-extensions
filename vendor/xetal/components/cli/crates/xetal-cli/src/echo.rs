//! `xetal run --echo`: each statement pretty-printed (decorated and
//! colored), then its output below it, like a notebook laid out as an
//! APL session: statements indented six spaces, output flush left.

use xetal_base::Diagnostic;

use crate::args::{Command, EvalArgs};
use crate::stages::{evaluate, seed_or_env};
use xetal_view::{ansi, view};

/// Print `source` as a notebook; an error fails the run after
/// everything has been shown.
pub(crate) fn echo(
    source: &str,
    origin: &str,
    seed: Option<u64>,
    delay: Option<u64>,
    untyped: bool,
) -> Result<String, Diagnostic> {
    let seed = seed.unwrap_or_else(xetal_eval::Rng::fresh_seed);
    let mut show = |text: &str| {
        pause(delay, text);
        let shown = ansi(&view(text));
        if text.trim().is_empty() {
            println!();
        }
        for line in shown.lines().filter(|_| !text.trim().is_empty()) {
            println!("{}{line}", crate::live::PROMPT);
        }
    };
    let failed = match crate::once::once(origin, source, (seed, untyped), &mut show) {
        Some(failed) => failed,
        None => {
            let mut session = xetal_repl::Session::new(origin, seed);
            session.untyped = untyped;
            crate::live::stream(&mut session, source, &mut show)
        }
    };
    match failed {
        true => Err(Diagnostic::new(
            "failed",
            "a statement failed (shown above)",
        )),
        false => Ok(String::new()),
    }
}

/// With `--delay`, wait before showing a statement that is not blank,
/// output shown so far flushed first (for recording a run as it goes).
fn pause(delay: Option<u64>, source: &str) {
    if let Some(ms) = delay.filter(|_| !source.trim().is_empty()) {
        let _ = std::io::Write::flush(&mut std::io::stdout());
        std::thread::sleep(std::time::Duration::from_millis(ms));
    }
}

/// `eval` and `run`, plain or as a notebook (a library run lists its
/// exports' types); other commands are not evaluations.
pub(crate) fn evaluation(command: &Command, source: &str) -> Option<Result<String, Diagnostic>> {
    // A library is not run: running one lists its exports, as `type`.
    if matches!(command, Command::Run { .. }) && xetal_program::is_library(source) {
        return Some(typed(source, &origin(command)));
    }
    Some(match command {
        Command::Run {
            context: Some(path),
            seed,
            ..
        } => seed_or_env(*seed)
            .and_then(|seed| crate::context::after_context(path, &origin(command), source, seed)),
        Command::Eval(EvalArgs {
            echo: true,
            seed,
            delay,
            untyped,
            ..
        })
        | Command::Run {
            echo: true,
            seed,
            delay,
            untyped,
            ..
        } => seed_or_env(*seed)
            .and_then(|seed| echo(source, &origin(command), seed, *delay, *untyped)),
        Command::Eval(EvalArgs { untyped, seed, .. }) | Command::Run { untyped, seed, .. } => {
            seed_or_env(*seed).and_then(|seed| evaluate(source, &origin(command), *untyped, seed))
        }
        Command::Type(_) => typed(source, &origin(command)),
        _ => return None,
    })
}

/// The name a program is reported by, and beside which its libraries
/// are looked for: its file, or `-e` for text on the command line.
fn origin(command: &Command) -> String {
    match command {
        Command::Run { file, .. } => file.clone(),
        Command::Eval(EvalArgs { input, .. }) | Command::Type(input) => {
            input.file.clone().unwrap_or_else(|| "-e".into())
        }
        _ => "-e".into(),
    }
}

/// `xetal type`: the type of each top-level item of a program, or of
/// each export of a library (a file naming `l:`).
fn typed(source: &str, name: &str) -> Result<String, Diagnostic> {
    let library = xetal_program::is_library(source);
    let mut loaded = match library {
        true => xetal_program::load_library(name, source)?,
        false => xetal_program::load(name, source)?,
    };
    let lines = xetal_types::check_program(&mut loaded.program)
        .map_err(|d| xetal_program::located(&loaded.sources, d))?;
    Ok(match library {
        true => xetal_program::library_types(&loaded.sources, lines),
        false => xetal_program::program_types(lines),
    }
    .join("\n"))
}
