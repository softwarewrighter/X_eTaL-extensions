//! Drawing the panes: each in its frame (the focused one marked), with
//! the cursor cell reversed in both so it shows in any terminal.

use ratatui::buffer::Buffer as Screen;
use ratatui::layout::{Position, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::widgets::{Paragraph, Widget};

use crate::theme::frame;
use crate::{Focus, Panes, Scroll};

impl Widget for &Panes {
    fn render(self, area: Rect, screen: &mut Screen) {
        let [l, r] = self.areas(area);
        let at = |s: Scroll| (s.row as u16, s.col as u16);
        let focused = |pane| self.focus == Some(pane);
        if l.area() > 0 {
            let block = frame("ASCII", focused(Focus::Source));
            let left = Paragraph::new(self.source.clone()).block(block);
            left.scroll(at(self.left)).render(l, screen);
        }
        if r.area() > 0 {
            let block = frame("Rendered", focused(Focus::Rendered));
            let right = Paragraph::new(self.rendered.clone()).block(block);
            right.scroll(at(self.right)).render(r, screen);
        }
        let (in_l, in_r) = self.cursors(area);
        reverse(screen, in_l, l);
        reverse(screen, in_r, r);
    }
}

/// Reverse the cell at `at` if it is inside `pane`'s frame.
fn reverse(screen: &mut Screen, at: Position, pane: Rect) {
    let inner = Rect::new(
        pane.x + 1,
        pane.y + 1,
        pane.width.saturating_sub(2),
        pane.height.saturating_sub(2),
    );
    if inner.contains(at) {
        screen[at].set_style(Style::default().add_modifier(Modifier::REVERSED));
    }
}
