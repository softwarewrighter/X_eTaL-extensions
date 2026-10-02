//! The live demo's engine, run natively against a store in memory.

use std::sync::{Arc, Mutex, OnceLock};

use xetal_play::{Run, check};
use xetal_store::{Memory, Store, install};

/// One store for every test here (the store in use is global).
fn store() -> &'static Arc<Memory> {
    static STORE: OnceLock<Arc<Memory>> = OnceLock::new();
    STORE.get_or_init(|| {
        let store = Arc::new(Memory::default());
        install(store.clone());
        store
    })
}

/// A run, one at a time: a run takes the store's pictures at its start
/// and end, so two at once (tests run in parallel) could take each
/// other's. The browser runs one at a time anyway.
fn run(src: &str, seed: u64) -> Run {
    let _turn = turn();
    xetal_play::run(src, seed)
}

fn turn() -> std::sync::MutexGuard<'static, ()> {
    static ONE_AT_A_TIME: Mutex<()> = Mutex::new(());
    ONE_AT_A_TIME.lock().unwrap_or_else(|e| e.into_inner())
}

#[test]
fn a_program_runs_and_its_output_is_kept() {
    store();
    let r = run("1 + 2\n'+ r_/ 1 2 3", 1);
    assert_eq!((r.out.as_str(), r.err.as_str()), ("3\n6\n", ""));
}

#[test]
fn a_library_comes_from_the_store_then_the_standard_ones() {
    store().put("Sq.xtl", "l:s_q := { _r * _r }\n").unwrap();
    let r = run("\"q:\" u_se< \"Sq\"\nq:s_q 4", 1);
    assert_eq!(r.out, "16\n", "{}", r.err);
    let r = run("\"s:\" u_se< \"Stats\"\ns:m_ean 1 2 3", 1);
    assert_eq!(r.out, "2.0\n", "{}", r.err);
}

#[test]
fn files_are_written_to_and_read_from_the_store() {
    let r = run(
        "\"1 2\" []N_PUT \"work/m.txt\"\nn_umbers []N_GET \"work/m.txt\"",
        1,
    );
    assert_eq!(r.out, "3\n1.0 2.0\n", "{}", r.err);
    assert_eq!(store().get("work/m.txt").unwrap(), "1 2");
}

#[test]
fn checking_gives_the_types_or_the_first_error() {
    store();
    assert_eq!(
        check("u:s_q := { _r * _r }\nu:s_q 3"),
        ["u:s_q : Num a => a -> a", "Int"]
    );
    let bad = check("1 + \"a\"");
    assert!(bad[0].starts_with("error[type-mismatch]"), "{bad:?}");
    let r = run("1 + \"a\"", 1);
    assert!(r.err.starts_with("error[type-mismatch]"), "{}", r.err);
}

#[test]
fn a_library_shows_its_exports_and_their_types() {
    store();
    let lib = "l:t_wice := { 2 * _r }\nh_alf := { _r / 2 }\n";
    assert_eq!(check(lib), ["l:t_wice : Num a => a -> a"]);
    assert_eq!(run(lib, 1).out, "l:t_wice : Num a => a -> a\n");
}

#[test]
fn a_line_is_read_from_the_store_in_use() {
    store().push_line("hello");
    assert_eq!(run("[]R_EAD @", 1).out, "hello\n");
}

#[test]
fn a_run_keeps_the_pictures_it_shows() {
    store();
    let r = run(
        "p := []S_HOW []G_RID 2 2 r_eshape 1 0 0 1\n[]S_HOW []G_RID 1 1 r_eshape 1",
        1,
    );
    assert_eq!(r.err, "");
    assert_eq!(r.pictures.len(), 2);
    assert!(
        r.pictures.iter().all(|p| p.starts_with("<svg")),
        "{:?}",
        r.pictures
    );
    assert!(run("1", 1).pictures.is_empty(), "each run starts with none");
}

/// Output streams: `run_to` hands each line to the writer as it is
/// printed, not all at the end (the live demo shows progress).
#[test]
fn output_is_handed_over_a_line_at_a_time() {
    use std::sync::{Arc, Mutex};
    let lines = Arc::new(Mutex::new(Vec::<String>::new()));
    let seen = lines.clone();
    let mut out = xetal_play::Lines::new(move |line: &str| seen.lock().unwrap().push(line.into()));
    let _turn = turn();
    let r = xetal_play::run_to("p := p_rint! 1\np := p_rint! \"two\"\n3", 1, &mut out);
    assert_eq!(r.err, "");
    assert_eq!(*lines.lock().unwrap(), ["1", "two", "3"]);
}

/// A writer that hands over whole lines, and what is left when dropped.
#[test]
fn lines_are_cut_at_newlines() {
    use std::io::Write;
    use std::sync::{Arc, Mutex};
    let lines = Arc::new(Mutex::new(Vec::<String>::new()));
    let seen = lines.clone();
    {
        let mut out =
            xetal_play::Lines::new(move |line: &str| seen.lock().unwrap().push(line.into()));
        out.write_all(b"a\nb").unwrap();
        out.write_all(b"c\n\nd").unwrap();
    }
    assert_eq!(*lines.lock().unwrap(), ["a", "bc", "", "d"]);
}

/// The notebook's events, in order: "S:" a statement's source (with
/// the comments above it), "O:" a line it printed.
fn notebook(src: &str, upto: Option<usize>) -> Vec<String> {
    use std::sync::{Arc, Mutex};
    let _turn = turn();
    let log = Arc::new(Mutex::new(Vec::<String>::new()));
    let (cells, lines) = (log.clone(), log.clone());
    let mut out =
        xetal_play::Lines::new(move |l: &str| lines.lock().unwrap().push(format!("O:{l}")));
    let mut cell = move |s: &str| cells.lock().unwrap().push(format!("S:{s}"));
    let r = xetal_play::notebook_to(src, 1, upto, &mut cell, &mut out);
    drop(out);
    assert_eq!(r.err, "", "{src}");
    log.lock().unwrap().clone()
}

const PROGRAM: &str = "# one\nx := 1\nx + 1\nu:f_ := {\n  _r * 10\n}\nu:f_ 4\n# the end\n";

#[test]
fn a_notebook_shows_each_statement_then_what_it_printed() {
    assert_eq!(
        notebook(PROGRAM, None),
        [
            "S:# one\nx := 1",
            "S:x + 1",
            "O:2",
            "S:u:f_ := {\n  _r * 10\n}",
            "S:u:f_ 4",
            "O:40",
            "S:# the end",
        ]
    );
}

#[test]
fn a_step_runs_up_to_a_statement_and_no_further() {
    assert_eq!(
        notebook(PROGRAM, Some(2)),
        ["S:# one\nx := 1", "S:x + 1", "O:2"]
    );
    assert_eq!(notebook(PROGRAM, Some(1)), ["S:# one\nx := 1"]);
    assert_eq!(xetal_play::statements(PROGRAM), 4);
    assert_eq!(
        xetal_play::statements("1 + "),
        0,
        "a program that does not load has none"
    );
}
