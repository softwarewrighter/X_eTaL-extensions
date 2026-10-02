//! Output shown as it is written. A session replays its accepted
//! source for each new line, so the first `skip` bytes of a run's
//! output were shown before; everything after them goes straight to
//! the sink, while the whole output is kept.

use std::io::{self, Write};

pub(crate) struct Live<'s> {
    pub(crate) skip: usize,
    pub(crate) all: Vec<u8>,
    pub(crate) sink: &'s mut (dyn Write + Send),
}

impl Write for Live<'_> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let start = self.all.len();
        self.all.extend_from_slice(buf);
        let from = self.skip.max(start);
        if from < self.all.len() {
            self.sink.write_all(&self.all[from..])?;
            self.sink.flush()?;
        }
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        self.sink.flush()
    }
}
