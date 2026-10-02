//! Arrays drawn as self-contained SVG (lang-choices QD5): a matrix as a
//! grid of cells (a large one as an image), a 2-row matrix of points as
//! a path, a rank-3 array as frames shown in turn. Pure text in, text
//! out: it knows nothing of the language, only shapes and cells, so the
//! command line, the browser and any other host show the same picture.
//! The pieces (colors, SVG elements, frames) are in `xetal-svg`.

mod grid;
mod path;
mod raster;

pub use grid::grid;
pub use path::path;
pub use raster::RASTER_CELLS;
pub use xetal_svg::{Cells, DrawError};
