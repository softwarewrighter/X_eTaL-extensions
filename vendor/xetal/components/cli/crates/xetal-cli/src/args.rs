//! The command line: subcommands, options and inputs.

use clap::{Args, Parser, Subcommand};
use xetal_base::{Diagnostic, LANG_NAME};

use crate::stages::read_input;

/// Full `-V` / `--version` block: version, copyright, license,
/// repository, then build information from `build.rs`.
const VERSION: &str = concat!(
    env!("CARGO_PKG_VERSION"),
    "\nCopyright (c) 2026 Michael A Wright\n",
    "License: ",
    env!("CARGO_PKG_LICENSE"),
    "\nRepository: ",
    env!("CARGO_PKG_REPOSITORY"),
    "\n\nBuild Information:\n  Host: ",
    env!("BUILD_HOST"),
    "\n  Commit: ",
    env!("GIT_HASH"),
    "\n  Timestamp: ",
    env!("BUILD_TIMESTAMP"),
);

/// A terse, statically typed, functional array language in ASCII.
#[derive(Parser)]
#[command(
    name = LANG_NAME,
    bin_name = "xetal",
    version = VERSION,
    about,
    long_about = "A terse, statically typed, functional array language whose \
                  source is plain ASCII. An underlined letter makes a name a \
                  function (`r_ev` is reverse), `_digits` give its axes \
                  (`o_-_2` rotates along axis 2), and a quoted function is \
                  an operand (`'+ r_/ v` reduces v by plus).",
    after_long_help = include_str!("cli_help.txt"),
    override_usage = "xetal [OPTIONS] [SCRIPT]\n       xetal [OPTIONS] <COMMAND>"
)]
pub(crate) struct Cli {
    #[command(subcommand)]
    pub(crate) command: Option<Command>,
    /// A script to run (`xetal FILE` is `xetal run FILE`; for shebangs).
    pub(crate) script: Option<String>,
    /// Where pictures shown with []S_HOW are written, as NAME-1.svg,
    /// NAME-2.svg, ... (default: XETAL_DRAW, else the current directory).
    #[arg(long, global = true, value_name = "DIR")]
    pub(crate) draw: Option<String>,
    /// Draw nested arrays in plain ASCII (. ' - | > v e) instead of box
    /// characters, as APL2's DISPLAY did on plain terminals.
    #[arg(long, global = true)]
    pub(crate) ascii: bool,
    /// Print every array result boxed, flat ones too, as APL2's DISPLAY
    /// draws it (what d_isplay gives).
    #[arg(long = "box", global = true)]
    pub(crate) boxed: bool,
    /// Set a configuration fact NAME, which a macro tests with
    /// `@ c_fg< "NAME"` (repeatable; `cli` holds at the command line).
    #[arg(long = "cfg", global = true, value_name = "NAME")]
    pub(crate) cfg: Vec<String>,
}

/// Source given inline with `-e` or as a file path.
#[derive(Args)]
pub(crate) struct Input {
    /// Source text to process.
    #[arg(
        short = 'e',
        long = "expr",
        conflicts_with = "file",
        allow_hyphen_values = true
    )]
    pub(crate) expr: Option<String>,
    /// Source file to process.
    pub(crate) file: Option<String>,
}

/// `eval` and `run` options: type-checked unless `--untyped`.
#[derive(Args)]
pub(crate) struct EvalArgs {
    #[command(flatten)]
    pub(crate) input: Input,
    /// Skip the type checker (for experiments such as the Y combinator).
    #[arg(long)]
    pub(crate) untyped: bool,
    /// Seed `r_oll!` so a run can be repeated (default: XETAL_SEED, else
    /// unpredictable).
    #[arg(long)]
    pub(crate) seed: Option<u64>,
    /// Show each statement pretty-printed, then its output (a notebook).
    #[arg(long)]
    pub(crate) echo: bool,
    /// With --echo, pause this many milliseconds after each statement.
    #[arg(long, requires = "echo", value_name = "MS")]
    pub(crate) delay: Option<u64>,
}

/// `doc` options: the cross-reference as JSON, as a site, or its doc
/// tests.
#[derive(Args)]
#[command(group(clap::ArgGroup::new("output").required(true).args(["json", "out", "test"])))]
pub(crate) struct DocArgs {
    #[command(flatten)]
    pub(crate) input: Input,
    /// Print the model as JSON: every item of the file, its imports and
    /// the system macros, with type, doc comment, source and uses.
    #[arg(long)]
    pub(crate) json: bool,
    /// Write the documentation as a static site into DIR: an index, a
    /// page per file with its items, its source drawn decorated, every
    /// name linked to its definition and its uses.
    #[arg(long, value_name = "DIR")]
    pub(crate) out: Option<String>,
    /// Run the file's `## >>` examples and compare what each prints
    /// with the output shown under it (each doc block one session).
    #[arg(long)]
    pub(crate) test: bool,
}

