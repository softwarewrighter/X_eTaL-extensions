//! A program with its libraries, ready to check and run: the macro
//! phase (`xetal-macro`, libraries on disk and built in), then Core.
//! Every error comes back located where it was written, as a plain
//! diagnostic for one file or `at FILE:LINE:COLUMN` for several. A
//! library file can be loaded and checked on its own.

mod library;
mod load;
mod types;

pub use library::{is_library, load_library, load_library_with};
pub use load::{Loaded, in_program, load, load_with, located};
pub use types::{library_types, program_types};
