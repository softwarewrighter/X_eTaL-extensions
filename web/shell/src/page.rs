//! One game for a page: the running [`Game`] kept for the page's one
//! thread, and the functions a page exports to `static/game.js`
//! (`game_page!` writes the exports). A page's whole host code is a
//! [`Page`] and that macro; nothing here knows a game's rules.

use std::cell::RefCell;

use crate::play::{Files, Game};
use crate::run::Linked;
use crate::source;

/// What a page plays: the program's name among its files, the files
/// ((name, text) pairs: the program, the libraries it imports, the
/// data it reads, each under the path X_eTaL asks for), and the
/// extensions linked in.
pub struct Page {
    pub program: &'static str,
    pub files: &'static [(&'static str, &'static str)],
    pub linked: &'static [Linked],
}

impl Page {
    fn files(&self) -> Files {
        Files::new(self.files.iter().copied())
    }
}

thread_local! {
    static GAME: RefCell<Option<Game>> = const { RefCell::new(None) };
}

fn with<T: Default>(f: impl FnOnce(&mut Game) -> T) -> T {
    GAME.with(|g| g.borrow_mut().as_mut().map(f).unwrap_or_default())
}

/// Starts the page's game from its beginning, rolling from `seed`, and
/// runs it to its first wait (the game before it, if any, is dropped).
pub fn start(page: &Page, seed: u32) {
    let game = Game::start(&page.files(), page.linked, page.program, u64::from(seed));
    GAME.with(|g| *g.borrow_mut() = Some(game));
}

/// Gives the game a line (an event's, a key's name, a typed line) and
/// runs it to its next wait; true once the game is over.
pub fn event(line: &str) -> bool {
    GAME.with(|g| {
        g.borrow_mut().as_mut().is_none_or(|g| {
            g.event(line);
            g.over()
        })
    })
}

/// The latest picture shown since this was last asked (SVG), or empty.
#[must_use]
pub fn picture() -> String {
    with(|g| g.take_picture().unwrap_or_default())
}

/// What the game printed since this was last asked.
#[must_use]
pub fn printed() -> String {
    with(Game::take_printed)
}

/// What the game stopped on, or empty.
#[must_use]
pub fn error() -> String {
    with(|g| g.error().to_string())
}

/// Is the game over (or not started)?
#[must_use]
pub fn over() -> bool {
    GAME.with(|g| g.borrow().as_ref().is_none_or(Game::over))
}

/// Stops the game where it waits.
pub fn stop() {
    with(Game::stop);
}

/// The names of the page's files, a line each: the sources a page may
/// show.
#[must_use]
pub fn names(page: &Page) -> String {
    let names: Vec<&str> = page.files.iter().map(|(n, _)| *n).collect();
    names.join("\n")
}

/// One of the page's files as HTML text: X_eTaL (`.xtl`, `.xtlm`)
/// drawn decorated, a span a token (`source::markup`: the classes
/// `shell.css` colors), anything else as it is, escaped. Empty for a
/// name that is not a file.
#[must_use]
pub fn rendered(page: &Page, name: &str) -> String {
    let files = page.files();
    let text = files.text(name).unwrap_or_default();
    if name.ends_with(".xtl") || name.ends_with(".xtlm") {
        // the names a file imports are classed by reading its libraries
        crate::play::install(&files, page.linked);
        return source::markup(text);
    }
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

/// The functions `static/game.js` is given, exported from a page's
/// binary for the [`Page`] named: `start(seed)`, `event(line)`,
/// `picture()`, `printed()`, `error()`, `over()`, `stop()`, `files()`
/// and `rendered(name)`. The page's crate depends on `wasm-bindgen`.
///
/// ```ignore
/// const PAGE: Page = Page { program: "play.xtl", files: FILES, linked: &[] };
/// xetal_ext_shell::game_page!(PAGE);
/// fn main() {}
/// ```
#[macro_export]
macro_rules! game_page {
    ($page:expr) => {
        /// Start the game from its beginning, to its first wait.
        #[wasm_bindgen::prelude::wasm_bindgen]
        pub fn start(seed: u32) {
            $crate::page::start(&$page, seed);
        }

        /// Give the game a line; true once the game is over.
        #[wasm_bindgen::prelude::wasm_bindgen]
        pub fn event(line: &str) -> bool {
            $crate::page::event(line)
        }

        /// The latest picture shown since last asked (SVG), or empty.
        #[wasm_bindgen::prelude::wasm_bindgen]
        #[must_use]
        pub fn picture() -> String {
            $crate::page::picture()
        }

        /// What the game printed since last asked.
        #[wasm_bindgen::prelude::wasm_bindgen]
        #[must_use]
        pub fn printed() -> String {
            $crate::page::printed()
        }

        /// What the game stopped on, or empty.
        #[wasm_bindgen::prelude::wasm_bindgen]
        #[must_use]
        pub fn error() -> String {
            $crate::page::error()
        }

        /// Is the game over?
        #[wasm_bindgen::prelude::wasm_bindgen]
        #[must_use]
        pub fn over() -> bool {
            $crate::page::over()
        }

        /// Stop the game where it waits.
        #[wasm_bindgen::prelude::wasm_bindgen]
        pub fn stop() {
            $crate::page::stop();
        }

        /// The names of the game's files, a line each.
        #[wasm_bindgen::prelude::wasm_bindgen]
        #[must_use]
        pub fn files() -> String {
            $crate::page::names(&$page)
        }

        /// One of the game's files as HTML text.
        #[wasm_bindgen::prelude::wasm_bindgen]
        #[must_use]
        pub fn rendered(name: &str) -> String {
            $crate::page::rendered(&$page, name)
        }
    };
}