/// `render` options: decorated Unicode by default.
#[derive(Args)]
pub(crate) struct RenderArgs {
    #[command(flatten)]
    pub(crate) input: Input,
    /// Convert decorated Unicode back to raw ASCII (validated by lexing).
    #[arg(long)]
    pub(crate) raw: bool,
    /// Print LaTeX math for a post-processor (KaTeX, MathJax, pdflatex).
    #[arg(long, conflicts_with = "raw")]
    pub(crate) latex: bool,
    /// Color the decorated form for a terminal; text that does not lex
    /// is shown as typed, in red, and never stops the rendering.
    #[arg(long, conflicts_with_all = ["raw", "latex"])]
    pub(crate) color: bool,
    /// Print the decorated form as HTML spans for a web page (classes
    /// c-builtin, c-number, ...), escaped; like --color, text that does
    /// not lex is shown as typed.
    #[arg(long, conflicts_with_all = ["raw", "latex", "color"])]
    pub(crate) html: bool,
}

#[derive(Subcommand)]
pub(crate) enum Command {
    /// Print the token stream.
    Lex(Input),
    /// Print the decorated Unicode form (or --raw, --latex).
    Render(RenderArgs),
    /// Draw an annotated diagram (SVG) of a line of source from a notes
    /// file: the decorated line with callouts anchored to its tokens.
    Diagram(Input),
    /// Print the program after macro expansion (imports and names as
    /// written).
    Expand(Input),
    /// Document a program or library: every item of it, of what it
    /// imports and of the system macros (--json, or a site with --out),
    /// or run its examples (--test).
    Doc(DocArgs),
    /// Print the surface AST or an ambiguity report.
    Parse(Input),
    /// Print the canonical form.
    Fmt(Input),
    /// Print the Core IR.
    Core(Input),
    /// Print the inferred type of each top-level item.
    Type(Input),
    /// Type-check, evaluate and print each result.
    Eval(EvalArgs),
    /// Type-check and run a program file.
    Run {
        file: String,
        /// Skip the type checker.
        #[arg(long)]
        untyped: bool,
        /// Seed `r_oll!` (default: XETAL_SEED, else unpredictable).
        #[arg(long)]
        seed: Option<u64>,
        /// Show each statement pretty-printed, then its output (a notebook).
        #[arg(long)]
        echo: bool,
        /// With --echo, pause this many milliseconds after each statement.
        #[arg(long, requires = "echo", value_name = "MS")]
        delay: Option<u64>,
        /// Run the program in FILE first, silently, then this one as its
        /// continuation, showing only this one's output (org-babel sessions).
        #[arg(long, value_name = "FILE", conflicts_with_all = ["echo", "untyped"])]
        context: Option<String>,
    },
    /// Start an interactive session.
    Repl,
    /// Edit a file: ASCII left, live decorated view right, types below
    /// (Ctrl-S save, Ctrl-R run, Ctrl-Q quit).
    Edit {
        /// The file to edit (created on the first save if missing).
        file: String,
    },
}

impl Command {
    pub(crate) fn stage(&self) -> &'static str {
        match self {
            Command::Lex(_) => "lex",
            Command::Render(_) => "render",
            Command::Diagram(_) => "diagram",
            Command::Expand(_) => "expand",
            Command::Doc(_) => "doc",
            Command::Parse(_) => "parse",
            Command::Fmt(_) => "fmt",
            Command::Core(_) => "core",
            Command::Type(_) => "type",
            Command::Eval(_) => "eval",
            Command::Run { .. } => "run",
            Command::Repl => "repl",
            Command::Edit { .. } => "edit",
        }
    }

    /// The source this command operates on, if it takes one.
    pub(crate) fn source(&self) -> Option<Result<String, Diagnostic>> {
        match self {
            Command::Lex(i)
            | Command::Expand(i)
            | Command::Parse(i)
            | Command::Fmt(i)
            | Command::Core(i)
            | Command::Type(i)
            | Command::Diagram(i)
            | Command::Eval(EvalArgs { input: i, .. }) => {
                Some(read_input(i.expr.as_deref(), i.file.as_deref()))
            }
            Command::Render(RenderArgs { input: i, .. })
            | Command::Doc(DocArgs { input: i, .. }) => {
                Some(read_input(i.expr.as_deref(), i.file.as_deref()))
            }
            Command::Run { file, .. } => Some(read_input(None, Some(file))),
            Command::Repl | Command::Edit { .. } => None,
        }
    }
}
