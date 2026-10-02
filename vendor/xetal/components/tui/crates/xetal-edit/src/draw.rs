//! The screen: both panes, the types or output pane, a status line.

use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::text::Line;
use ratatui::widgets::Paragraph;
use xetal_panes::{Focus, Panes, frame};

use crate::{Editor, Pane};

impl Editor {
    /// Draw the editor into `frame`; the cursor shows in the ASCII pane
    /// while it has the focus.
    pub fn draw(&self, frame: &mut Frame) {
        let [top, bottom, status] = self.areas(frame.area());
        let mut panes = Panes::new(&self.buffer, self.report.mark);
        panes.zoomed = self.zoom;
        (panes.left, panes.right) = (self.left.get(), self.right.get());
        panes.focus = match self.focus {
            Pane::Source => Some(Focus::Source),
            Pane::Rendered => Some(Focus::Rendered),
            Pane::Output => None,
        };
        if self.focus == Pane::Source {
            panes.follow(top);
            self.left.set(panes.left);
            self.right.set(panes.right);
            frame.set_cursor_position(panes.cursors(top).0);
        }
        frame.render_widget(&panes, top);
        if bottom.area() > 0 {
            frame.render_widget(self.bottom(), bottom);
        }
        frame.render_widget(Paragraph::new(self.status_line()), status);
    }

    /// The two panes, the bottom pane and the status line; zoomed, the
    /// focused pane takes all but the status line.
    fn areas(&self, full: Rect) -> [Rect; 3] {
        let none = Rect::default();
        if self.zoom {
            let [main, status] =
                Layout::vertical([Constraint::Min(3), Constraint::Length(1)]).areas(full);
            return match self.focus {
                Pane::Output => [none, main, status],
                _ => [main, none, status],
            };
        }
        Layout::vertical([
            Constraint::Min(3),
            Constraint::Length(6),
            Constraint::Length(1),
        ])
        .areas(full)
    }

    fn bottom(&self) -> Paragraph<'_> {
        let title = if self.ran { "Output" } else { "Types" };
        let lines: Vec<Line> = self
            .report
            .lines
            .iter()
            .map(|l| Line::from(l.as_str()))
            .collect();
        let at = (self.out.get().row as u16, self.out.get().col as u16);
        Paragraph::new(lines)
            .block(frame(title, self.focus == Pane::Output))
            .scroll(at)
    }

    fn status_line(&self) -> String {
        let dirty = if self.buffer.is_dirty() { " [+]" } else { "" };
        let name = self.path.file_name().map_or(self.path.as_os_str(), |n| n);
        format!(" {}{dirty}  {}", name.to_string_lossy(), self.status)
    }
}
