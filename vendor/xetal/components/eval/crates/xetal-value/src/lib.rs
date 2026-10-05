//! Runtime values: scalars, arrays, closures and built-ins, the
//! persistent environment, printed results, conversions to generic
//! arrays.

mod convert;
mod display;
mod grid;
mod shown;
mod tags;
mod value;

pub use convert::{as_array, as_vector, major_cells, to_value};
pub use display::{picture, printed};
pub use grid::grid;
pub use shown::{nested, shown};
pub use tags::{COLORS, KEYS, key_named, name as tag_name};
pub use value::{Closure, Env, Frame, Prim, Slot, Value, extend, lookup};
