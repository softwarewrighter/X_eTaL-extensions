//! A memory store as the store in use: files read and written, lines
//! taken in the order queued, pictures kept until taken.

use crate::{Memory, Store};

impl Store for Memory {
    fn get(&self, path: &str) -> Result<String, String> {
        let files = self
            .files
            .lock()
            .map_err(|_| "the store is unusable".to_string())?;
        files
            .get(path)
            .cloned()
            .ok_or_else(|| format!("{path}: no such file"))
    }

    fn put(&self, path: &str, text: &str) -> Result<(), String> {
        let mut files = self
            .files
            .lock()
            .map_err(|_| "the store is unusable".to_string())?;
        files.insert(path.into(), text.into());
        Ok(())
    }

    fn line(&self) -> Result<String, String> {
        let mut typed = self
            .typed
            .lock()
            .map_err(|_| "the store is unusable".to_string())?;
        typed.pop_front().ok_or_else(|| "no more input".to_string())
    }

    fn show(&self, svg: &str) -> Result<(), String> {
        let mut pictures = self
            .pictures
            .lock()
            .map_err(|_| "the store is unusable".to_string())?;
        pictures.push(svg.into());
        Ok(())
    }

    fn take_shown(&self) -> Vec<String> {
        self.pictures
            .lock()
            .map(|mut p| std::mem::take(&mut *p))
            .unwrap_or_default()
    }
}
