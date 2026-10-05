//! A run the page steps (D50, Saga 25): started, taken a slice at a
//! time, waiting at `[]R_EAD` until a line is fed, its output written
//! as it is printed - what the live demo's terminal drives.

use std::io::Write;
use std::sync::{Arc, Mutex, OnceLock};

use xetal_play::{Interactive, Step};
use xetal_store::{Memory, install};

fn store() {
    static STORE: OnceLock<Arc<Memory>> = OnceLock::new();
    STORE.get_or_init(|| {
        let store = Arc::new(Memory::default());
        install(store.clone());
        store
    });
}

/// Output shared with the test.
#[derive(Clone, Default)]
struct Shared(Arc<Mutex<Vec<u8>>>);

impl Write for Shared {
    fn write(&mut self, b: &[u8]) -> std::io::Result<usize> {
        self.0
            .lock()
            .map_err(|_| std::io::Error::other("poisoned"))?
            .extend_from_slice(b);
        Ok(b.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl Shared {
    fn text(&self) -> String {
        String::from_utf8_lossy(&self.0.lock().expect("lock")).into_owned()
    }
}

const ADVENTURE: &str = "u:t_urn := { n ->
  cmd := []R_EAD @
  cmd m_atch \"q\" ? n
  shown := p_rint! \"you \" c_at cmd
  u:t_urn n + 1
}
shown := p_rint! \"ready\"
u:t_urn 0
";

/// Step until the run waits or ends.
fn settle(run: &mut Interactive) -> Step {
    loop {
        match run.step(1000) {
            Step::Running => continue,
            other => return other,
        }
    }
}

#[test]
fn an_interactive_run_waits_for_each_line() {
    store();
    let out = Shared::default();
    let mut run = Interactive::start(ADVENTURE, 1, Box::new(out.clone())).expect("starts");
    assert_eq!(settle(&mut run), Step::Waiting);
    assert_eq!(out.text(), "ready\n");
    run.feed("look".into());
    assert_eq!(settle(&mut run), Step::Waiting);
    assert_eq!(out.text(), "ready\nyou look\n");
    run.feed("q".into());
    let Step::Done(end) = settle(&mut run) else {
        panic!("should finish")
    };
    assert_eq!(out.text(), "ready\nyou look\n1\n");
    assert_eq!(end.err, "");
}

#[test]
fn an_ill_typed_program_does_not_start() {
    store();
    let Err(end) = Interactive::start("1 + \"a\"", 1, Box::new(Shared::default())) else {
        panic!("should not start");
    };
    assert!(end.err.contains("type-mismatch"), "{}", end.err);
}

#[test]
fn a_runtime_error_ends_the_run() {
    store();
    let mut run = Interactive::start("p_rint! 1; 9 s_elect 1 2 3", 1, Box::new(Shared::default()))
        .expect("starts");
    let Step::Done(end) = settle(&mut run) else {
        panic!("should finish")
    };
    assert!(end.err.contains("error["), "{}", end.err);
}
