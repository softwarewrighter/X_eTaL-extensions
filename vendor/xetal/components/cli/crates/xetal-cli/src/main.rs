//! The `xetal` binary: every pipeline stage exposed as deterministic text.

mod args;
mod context;
mod echo;
mod live;
mod once;
mod stages;

use std::process::ExitCode;

use clap::Parser;

use crate::args::{Cli, Command};
use crate::stages::run;

fn main() -> ExitCode {
    // A closed stdout (`xetal run FILE | head`) ends the run quietly, as
    // it does for cat or grep: Rust ignores SIGPIPE by default, which
    // turns every later write into a panic or an error[io]; the default
    // disposition ends the process with the signal instead.
    sigpipe::reset();
    xetal_tty::install(std::sync::Arc::new(xetal_line::Terminal));
    let cli = Cli::parse();
    let draw = cli.draw.clone();
    xetal_grid::set_ascii(cli.ascii);
    xetal_grid::set_boxed(cli.boxed);
    xetal_system::set_flags(cli.cfg.clone());
    let command = command(cli.command, cli.script);
    install_drawing(draw, &command);
    match run(&command) {
        Ok(text) => {
            if !text.is_empty() {
                println!("{text}");
            }
            ExitCode::SUCCESS
        }
        Err(diag) => {
            eprintln!("{diag}");
            ExitCode::FAILURE
        }
    }
}

/// The command to run: a subcommand, or `run` for a bare script.
fn command(command: Option<Command>, script: Option<String>) -> Command {
    match (command, script) {
        // Options such as --draw may come before or after the subcommand
        // (they are global); a bare script cannot go with one.
        (Some(_), Some(script)) => {
            use clap::CommandFactory;
            Cli::command()
                .error(
                    clap::error::ErrorKind::ArgumentConflict,
                    format!("give a subcommand or a script, not both ({script})"),
                )
                .exit()
        }
        (Some(command), None) => command,
        (None, Some(file)) => Command::Run {
            file,
            untyped: false,
            seed: None,
            echo: false,
            delay: None,
            context: None,
        },
        (None, None) => {
            use clap::CommandFactory;
            Cli::command()
                .error(
                    clap::error::ErrorKind::MissingSubcommand,
                    "give a subcommand or a script FILE",
                )
                .exit()
        }
    }
}

/// Pictures shown with `[]S_HOW` go to numbered files named after the
/// program (`life.xtl` draws `life-1.svg`, ...; `-e` text draws
/// `eval-1.svg`) in `--draw DIR`, else `XETAL_DRAW`, else the current
/// directory; each path written is reported on stderr.
fn install_drawing(draw: Option<String>, command: &Command) {
    let dir = draw
        .or_else(|| std::env::var("XETAL_DRAW").ok())
        .unwrap_or_else(|| ".".into());
    let file = match command {
        Command::Run { file, .. } => Some(file.as_str()),
        Command::Eval(args) => args.input.file.as_deref(),
        _ => None,
    };
    let stem = file
        .and_then(|f| std::path::Path::new(f).file_stem())
        .map_or_else(
            || command.stage().to_string(),
            |s| s.to_string_lossy().into_owned(),
        );
    let notify = |path: &std::path::Path| eprintln!("drawn {}", path.display());
    let store = xetal_store::Drawing::new(dir, &stem, notify);
    xetal_store::install(std::sync::Arc::new(store));
}
