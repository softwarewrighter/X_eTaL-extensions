//! The worker: runs one program per message, posting each line of
//! output, each picture and each file written as it happens. Workers
//! have no local storage, so the page sends the saved files with the
//! program, and the worker posts back what it writes.

use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

use wasm_bindgen::{JsCast, JsValue, closure::Closure};
use web_sys::{DedicatedWorkerGlobalScope, MessageEvent};
use xetal_store::Store;

use crate::{Event, Mode, Request};

pub(crate) fn post(event: Event) {
    let scope: DedicatedWorkerGlobalScope = js_sys::global().unchecked_into();
    let _ = scope.post_message(&JsValue::from_str(&event.encode()));
}

/// The page's files as they were when the run began, and what the
/// program writes, posted back as it is written.
struct Snapshot(Mutex<BTreeMap<String, String>>);

impl Store for Snapshot {
    fn get(&self, path: &str) -> Result<String, String> {
        let files = self.0.lock().map_err(|_| "the files are unusable")?;
        files
            .get(path)
            .cloned()
            .ok_or_else(|| format!("{path}: no such file"))
    }

    fn put(&self, path: &str, text: &str) -> Result<(), String> {
        let mut files = self.0.lock().map_err(|_| "the files are unusable")?;
        files.insert(path.into(), text.into());
        post(Event::Wrote(path.into(), text.into()));
        Ok(())
    }

    fn line(&self) -> Result<String, String> {
        Err("no keyboard in a notebook (run the program to type its lines)".into())
    }

    fn show(&self, svg: &str) -> Result<(), String> {
        post(Event::Picture(svg.into()));
        Ok(())
    }
}

/// Run one request: plainly as a program stepped in slices (waiting at
/// `[]R_EAD` for the page to send a typed line), or as a notebook
/// posting each statement before it runs.
fn run(req: Request) {
    let files = req.files.into_iter().collect();
    xetal_store::install(Arc::new(Snapshot(Mutex::new(files))));
    // The program's terminal: standard error a red line, a 24 by 80 grid.
    let error = |line: &str| post(Event::Out(format!("\x1b[31m{line}\x1b[39m")));
    let tty = xetal_tty::Forward {
        error,
        facts: [24, 80, 1, 1],
    };
    xetal_tty::install(Arc::new(tty));
    xetal_play::set_boxed(req.boxed);
    if let Mode::Notebook(upto) = req.mode {
        let mut out = xetal_play::Lines::new(|line: &str| post(Event::Out(line.into())));
        let mut cell = |source: &str| post(Event::Source(source.into()));
        let run = xetal_play::notebook_to(&req.src, req.seed, upto, &mut cell, &mut out);
        drop(out);
        return crate::stepping::finish(run);
    }
    crate::stepping::start(&req.src, req.seed);
}

/// The worker's entry: run each request the page sends.
pub fn start() {
    let on_message = Closure::<dyn FnMut(MessageEvent)>::new(|m: MessageEvent| {
        let Some(text) = m.data().as_string() else {
            return;
        };
        match crate::stepping::typed_line(&text) {
            Some(line) => crate::stepping::typed(line),
            None => {
                if let Some(req) = Request::decode(&text) {
                    run(req);
                }
            }
        }
    });
    let scope: DedicatedWorkerGlobalScope = js_sys::global().unchecked_into();
    scope.set_onmessage(Some(on_message.as_ref().unchecked_ref()));
    on_message.forget();
    post(Event::Ready);
}
