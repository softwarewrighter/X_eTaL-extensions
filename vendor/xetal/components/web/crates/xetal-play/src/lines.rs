//! Output handed over a line at a time, as it is printed: the live
//! demo's worker posts each line to the page, so progress shows while a
//! program runs.

use std::io::{Result, Write};

/// A writer that calls `each` with every complete line written (without
/// its newline), and with what is left when it is dropped.
pub struct Lines<F: FnMut(&str) + Send> {
    pending: Vec<u8>,
    each: F,
}

impl<F: FnMut(&str) + Send> Lines<F> {
    pub fn new(each: F) -> Self {
        Lines {
            pending: Vec::new(),
            each,
        }
    }
}

impl<F: FnMut(&str) + Send> Write for Lines<F> {
    fn write(&mut self, bytes: &[u8]) -> Result<usize> {
        for &b in bytes {
            match b {
                b'\n' => {
                    let line = std::mem::take(&mut self.pending);
                    (self.each)(&String::from_utf8_lossy(&line));
                }
                b => self.pending.push(b),
            }
        }
        Ok(bytes.len())
    }

    fn flush(&mut self) -> Result<()> {
        Ok(())
    }
}

impl<F: FnMut(&str) + Send> Drop for Lines<F> {
    fn drop(&mut self) {
        if !self.pending.is_empty() {
            let rest = std::mem::take(&mut self.pending);
            (self.each)(&String::from_utf8_lossy(&rest));
        }
    }
}
