//! `xetal-x`: the vendored `xetal` CLI, unchanged, with one difference:
//! its store sends paths starting `ext:` to native extensions (see
//! `ext.rs`) and everything else where `xetal` sends it. Every
//! subcommand, flag and message is the vendored CLI's: its modules are
//! compiled here from `vendor/xetal/` as they are.
//!
//! Extensions are packages (directories with an `extension.toml`) found
//! through `--ext DIR` (repeatable, before the subcommand) and the
//! colon-separated `XETAL_EXT_PATH`; a directory given is a package or
//! holds packages. Each package's facade directory is added to
//! `XETAL_PATH`, after the user's, so programs import facades by name. Their native libraries are looked for under each
//! package's `native/<platform>/`, then beside this executable (where
//! Cargo builds the workspace's extensions).

// The vendored CLI's modules, verbatim (main.rs below replaces its main).
// rustfmt must not touch them: they live under vendor/.
#[rustfmt::skip]
#[path = "../../../vendor/xetal/components/cli/crates/xetal-cli/src/args.rs"]
mod args;
#[rustfmt::skip]
#[path = "../../../vendor/xetal/components/cli/crates/xetal-cli/src/context.rs"]
mod context;
#[rustfmt::skip]
#[path = "../../../vendor/xetal/components/cli/crates/xetal-cli/src/echo.rs"]
mod echo;
#[rustfmt::skip]
#[path = "../../../vendor/xetal/components/cli/crates/xetal-cli/src/live.rs"]
mod live;
#[rustfmt::skip]
#[path = "../../../vendor/xetal/components/cli/crates/xetal-cli/src/once.rs"]
mod once;
#[rustfmt::skip]
#[path = "../../../vendor/xetal/components/cli/crates/xetal-cli/src/stages.rs"]
mod stages;

mod ext;
mod protocol;

use std::process::ExitCode;

use clap::Parser;

use crate::args::{Cli, Command};
use crate::stages::run;

fn main() -> ExitCode {
    sigpipe::reset();
    let (argv, ext_dirs, list) = match ext::split_args(std::env::args_os()) {
        Ok(split) => split,
        Err(message) => {
            eprintln!("error[ext]: {message}");
            return ExitCode::FAILURE;
        }
    };
    let registry = match ext::load(&ext_dirs) {
        Ok((registry, facades)) => {
            if let Some(path) = ext::library_path(&facades) {
                // SAFETY: no other thread exists yet; the vendored CLI
                // reads XETAL_PATH when a program imports a library.
                #[allow(unsafe_code)]
                unsafe {
                    std::env::set_var("XETAL_PATH", path);
                }
            }
            registry
        }
        Err(diag) => {
            eprintln!("{diag}");
            return ExitCode::FAILURE;
        }
    };
    if list {
        print!("{}", ext::listing(&registry));
        return ExitCode::SUCCESS;
    }
    // From here on, the vendored CLI's main.
    let cli = Cli::parse_from(argv);
    let draw = cli.draw.clone();
    xetal_grid::set_ascii(cli.ascii);
    xetal_grid::set_boxed(cli.boxed);
    let command = match (cli.command, cli.script) {
        (Some(command), _) => command,
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
    };
    install_store(draw, &command, registry);
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

/// The vendored CLI's drawing store (pictures as numbered SVG files),
/// wrapped so `ext:` paths reach the extensions.
fn install_store(draw: Option<String>, command: &Command, registry: xetal_ext_loader::Registry) {
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
    let inner = xetal_store::Drawing::new(dir, &stem, notify);
    xetal_store::install(std::sync::Arc::new(ext::ExtStore::new(inner, registry)));
}
