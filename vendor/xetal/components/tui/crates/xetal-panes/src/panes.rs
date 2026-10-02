//! The two panes as one widget, each with its own scroll.

use ratatui::layout::{Constraint, Layout, Position, Rect};
use ratatui::text::Line;
use xetal_buffer::Buffer;
use xetal_view::{column, view, width};

use crate::Scroll;
use crate::lines::{ascii, rendered, split};

/// Which pane has the focus (drawn with a highlighted border).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Focus {
    Source,
    Rendered,
}

/// Both panes for one state of the text.
pub struct Panes {
    pub(crate) source: Vec<Line<'static>>,
    pub(crate) rendered: Vec<Line<'static>>,
    /// Cursor row (in the text) and its column in each pane.
    cursor: (usize, usize, usize),
    pub left: Scroll,
    pub right: Scroll,
    pub focus: Option<Focus>,
    /// Only the focused pane is shown, over the whole area.
    pub zoomed: bool,
}

impl Panes {
    /// The panes for `buffer`, unscrolled, with the byte range `mark`
    /// (an error's span) highlighted in both.
    pub fn new(buffer: &Buffer, mark: Option<(usize, usize)>) -> Self {
        let text = buffer.text();
        let lines = split(&view(&text));
        let (row, col) = buffer.cursor();
        let left = width(&buffer.lines()[row].chars().take(col).collect::<String>());
        let right = lines.get(row).map_or(0, |l| column(l, buffer.offset()));
        Panes {
            source: lines.iter().map(|l| ascii(&text, l, mark)).collect(),
            rendered: lines.iter().map(|l| rendered(l, mark)).collect(),
            cursor: (row, left, right),
            left: Scroll::default(),
            right: Scroll::default(),
            focus: None,
            zoomed: false,
        }
    }

    /// Both scrolls moved just enough to show the cursor in `area`.
    pub fn follow(&mut self, area: Rect) {
        let [l, r] = self.areas(area).map(|p| {
            (
                p.height.saturating_sub(2) as usize,
                p.width.saturating_sub(2) as usize,
            )
        });
        // A pane hidden by zoom keeps its scroll.
        if l.0 * l.1 > 0 {
            self.left = self.left.follow((self.cursor.0, self.cursor.1), l);
        }
        if r.0 * r.1 > 0 {
            self.right = self.right.follow((self.cursor.0, self.cursor.2), r);
        }
    }

    /// Where the cursor is drawn in the source and rendered panes.
    pub fn cursors(&self, area: Rect) -> (Position, Position) {
        let [l, r] = self.areas(area);
        let at = |pane: Rect, s: Scroll, col: usize| {
            let (y, x) = (
                self.cursor.0.saturating_sub(s.row),
                col.saturating_sub(s.col),
            );
            Position::new(pane.x + 1 + x as u16, pane.y + 1 + y as u16)
        };
        (
            at(l, self.left, self.cursor.1),
            at(r, self.right, self.cursor.2),
        )
    }

    /// The ASCII and rendered panes' areas: side by side, or zoomed,
    /// the focused one over all of `area` and the other empty.
    pub fn areas(&self, area: Rect) -> [Rect; 2] {
        let none = Rect::default();
        match (self.zoomed, self.focus) {
            (false, _) => Layout::horizontal([Constraint::Percentage(50); 2]).areas(area),
            (true, Some(Focus::Source)) => [area, none],
            (true, Some(Focus::Rendered)) => [none, area],
            (true, None) => [none, none],
        }
    }

    /// Rows of text and the widest rendered line (for scroll limits).
    pub fn extent(&self) -> (usize, usize) {
        (
            self.rendered.len(),
            self.rendered.iter().map(Line::width).max().unwrap_or(0),
        )
    }
}
