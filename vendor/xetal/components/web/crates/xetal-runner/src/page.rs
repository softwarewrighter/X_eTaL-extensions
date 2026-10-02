//! A run on the page itself, for a program that reads the keyboard
//! ([]R_EAD): a worker has no prompt to ask with. The page cannot
//! repaint while it runs, so the prompt shows the last lines printed.

use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

use xetal_play::Run;

use crate::Request;

/// The last lines a run on the page printed, for the keyboard prompt.
static RECENT: Mutex<VecDeque<String>> = Mutex::new(VecDeque::new());

/// The last lines printed by a run on the page, shown when it asks for
/// a line ([]R_EAD), since the page cannot repaint while it runs.
pub fn recent() -> String {
    RECENT
        .lock()
        .map(|r| r.iter().cloned().collect::<Vec<_>>().join("\n"))
        .unwrap_or_default()
}

/// Run on the page (a program reading the keyboard), keeping the last
/// lines printed for the prompt.
pub(crate) fn on_page(req: &Request) -> Run {
    xetal_play::set_boxed(req.boxed);
    let printed = Arc::new(Mutex::new(String::new()));
    let p = printed.clone();
    if let Ok(mut recent) = RECENT.lock() {
        recent.clear();
    }
    let mut out = xetal_play::Lines::new(move |line: &str| {
        if let Ok(mut s) = p.lock() {
            s.push_str(&format!("{line}\n"));
        }
        if let Ok(mut recent) = RECENT.lock() {
            recent.push_back(line.into());
            if recent.len() > 12 {
                recent.pop_front();
            }
        }
    });
    let run = xetal_play::run_to(&req.src, req.seed, &mut out);
    drop(out);
    let out = printed.lock().map(|s| s.clone()).unwrap_or_default() + &run.out;
    Run { out, ..run }
}
