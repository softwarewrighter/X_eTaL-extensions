//! A game kept running: an X_eTaL program started once, run to where
//! it waits (`[]E_VENT`, `[]K_EY`, `[]R_EAD`), given a line an event
//! (`tick 0.05`, `key space`, `click 364 221`) and run to its next
//! wait, what it printed and the picture it showed (`[]S_HOW`) taken
//! after each. The host decides nothing about a game: lines go in,
//! text and pictures come out.
//!
//! - [`Files`]: what the program reads, by the path X_eTaL asks for
//! - [`Game`]: the running program
//! - [`scene`]: the board's calls a picture carries
//!
//! `page` keeps one game for a page and exports it to `game.js`.

use std::io::Write;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use xetal_play::{Interactive, Step};
use xetal_store::Store;

use crate::run::{Linked, store_over};

/// The most transitions a program may run between two waits unless a
/// host says otherwise ([`Game::start_within`]): a program that never
/// waits stops with an error instead of holding the page for good.
pub const LIMIT: usize = 2_000_000_000;

/// A game's files: its program, the libraries it imports and the data
/// it reads, each under the path X_eTaL asks the store for (`play.xtl`,
/// `Walking.xtl`, `../../world/zones.toml`). Cheap to clone.
#[derive(Clone, Debug, Default)]
pub struct Files(Arc<Vec<(String, String)>>);

impl Files {
    /// The files, from (name, text) pairs; of two with one name the
    /// first is read.
    pub fn new<N: Into<String>, T: Into<String>>(pairs: impl IntoIterator<Item = (N, T)>) -> Files {
        let pairs = pairs.into_iter().map(|(n, t)| (n.into(), t.into()));
        Files(Arc::new(pairs.collect()))
    }

    /// The text of a file, by its name.
    #[must_use]
    pub fn text(&self, name: &str) -> Option<&str> {
        let found = self.0.iter().find(|(n, _)| n == name);
        found.map(|(_, t)| t.as_str())
    }

    /// The names, in the order given.
    pub fn names(&self) -> impl Iterator<Item = &str> {
        self.0.iter().map(|(n, _)| n.as_str())
    }
}

/// Where a game is: what it waits for, or over.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum State {
    /// Waiting for an event's line (`[]E_VENT`).
    Event,
    /// Waiting for a key's name (`[]K_EY`).
    Key,
    /// Waiting for a typed line (`[]R_EAD`).
    Line,
    /// Ended, stopped or failed ([`Game::error`] says which).
    Over,
}

/// What a program has printed and shown and not yet been asked for,
/// and what it wrote.
#[derive(Default)]
struct Shown {
    printed: Vec<u8>,
    picture: Option<String>,
    pictures: usize,
    written: Vec<(String, String)>,
}

type Shared = Arc<Mutex<Shown>>;

fn locked(shared: &Shared) -> MutexGuard<'_, Shown> {
    shared.lock().unwrap_or_else(PoisonError::into_inner)
}

/// The store a game runs on: its files, what it wrote (kept for the
/// run, read back before the files), and the pictures it shows.
struct Session {
    files: Files,
    shared: Shared,
}

impl Store for Session {
    fn get(&self, path: &str) -> Result<String, String> {
        let shown = locked(&self.shared);
        let written = shown.written.iter().find(|(n, _)| n == path);
        let text = written.map(|(_, t)| t.as_str());
        let text = text.or_else(|| self.files.text(path));
        text.map(str::to_string)
            .ok_or_else(|| format!("{path}: no such file"))
    }

    fn put(&self, path: &str, text: &str) -> Result<(), String> {
        let mut shown = locked(&self.shared);
        match shown.written.iter_mut().find(|(n, _)| n == path) {
            Some(file) => file.1 = text.to_string(),
            None => shown.written.push((path.to_string(), text.to_string())),
        }
        Ok(())
    }

    fn line(&self) -> Result<String, String> {
        Err("no more input".into())
    }

    fn show(&self, svg: &str) -> Result<(), String> {
        let mut shown = locked(&self.shared);
        shown.picture = Some(svg.to_string());
        shown.pictures += 1;
        Ok(())
    }
}

/// The program's standard output, kept until it is taken.
struct Out(Shared);

