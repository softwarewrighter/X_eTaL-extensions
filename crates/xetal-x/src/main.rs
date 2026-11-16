//! `xetal-x`: the vendored `xetal` CLI, unchanged, with one difference:
//! its store sends paths starting `ext:` to native extensions (see
//! `ext.rs`) and everything else where `xetal` sends it. Every
//! subcommand, flag and message is the CLI's own: X_eTaL's command line
//! is a library, `xetal-command` (ask E3, landed), used here from
//! X_eTaL's clone, `work/xetal/`; only its binary's short `main` is
//! repeated, with the store wrapped.
//!
//! Extensions are packages (directories with an `extension.toml`) found
//! through `--ext DIR` (repeatable, before the subcommand) and the
//! colon-separated `XETAL_EXT_PATH`; a directory given is a package or
//! holds packages. Each package's facade directory is added to
//! `XETAL_PATH`, after the user's, so programs import facades by name. Their native libraries are looked for under each
//! package's `native/<platform>/`, then beside this executable (where
//! Cargo builds the workspace's extensions).

mod ext;

use std::process::ExitCode;

use xetal_base::Diagnostic;
use xetal_command::{Cli, Command, Host, Local};

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
    // --version: say what this is before the vendored CLI's block
    if argv.get(1).is_some_and(|a| a == "--version" || a == "-V") {
        println!(
            "xetal-x {} (X_eTaL-extensions {}): the vendored X_eTaL CLI below, with native extensions (experimental)",
            env!("CARGO_PKG_VERSION"),
            env!("XETAL_X_SHA")
        );
    }
    // From here on, the vendored CLI's main.
    xetal_tty::install(std::sync::Arc::new(xetal_line::Terminal));
    let argv = argv.into_iter().map(|a| a.to_string_lossy().into_owned());
    let cli = xetal_command::parse(argv).unwrap_or_else(|e| e.exit());
    let draw = cli.draw.clone();
    let events = cli.events.clone();
    xetal_grid::set_ascii(cli.ascii);
    xetal_grid::set_boxed(cli.boxed);
    xetal_system::set_flags(cli.cfg.clone());
    let command = command(cli.command, cli.script);
    install_store(draw, events, &command, registry);
    // The program runs on a thread of its own; this (the main) thread
    // serves windows for extensions like canvas (plan M1), and does
    // nothing at all unless the program opens one.
    let done = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let finished = done.clone();
    let program = std::thread::spawn(move || {
        let (mut out, mut err) = (std::io::stdout(), std::io::stderr());
        let mut host = Host {
            files: &Local,
            out: &mut out,
            err: &mut err,
            seed: None,
        };
        let result = run(&command, &mut host).map_err(|d| placed(d, &command, &host));
        finished.store(true, std::sync::atomic::Ordering::SeqCst);
        result
    });
    xetal_ext_ui::serve(|| done.load(std::sync::atomic::Ordering::SeqCst));
    match program.join() {
        Ok(Ok(text)) => {
            if !text.is_empty() {
                println!("{text}");
            }
            ExitCode::SUCCESS
        }
        Ok(Err(diag)) => {
            eprintln!("{diag}");
            ExitCode::FAILURE
        }
        Err(_) => {
            eprintln!("error[internal]: the program's thread failed");
            ExitCode::FAILURE
        }
    }
}

/// The interactive commands on this terminal; the rest through the
/// library.
fn run(command: &Command, host: &mut Host) -> Result<String, Diagnostic> {
    match command {
        Command::Repl => {
            xetal_repl::stdio().map_err(|e| Diagnostic::new("io", e.to_string()))?;
            Ok(String::new())
        }
        Command::Edit {
            file,
            screen: true,
            keys,
            size,
        } => screen(file, keys.as_deref().unwrap_or(""), size),
        Command::Edit { file, .. } => {
            use std::io::IsTerminal;
            if !(std::io::stdin().is_terminal() && std::io::stdout().is_terminal()) {
                return Err(Diagnostic::new(
                    "no-terminal",
                    "xetal edit needs a terminal",
                ));
            }
            xetal_edit::run(std::path::Path::new(file))
                .map_err(|e| Diagnostic::new("io", e.to_string()))?;
            Ok(String::new())
        }
        _ => xetal_command::run(command, host),
    }
}

/// `xetal edit FILE --screen --keys KEYS --size COLSxROWS`: the
/// editor's screen after the keys, as text.
fn screen(file: &str, keys: &str, size: &str) -> Result<String, Diagnostic> {
    let bad = |m: String| Diagnostic::new("bad-screen", m);
    let size = size
        .split_once('x')
        .and_then(|(c, r)| Some((c.parse::<u16>().ok()?, r.parse::<u16>().ok()?)))
        .filter(|(c, r)| *c > 0 && *r > 0)
        .ok_or_else(|| bad(format!("--size must be COLSxROWS, as 100x32, got {size:?}")))?;
    let keys = xetal_keys::keys(keys).map_err(bad)?;
    let editor = xetal_edit::Editor::open(std::path::Path::new(file))
        .map_err(|e| Diagnostic::new("io", format!("cannot read {file}: {e}")))?;
    Ok(xetal_shell::screen(editor, &keys, size).join("\n"))
}

/// `d` with its file, line and column, if it has a span not yet placed
/// (a stage that reads the source alone: lex, parse, render, ...).
fn placed(d: Diagnostic, command: &Command, host: &Host) -> Diagnostic {
    match (d.place(), d.span, command.source(host)) {
        (None, Some(_), Some(Ok(text))) => d.placed_in(&command.origin(), &text),
        _ => d,
    }
}

/// The command to run: a subcommand, or `run` for a bare script (the
/// vendored CLI's `command`, repeated as its `main` is).
fn command(command: Option<Command>, script: Option<String>) -> Command {
    match (command, script) {
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

/// The vendored CLI's drawing store (pictures as numbered SVG files),
/// wrapped so `ext:` paths reach the extensions.
fn install_store(
    draw: Option<String>,
    events: Option<String>,
    command: &Command,
    registry: xetal_ext_loader::Registry,
) {
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
    // --events FILE: scripted events for the program (as the CLI does)
    let inner = match events.map(std::fs::read_to_string) {
        Some(Ok(lines)) => inner.scripted(&lines),
        Some(Err(e)) => {
            eprintln!("error[io]: --events: {e}");
            std::process::exit(2);
        }
        None => inner,
    };
    xetal_store::install(std::sync::Arc::new(xetal_ext_bridge::ExtStore::new(
        inner, registry,
    )));
}
