//! The session on standard input and output. On a terminal the line is
//! edited live, drawn decorated as it is typed, with history
//! (`xetal-line`); otherwise lines are read plainly (a prompt only when
//! input is a terminal), so piped sessions print just results.

use std::io::{BufRead, IsTerminal, Write};

use crate::{Reply, Session};

/// The session with the live line editor.
fn live() -> std::io::Result<()> {
    let (mut session, mut line, mut more) =
        (Session::default(), xetal_line::Line::default(), false);
    loop {
        let prompt = if more { "    ...  " } else { "xetal> " };
        let Some(text) = xetal_line::read_line(&mut line, prompt)? else {
            return Ok(());
        };
        more = match session.feed(&text) {
            Reply::More => true,
            Reply::Done { out, err } => {
                print!("{out}");
                eprint!("{err}");
                false
            }
        };
    }
}

fn prompt(more: bool) {
    print!("{}", if more { "    ...  " } else { "xetal> " });
    let _ = std::io::stdout().flush();
}

/// Read lines until end of input; results go to stdout, warnings and
/// errors to stderr.
pub fn stdio() -> std::io::Result<()> {
    if std::io::stdin().is_terminal() && std::io::stdout().is_terminal() {
        return live();
    }
    let tty = std::io::stdin().is_terminal();
    let mut session = Session::default();
    if tty {
        prompt(false);
    }
    for line in std::io::stdin().lock().lines() {
        let more = match session.feed(&line?) {
            Reply::More => true,
            Reply::Done { out, err } => {
                print!("{out}");
                eprint!("{err}");
                false
            }
        };
        if tty {
            prompt(more);
        }
    }
    Ok(())
}