impl Write for Out {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        locked(&self.0).printed.extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

/// The store of a game's files with extensions linked in (as
/// `run::install_with` links them), installed; what it shows goes to
/// `shared`.
fn installed(files: &Files, linked: &[Linked], shared: &Shared) -> Arc<dyn Store> {
    let session = Session {
        files: files.clone(),
        shared: shared.clone(),
    };
    let (store, _) = store_over(session, linked, &[]);
    let store: Arc<dyn Store> = Arc::new(store);
    xetal_store::install(store.clone());
    store
}

/// Installs a store of `files` and `linked` with no game on it: for
/// reading a game's sources (`source::markup`) before or beside a
/// game. A running game installs its own again at its next event.
pub fn install(files: &Files, linked: &[Linked]) {
    installed(files, linked, &Shared::default());
}

/// A running program and what it has printed and shown.
///
/// X_eTaL's `Interactive` keeps a loaded program for the life of the
/// process, so every start costs the page that program's memory: a
/// page that starts again and again grows, slowly.
pub struct Game {
    run: Option<Interactive>,
    state: State,
    store: Arc<dyn Store>,
    shared: Shared,
    error: String,
    limit: usize,
}

impl Game {
    /// Starts `program` (a name among `files`) rolling from `seed`,
    /// with the extensions `linked`, and runs it to its first wait.
    /// A program that is missing, does not load or does not check is
    /// a game that is over, with its error.
    ///
    /// # Panics
    ///
    /// When a linked descriptor is invalid: a build error, not a user's.
    #[must_use]
    pub fn start(files: &Files, linked: &[Linked], program: &str, seed: u64) -> Game {
        Game::start_within(files, linked, program, seed, LIMIT)
    }

    /// As [`Game::start`], the program stopped with an error when it
    /// runs more than `limit` transitions between two waits.
    ///
    /// # Panics
    ///
    /// When a linked descriptor is invalid: a build error, not a user's.
    #[must_use]
    pub fn start_within(
        files: &Files,
        linked: &[Linked],
        program: &str,
        seed: u64,
        limit: usize,
    ) -> Game {
        let shared = Shared::default();
        let store = installed(files, linked, &shared);
        let mut game = Game {
            run: None,
            state: State::Over,
            store,
            shared,
            error: String::new(),
            limit,
        };
        let Some(src) = files.text(program) else {
            game.error = format!("{program}: no such file");
            return game;
        };
        match Interactive::start(src, seed, Box::new(Out(game.shared.clone()))) {
            Ok(run) => game.run = Some(run),
            Err(r) => {
                // a library has nothing to run: its types are its output
                locked(&game.shared)
                    .printed
                    .extend_from_slice(r.out.as_bytes());
                game.error = r.err.trim().to_string();
            }
        }
        game.go();
        game
    }

    /// Gives the waiting program a line -- an event's (`tick 0.05`), a
    /// key's name, a typed line: whatever it waits for -- and runs it
    /// to its next wait. Nothing once the game is over.
    pub fn event(&mut self, line: &str) -> State {
        if let Some(run) = self.run.as_mut() {
            run.feed(line.to_string());
            self.go();
        }
        self.state
    }

    /// Runs until the program waits, ends, fails or passes the limit.
    /// The store is one for the process, so the game's is installed
    /// again first: a page may have run something else since.
    fn go(&mut self) {
        let Some(run) = self.run.as_mut() else {
            self.state = State::Over;
            return;
        };
        xetal_store::install(self.store.clone());
        self.state = match run.step(self.limit) {
            Step::Waiting => State::Line,
            Step::WaitingKey => State::Key,
            Step::WaitingEvent => State::Event,
            Step::Done(r) => {
                self.error = r.err.trim().to_string();
                State::Over
            }
            Step::Running => {
                self.error = format!("stopped: {} steps without waiting", self.limit);
                State::Over
            }
        };
        if self.state == State::Over {
            self.run = None;
        }
    }

    /// Stops the program where it waits: the game is over, with no
    /// error. (A program that should end itself is given `end`.)
    pub fn stop(&mut self) {
        self.run = None;
        self.state = State::Over;
    }

    /// What the game waits for, or over.
    #[must_use]
    pub fn state(&self) -> State {
        self.state
    }

    /// Has the program ended, been stopped or failed?
    #[must_use]
    pub fn over(&self) -> bool {
        self.state == State::Over
    }

    /// What the program stopped on, a line each: its error, and before
    /// it the warnings of a program that ended. Empty while it runs and
    /// after a clean end.
    #[must_use]
    pub fn error(&self) -> &str {
        &self.error
    }

    /// What the program printed since this was last asked.
    pub fn take_printed(&mut self) -> String {
        let printed = std::mem::take(&mut locked(&self.shared).printed);
        String::from_utf8_lossy(&printed).into_owned()
    }

    /// The latest picture the program showed since this was last asked
    /// (an SVG document; [`scene`] reads the board's calls in it), or
    /// none when it showed none.
    pub fn take_picture(&mut self) -> Option<String> {
        locked(&self.shared).picture.take()
    }

    /// How many pictures the program has shown since it started.
    #[must_use]
    pub fn pictures(&self) -> usize {
        locked(&self.shared).pictures
    }
}

/// The scene a picture carries: the text of its
/// `<metadata id="scene">`, the board player's calls, a line each (as
/// written: the board player reads it as text). None when the picture
/// has none.
#[must_use]
pub fn scene(svg: &str) -> Option<&str> {
    const OPEN: &str = "<metadata id=\"scene\">";
    let start = svg.find(OPEN)? + OPEN.len();
    let end = svg[start..].find("</metadata>")?;
    Some(&svg[start..start + end])
}
