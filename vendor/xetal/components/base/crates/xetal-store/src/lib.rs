//! Where a program's files live. The command line reads and writes the
//! disk; a host can install another store instead (the live demo
//! installs the browser's local storage), and both the system
//! built-ins (`[]N_GET`, `[]N_PUT`) and library lookup go through it,
//! as do pictures shown with `[]S_HOW`.

mod current;
mod drawing;
mod memory;
mod memory_store;
mod replay;
mod stores;

pub use current::{install, read, read_line, show, take_shown, write};
pub use drawing::Drawing;
pub use memory::Memory;
pub use replay::{muted, replay, shown};
pub use stores::{Disk, Store};
