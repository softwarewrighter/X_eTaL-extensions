//! Full-screen terminal sessions. `with_terminal` enters raw mode and
//! the alternate screen, runs the app, and restores the terminal on
//! return, on error and on panic (ratatui installs the panic hook).

use std::io;

use ratatui::DefaultTerminal;

/// Run `app` on the terminal; the terminal is restored however it ends.
pub fn with_terminal<T>(app: impl FnOnce(&mut DefaultTerminal) -> io::Result<T>) -> io::Result<T> {
    let mut terminal = ratatui::try_init()?;
    let result = app(&mut terminal);
    ratatui::restore();
    result
}
