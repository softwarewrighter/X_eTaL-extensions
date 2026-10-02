//! What is drawn: the cells of an array, and how its shape is read.

/// The items of an array, in row-major order.
#[derive(Debug, Clone, PartialEq)]
pub enum Cells {
    /// Numbers (Bool and Int as well): all 0 or 1 draws dark "on" cells;
    /// anything else goes through the palette.
    Numbers(Vec<f64>),
    /// Characters, drawn in their cells; a space is left empty.
    Chars(Vec<char>),
}

impl Cells {
    /// How many cells there are.
    pub fn len(&self) -> usize {
        match self {
            Cells::Numbers(v) => v.len(),
            Cells::Chars(v) => v.len(),
        }
    }

    /// True when there are no cells.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// Why an array cannot be drawn.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DrawError {
    /// More than three axes (frames, rows, columns).
    Rank(usize),
    /// No cells to draw.
    Empty,
    /// The cells do not fill the shape.
    Length { expected: usize, found: usize },
    /// A path is 2 rows (x over y) of at least 2 points, or frames of them.
    Points(Vec<usize>),
}

impl DrawError {
    /// The diagnostic code, as `error[CODE]` shows it.
    pub fn code(&self) -> &'static str {
        match self {
            DrawError::Rank(_) => "rank",
            DrawError::Empty => "empty",
            DrawError::Length { .. } => "length-mismatch",
            DrawError::Points(_) => "shape-mismatch",
        }
    }

    /// A sentence saying what is wrong.
    pub fn message(&self) -> String {
        match self {
            DrawError::Rank(r) => {
                format!("a drawing has at most 3 axes (frames, rows, columns), not {r}")
            }
            DrawError::Empty => "an empty array has nothing to draw".into(),
            DrawError::Length { expected, found } => {
                format!("the shape holds {expected} cells, but {found} were given")
            }
            DrawError::Points(shape) => format!(
                "a path is 2 rows of points (x over y), at least 2 of them, or frames of such paths; not shape {shape:?}"
            ),
        }
    }
}

/// The shape read as frames of rows by columns.
#[derive(Debug, Clone, Copy)]
pub struct Layout {
    pub frames: usize,
    pub rows: usize,
    pub cols: usize,
}

impl Layout {
    /// A scalar is one cell, a vector one row, a matrix one frame.
    pub fn of(shape: &[usize], found: usize) -> Result<Layout, DrawError> {
        let (frames, rows, cols) = match *shape {
            [] => (1, 1, 1),
            [c] => (1, 1, c),
            [r, c] => (1, r, c),
            [f, r, c] => (f, r, c),
            _ => return Err(DrawError::Rank(shape.len())),
        };
        let expected = frames * rows * cols;
        if expected == 0 {
            return Err(DrawError::Empty);
        }
        if found != expected {
            return Err(DrawError::Length { expected, found });
        }
        Ok(Layout { frames, rows, cols })
    }

    pub fn per_frame(&self) -> usize {
        self.rows * self.cols
    }
}
