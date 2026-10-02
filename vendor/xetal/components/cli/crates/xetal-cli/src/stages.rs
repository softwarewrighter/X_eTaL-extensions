//! Running the pipeline as far as a command asks.

use xetal_base::Diagnostic;

use crate::args::{Command, RenderArgs};

pub(crate) fn read_input(expr: Option<&str>, file: Option<&str>) -> Result<String, Diagnostic> {
    match (expr, file) {
        (Some(expr), _) => Ok(expr.to_string()),
        (None, Some(path)) => std::fs::read_to_string(path)
            .map_err(|e| Diagnostic::new("io", format!("cannot read {path}: {e}"))),
        (None, None) => Err(Diagnostic::new(
            "no-input",
            "give source with -e EXPR or a FILE path",
        )),
    }
}

/// Run the pipeline as far as `command` asks. Every stage runs the
/// earlier ones first, so an early error is reported by any command.
pub(crate) fn run(command: &Command) -> Result<String, Diagnostic> {
    if matches!(command, Command::Repl) {
        xetal_repl::stdio().map_err(|e| Diagnostic::new("io", e.to_string()))?;
        return Ok(String::new());
    }
    if let Command::Edit { file } = command {
        use std::io::IsTerminal;
        if !(std::io::stdin().is_terminal() && std::io::stdout().is_terminal()) {
            return Err(Diagnostic::new(
                "no-terminal",
                "xetal edit needs a terminal",
            ));
        }
        xetal_edit::run(std::path::Path::new(file))
            .map_err(|e| Diagnostic::new("io", e.to_string()))?;
        return Ok(String::new());
    }
    let Some(source) = command.source() else {
        return Err(Diagnostic::unsupported(command.stage()));
    };
    let source = source?;
    if let Command::Render(args) = command {
        return render(args, &source);
    }
    if let Command::Diagram(_) = command {
        return xetal_diagram::diagram(&source);
    }
    if let Some(result) = crate::echo::evaluation(command, &source) {
        return result;
    }
    let tokens = xetal_lex::lex(&source)?;
    match command {
        Command::Lex(_) => Ok(tokens
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join("\n")),
        Command::Parse(_) => Ok(xetal_syntax::parse(&source)?.to_string()),
        Command::Fmt(_) => xetal_render::canonical(&source),
        Command::Core(_) => Ok(xetal_core::lower(&source)?.to_string()),
        _ => Err(Diagnostic::unsupported(command.stage())),
    }
}

fn render(args: &RenderArgs, source: &str) -> Result<String, Diagnostic> {
    if args.raw {
        let raw = xetal_render::undecorate(source)?;
        xetal_lex::lex(&raw)?;
        Ok(raw)
    } else if args.color {
        Ok(xetal_view::ansi(&xetal_view::view(source)))
    } else if args.html {
        Ok(xetal_view::html(&xetal_view::view(source)))
    } else if args.latex {
        xetal_render::latex(source)
    } else {
        xetal_render::decorate(source)
    }
}

/// The `--seed` given, else `XETAL_SEED`, else none (unpredictable).
pub(crate) fn seed_or_env(seed: Option<u64>) -> Result<Option<u64>, Diagnostic> {
    match (seed, std::env::var("XETAL_SEED")) {
        (Some(s), _) => Ok(Some(s)),
        (None, Ok(text)) => text.trim().parse().map(Some).map_err(|_| {
            Diagnostic::new(
                "bad-seed",
                format!("XETAL_SEED must be a whole number, got {text:?}"),
            )
        }),
        (None, Err(_)) => Ok(None),
    }
}

/// Type-check (unless `untyped`), then evaluate, streaming results to
/// stdout; warnings go to stderr.
pub(crate) fn evaluate(
    source: &str,
    name: &str,
    untyped: bool,
    seed: Option<u64>,
) -> Result<String, Diagnostic> {
    let xetal_program::Loaded {
        sources,
        mut program,
    } = xetal_program::load(name, source)?;
    let at = |d| xetal_program::located(&sources, d);
    if !untyped {
        xetal_types::check_program(&mut program).map_err(at)?;
    }
    let mut stdout = std::io::stdout();
    let (warnings, result) = xetal_eval::eval_program(&program, &mut stdout, seed);
    for warning in warnings {
        eprintln!("{}", at(warning));
    }
    result.map(|()| String::new()).map_err(at)
}
