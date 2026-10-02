//! The browser's local storage as the program's files, a prompt as its
//! keyboard, and the output pane as where its pictures are shown: installed as the store (`xetal-store`) the language
//! reads and writes through, so saved files, your libraries and
//! `[]N_PUT` / `[]N_GET` / `[]R_EAD` all work in the page.

use std::sync::Mutex;

use xetal_store::Store;

/// The pictures shown (`[]S_HOW`) and not yet taken by the output pane.
static SHOWN: Mutex<Vec<String>> = Mutex::new(Vec::new());

const PREFIX: &str = "xetal:";

fn storage() -> Result<web_sys::Storage, String> {
    web_sys::window()
        .and_then(|w| w.local_storage().ok().flatten())
        .ok_or_else(|| "this browser keeps no local storage".to_string())
}

/// Files in local storage, under `xetal:` and their path.
pub struct Local;

impl Store for Local {
    fn get(&self, path: &str) -> Result<String, String> {
        let found = storage()?
            .get_item(&format!("{PREFIX}{path}"))
            .ok()
            .flatten();
        found.ok_or_else(|| format!("{path}: no such file"))
    }

    fn put(&self, path: &str, text: &str) -> Result<(), String> {
        let key = format!("{PREFIX}{path}");
        storage()?
            .set_item(&key, text)
            .map_err(|_| format!("{path}: the browser refused to store it"))
    }

    fn line(&self) -> Result<String, String> {
        let window = web_sys::window().ok_or("no window")?;
        let typed = window
            .prompt_with_message(&format!(
                "{}\n\n[]R_EAD: type a line",
                xetal_runner::recent()
            ))
            .ok()
            .flatten();
        typed.ok_or_else(|| "no more input (the prompt was cancelled)".to_string())
    }

    fn show(&self, svg: &str) -> Result<(), String> {
        let mut shown = SHOWN.lock().map_err(|_| "the pictures are unusable")?;
        shown.push(svg.into());
        Ok(())
    }

    fn take_shown(&self) -> Vec<String> {
        SHOWN
            .lock()
            .map(|mut s| std::mem::take(&mut *s))
            .unwrap_or_default()
    }
}

/// The paths of the files saved in local storage, in order.
pub fn saved() -> Vec<String> {
    let Ok(store) = storage() else {
        return Vec::new();
    };
    let count = store.length().unwrap_or(0);
    let mut paths: Vec<String> = (0..count)
        .filter_map(|i| store.key(i).ok().flatten())
        .filter_map(|k| k.strip_prefix(PREFIX).map(String::from))
        .collect();
    paths.sort();
    paths
}
