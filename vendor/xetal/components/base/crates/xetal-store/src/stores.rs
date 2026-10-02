//! Stores: what a store is, and the disk.

use std::io::BufRead;

/// A place files are read from and written to, by path, and where a
/// line typed at the keyboard comes from (`[]R_EAD`).
pub trait Store: Send + Sync {
    fn get(&self, path: &str) -> Result<String, String>;
    fn put(&self, path: &str, text: &str) -> Result<(), String>;

    /// A line typed at the keyboard, without its newline; standard
    /// input unless the store knows better (the browser asks).
    fn line(&self) -> Result<String, String> {
        let mut line = String::new();
        match std::io::stdin().lock().read_line(&mut line) {
            Ok(0) => Err("no more input".into()),
            Ok(_) => Ok(line.trim_end_matches(['\n', '\r']).to_string()),
            Err(e) => Err(e.to_string()),
        }
    }

    /// Show a picture (an SVG document, `[]S_HOW`): the host decides
    /// how (numbered files on the command line, a pane in the browser).
    fn show(&self, _svg: &str) -> Result<(), String> {
        Err("this host has no place to show pictures".into())
    }

    /// The pictures shown since they were last taken, in order; a store
    /// that keeps none (files on disk) gives none.
    fn take_shown(&self) -> Vec<String> {
        Vec::new()
    }
}

/// The file system; writing makes missing directories.
pub struct Disk;

impl Store for Disk {
    fn get(&self, path: &str) -> Result<String, String> {
        std::fs::read_to_string(path).map_err(|e| format!("{path}: {e}"))
    }

    fn put(&self, path: &str, text: &str) -> Result<(), String> {
        let dir = std::path::Path::new(path).parent();
        if let Some(dir) = dir.filter(|d| !d.as_os_str().is_empty()) {
            std::fs::create_dir_all(dir).map_err(|e| format!("{path}: {e}"))?;
        }
        std::fs::write(path, text).map_err(|e| format!("{path}: {e}"))
    }
}
