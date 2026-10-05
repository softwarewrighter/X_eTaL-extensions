//! Lines of cells: complete ones in a bounded scrollback, and the line
//! still being written.

use std::collections::VecDeque;

use crate::cell::{Cell, Style};

/// What a program has written, as styled lines.
pub struct Screen {
    lines: VecDeque<Vec<Cell>>,
    current: Vec<Cell>,
    limit: usize,
}

impl Screen {
    /// A screen keeping at most `limit` complete lines.
    pub fn new(limit: usize) -> Self {
        Screen {
            lines: VecDeque::new(),
            current: Vec::new(),
            limit: limit.max(1),
        }
    }

    /// Write `text` in `style`; a newline ends the current line.
    pub fn write(&mut self, text: &str, style: Style) {
        for ch in text.chars() {
            match ch {
                '\n' => {
                    self.lines.push_back(std::mem::take(&mut self.current));
                    while self.lines.len() > self.limit {
                        self.lines.pop_front();
                    }
                }
                '\r' => {}
                ch => self.current.push(Cell { ch, style }),
            }
        }
    }

    /// Empty the screen.
    pub fn clear(&mut self) {
        self.lines.clear();
        self.current.clear();
    }

    /// The last `height` rows of `width` cells: lines wrapped at the
    /// width, the unfinished line last, padded with blanks above.
    pub fn rows(&self, width: usize, height: usize) -> Vec<Vec<Cell>> {
        let width = width.max(1);
        let mut rows: Vec<Vec<Cell>> = Vec::new();
        let lines = self
            .lines
            .iter()
            .chain((!self.current.is_empty()).then_some(&self.current));
        for line in lines {
            match line.is_empty() {
                true => rows.push(Vec::new()),
                false => rows.extend(line.chunks(width).map(<[Cell]>::to_vec)),
            }
        }
        let start = rows.len().saturating_sub(height);
        let mut shown: Vec<Vec<Cell>> = vec![Vec::new(); height.saturating_sub(rows.len())];
        shown.extend(rows.drain(start..));
        for row in &mut shown {
            row.resize(width, Cell::default());
        }
        shown
    }

    /// The text written (complete lines and the current one), for
    /// copying a session.
    pub fn transcript(&self) -> String {
        let text = |line: &Vec<Cell>| line.iter().map(|c| c.ch).collect::<String>();
        let mut out: Vec<String> = self.lines.iter().map(text).collect();
        if !self.current.is_empty() {
            out.push(text(&self.current));
        }
        out.join("\n")
    }
}
