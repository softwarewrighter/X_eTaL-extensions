//! The disk, with pictures: what the command line installs. Files and
//! the keyboard behave as on the disk; each picture shown (`[]S_HOW`) is
//! written to the next numbered file, `DIR/STEM-1.svg`, `DIR/STEM-2.svg`,
//! ..., and reported by the host's callback.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

use crate::{Disk, Store};

/// The disk, writing pictures as numbered SVG files in one directory.
pub struct Drawing {
    dir: PathBuf,
    stem: String,
    shown: AtomicUsize,
    notify: fn(&Path),
}

impl Drawing {
    /// Pictures go to `dir/stem-N.svg`; `notify` hears each path written.
    pub fn new(dir: impl Into<PathBuf>, stem: &str, notify: fn(&Path)) -> Drawing {
        Drawing {
            dir: dir.into(),
            stem: stem.into(),
            shown: AtomicUsize::new(0),
            notify,
        }
    }
}

impl Store for Drawing {
    fn get(&self, path: &str) -> Result<String, String> {
        Disk.get(path)
    }

    fn put(&self, path: &str, text: &str) -> Result<(), String> {
        Disk.put(path, text)
    }

    fn show(&self, svg: &str) -> Result<(), String> {
        let n = self.shown.fetch_add(1, Ordering::SeqCst) + 1;
        let path = self.dir.join(format!("{}-{n}.svg", self.stem));
        Disk.put(&path.display().to_string(), svg)?;
        (self.notify)(&path);
        Ok(())
    }
}
