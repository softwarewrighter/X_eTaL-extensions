//! A fixed grid of rows and columns with a cursor (QD6): text written
//! at the cursor, wrapping at the width and scrolling at the bottom,
//! and the screen functions' ANSI sequences (`ansi`) obeyed.

use crate::ansi::{Sequence, read};
use crate::cell::{Cell, Style};

/// The terminal's screen when a program places text and styles it.
pub struct Grid {
    cells: Vec<Vec<Cell>>,
    row: usize,
    col: usize,
    style: Style,
}

impl Grid {
    pub fn new(rows: usize, cols: usize) -> Self {
        let blank = vec![vec![Cell::default(); cols.max(1)]; rows.max(1)];
        Grid {
            cells: blank,
            row: 0,
            col: 0,
            style: Style::default(),
        }
    }

    /// Write `text`, obeying the sequences in it.
    pub fn write(&mut self, text: &str) {
        let mut chars = text.chars().peekable();
        while let Some(ch) = chars.next() {
            match ch {
                '\x1b' => match read(&mut chars) {
                    Some(Sequence::Place(r, c)) => self.place(r, c),
                    Some(Sequence::Clear) => self.clear(),
                    Some(Sequence::Style(codes)) => {
                        self.style = crate::ansi::apply(self.style, &codes)
                    }
                    None => {}
                },
                '\n' => self.newline(),
                '\r' => self.col = 0,
                ch => self.put(ch),
            }
        }
    }

    /// The rows of cells.
    pub fn rows(&self) -> &[Vec<Cell>] {
        &self.cells
    }

    fn put(&mut self, ch: char) {
        if self.col >= self.cells[0].len() {
            self.newline();
        }
        self.cells[self.row][self.col] = Cell {
            ch,
            style: self.style,
        };
        self.col += 1;
    }

    fn newline(&mut self) {
        self.col = 0;
        self.row += 1;
        if self.row == self.cells.len() {
            let cols = self.cells[0].len();
            self.cells.remove(0);
            self.cells.push(vec![Cell::default(); cols]);
            self.row -= 1;
        }
    }

    /// Row and column from 1, kept on the screen.
    fn place(&mut self, r: usize, c: usize) {
        self.row = r.clamp(1, self.cells.len()) - 1;
        self.col = c.clamp(1, self.cells[0].len()) - 1;
    }

    fn clear(&mut self) {
        let (rows, cols) = (self.cells.len(), self.cells[0].len());
        *self = Grid {
            style: self.style,
            ..Grid::new(rows, cols)
        };
    }
}
