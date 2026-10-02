//! Scroll offsets: following the cursor, or moved by hand.

/// The first row and column shown in a pane.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Scroll {
    pub row: usize,
    pub col: usize,
}

impl Scroll {
    /// Scrolled just enough to show `(row, col)` in a view of
    /// `(height, width)`.
    pub fn follow(self, (row, col): (usize, usize), (height, width): (usize, usize)) -> Scroll {
        let fit = |first: usize, at: usize, size: usize| {
            first.min(at).max((at + 1).saturating_sub(size.max(1)))
        };
        Scroll {
            row: fit(self.row, row, height),
            col: fit(self.col, col, width),
        }
    }

    /// Moved by `(rows, cols)`, staying within `(rows, cols)` of content.
    pub fn by(self, (dr, dc): (isize, isize), (rows, cols): (usize, usize)) -> Scroll {
        let step = |at: usize, d: isize, max: usize| {
            at.saturating_add_signed(d).min(max.saturating_sub(1))
        };
        Scroll {
            row: step(self.row, dr, rows),
            col: step(self.col, dc, cols),
        }
    }
}
