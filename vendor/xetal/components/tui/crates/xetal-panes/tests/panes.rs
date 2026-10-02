//! The source and rendered panes, drawn into a test terminal buffer.

use ratatui::Terminal;
use ratatui::backend::TestBackend;
use ratatui::buffer::Buffer as Screen;
use ratatui::layout::Rect;
use xetal_buffer::Buffer;
use xetal_panes::{Focus, Panes, Scroll};

/// The screen as text, one string per row (combining marks kept).
fn rows(screen: &Screen) -> Vec<String> {
    let area = screen.area;
    (0..area.height)
        .map(|y| {
            (0..area.width)
                .map(|x| screen[(x, y)].symbol())
                .collect::<String>()
                .trim_end()
                .to_string()
        })
        .collect()
}

fn draw(buffer: &Buffer, width: u16, height: u16) -> (Vec<String>, Panes) {
    let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
    let panes = Panes::new(buffer, None);
    terminal
        .draw(|f| f.render_widget(&panes, f.area()))
        .unwrap();
    (rows(terminal.backend().buffer()), panes)
}

#[test]
fn ascii_on_the_left_decorated_on_the_right() {
    let (screen, _) = draw(&Buffer::new("u:s_quare := { _r * _r }\nu:s_quare 7"), 64, 4);
    assert!(
        screen[1].starts_with("\u{2502}u:s_quare := { _r * _r }"),
        "{screen:?}"
    );
    assert!(
        screen[1].contains("\u{1d58}s\u{332}quare \u{2190} { \u{2375} \u{d7} \u{2375} }"),
        "{screen:?}"
    );
    assert!(screen[2].contains("\u{1d58}s\u{332}quare 7"), "{screen:?}");
}

#[test]
fn the_cursor_maps_into_both_panes() {
    let mut b = Buffer::new("x^2 + y");
    for _ in 0..4 {
        b.right();
    }
    let (_, panes) = draw(&b, 40, 3);
    let area = Rect::new(0, 0, 40, 3);
    let (left, right) = panes.cursors(area);
    assert_eq!((left.x, left.y), (1 + 4, 1));
    assert_eq!((right.x, right.y), (21 + 3, 1));
}

#[test]
fn text_hidden_in_the_rendered_pane_stays_in_the_ascii_pane() {
    let (screen, _) = draw(&Buffer::new("\"s:\" u_se< \"Stats\" # `;`"), 64, 3);
    assert!(
        screen[1].starts_with("\u{2502}\"s:\" u_se< \"Stats\" # `;`"),
        "{screen:?}"
    );
}

#[test]
fn invalid_text_still_draws() {
    let (screen, _) = draw(&Buffer::new("3-1 r_ev x"), 40, 3);
    assert!(screen[1].contains("3-1 r\u{332}ev x"), "{screen:?}");
}

#[test]
fn a_marked_span_is_highlighted_in_both_panes() {
    let b = Buffer::new("1 + 'x");
    let panes = Panes::new(&b, Some((4, 6)));
    let mut terminal = Terminal::new(TestBackend::new(40, 3)).unwrap();
    terminal
        .draw(|f| f.render_widget(&panes, f.area()))
        .unwrap();
    let screen = terminal.backend().buffer();
    let red = |x: u16| screen[(x, 1)].bg == ratatui::style::Color::Red;
    assert!(red(1 + 4) && red(1 + 5) && !red(1));
    assert!(red(21 + 4) && !red(21));
}

#[test]
fn following_scrolls_rows_and_columns_to_the_cursor() {
    let mut b = Buffer::new("a\nb\nc\nd\n0123456789abcdefghij");
    for _ in 0..4 {
        b.down();
    }
    b.end();
    let mut panes = Panes::new(&b, None);
    let area = Rect::new(0, 0, 20, 4);
    panes.follow(area);
    assert_eq!((panes.left.row, panes.left.col), (3, 13));
    let screen = draw_with(&panes, 20, 4);
    assert!(screen[2].starts_with("\u{2502}defghij"), "{screen:?}");
    let (left, _) = panes.cursors(area);
    assert_eq!((left.x, left.y), (1 + 7, 2));
}

#[test]
fn scrolling_by_hand_stays_inside_the_content() {
    let s = Scroll::default().by((5, -3), (3, 10));
    assert_eq!((s.row, s.col), (2, 0));
    assert_eq!(s.by((-1, 20), (3, 10)), Scroll { row: 1, col: 9 });
}

#[test]
fn symbols_are_light_blue() {
    assert_eq!(
        xetal_panes::style(xetal_view::Class::Symbol).fg,
        Some(ratatui::style::Color::LightBlue)
    );
}

#[test]
fn macros_are_bold_yellow() {
    let style = xetal_panes::style(xetal_view::Class::Macro);
    assert_eq!(style.fg, Some(ratatui::style::Color::Yellow));
    assert!(style.add_modifier.contains(ratatui::style::Modifier::BOLD));
}

#[test]
fn the_focused_pane_has_a_highlighted_border() {
    let mut panes = Panes::new(&Buffer::new("x"), None);
    panes.focus = Some(Focus::Rendered);
    let mut terminal = Terminal::new(TestBackend::new(40, 3)).unwrap();
    terminal
        .draw(|f| f.render_widget(&panes, f.area()))
        .unwrap();
    let screen = terminal.backend().buffer();
    assert_eq!(screen[(20, 0)].fg, ratatui::style::Color::Yellow);
    assert_ne!(screen[(0, 0)].fg, ratatui::style::Color::Yellow);
}

fn draw_with(panes: &Panes, width: u16, height: u16) -> Vec<String> {
    let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
    terminal.draw(|f| f.render_widget(panes, f.area())).unwrap();
    rows(terminal.backend().buffer())
}
