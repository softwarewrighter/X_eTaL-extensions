//! The editor as a state machine: keys in, screen and file out.

use std::path::PathBuf;

use ratatui::Terminal;
use ratatui::backend::TestBackend;
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use xetal_edit::{Editor, Flow};

fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("xetal-edit-{}-{name}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    dir.join("t.xtl")
}

fn keys(e: &mut Editor, text: &str) -> Flow {
    let mut flow = Flow::Continue;
    for c in text.chars() {
        let code = if c == '\n' {
            KeyCode::Enter
        } else {
            KeyCode::Char(c)
        };
        flow = e.handle(KeyEvent::new(code, KeyModifiers::NONE));
    }
    flow
}

fn ctrl(e: &mut Editor, c: char) -> Flow {
    e.handle(KeyEvent::new(KeyCode::Char(c), KeyModifiers::CONTROL))
}

fn screen(e: &Editor) -> String {
    let mut t = Terminal::new(TestBackend::new(70, 12)).unwrap();
    t.draw(|f| e.draw(f)).unwrap();
    let b = t.backend().buffer();
    (0..b.area.height)
        .map(|y| {
            (0..b.area.width)
                .map(|x| b[(x, y)].symbol())
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn a_new_file_is_created_on_save() {
    let path = scratch("new");
    let mut e = Editor::open(&path).unwrap();
    keys(&mut e, "1 + 2");
    assert!(screen(&e).contains("[+]"), "dirty marker");
    ctrl(&mut e, 's');
    assert_eq!(std::fs::read_to_string(&path).unwrap(), "1 + 2\n");
    assert!(!screen(&e).contains("[+]"));
}

#[test]
fn types_show_live_and_results_on_ctrl_r() {
    let mut e = Editor::open(&scratch("types")).unwrap();
    keys(&mut e, "u:s_quare := { _r * _r }\nu:s_quare 7");
    let s = screen(&e);
    assert!(s.contains("u:s_quare : Num a => a -> a"), "{s}");
    assert!(!s.contains("49"), "no evaluation before Ctrl-R: {s}");
    ctrl(&mut e, 'r');
    assert!(screen(&e).contains("49"), "{}", screen(&e));
}

#[test]
fn errors_are_reported_live() {
    let mut e = Editor::open(&scratch("errors")).unwrap();
    keys(&mut e, "1 + 'x");
    let s = screen(&e);
    assert!(s.contains("error[bad-quote]"), "{s}");
}

#[test]
fn quitting_with_unsaved_changes_asks_first() {
    let mut e = Editor::open(&scratch("quit")).unwrap();
    keys(&mut e, "x");
    assert_eq!(ctrl(&mut e, 'q'), Flow::Continue);
    assert!(screen(&e).contains("unsaved"), "{}", screen(&e));
    assert_eq!(ctrl(&mut e, 'q'), Flow::Quit);
}

#[test]
fn an_existing_file_opens_with_both_panes() {
    let path = scratch("open");
    std::fs::write(&path, "x := 2\nx^2\n").unwrap();
    let mut e = Editor::open(&path).unwrap();
    let s = screen(&e);
    assert!(s.contains("x^2") && s.contains("x\u{b2}"), "{s}");
    assert_eq!(ctrl(&mut e, 'q'), Flow::Quit);
}

fn press(e: &mut Editor, code: KeyCode) {
    e.handle(KeyEvent::new(code, KeyModifiers::NONE));
}

fn border_color(e: &Editor, x: u16, y: u16) -> ratatui::style::Color {
    let mut t = Terminal::new(TestBackend::new(70, 12)).unwrap();
    t.draw(|f| e.draw(f)).unwrap();
    t.backend().buffer()[(x, y)].fg
}

#[test]
fn tab_moves_the_focus_through_the_panes() {
    let mut e = Editor::open(&scratch("tab")).unwrap();
    use ratatui::style::Color::Yellow;
    assert_eq!(border_color(&e, 0, 0), Yellow);
    press(&mut e, KeyCode::Tab);
    assert_eq!(border_color(&e, 35, 0), Yellow);
    assert_ne!(border_color(&e, 0, 0), Yellow);
    press(&mut e, KeyCode::Tab);
    assert_eq!(border_color(&e, 0, 5), Yellow);
    e.handle(KeyEvent::new(KeyCode::BackTab, KeyModifiers::SHIFT));
    assert_eq!(border_color(&e, 35, 0), Yellow);
}

#[test]
fn arrows_scroll_the_focused_rendered_pane() {
    let mut e = Editor::open(&scratch("hscroll")).unwrap();
    keys(
        &mut e,
        "1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21 22 23 24 25",
    );
    press(&mut e, KeyCode::Home);
    let before = screen(&e);
    press(&mut e, KeyCode::Tab);
    for _ in 0..6 {
        press(&mut e, KeyCode::Right);
    }
    let after = screen(&e);
    let row = |s: &str| {
        s.lines()
            .nth(1)
            .unwrap()
            .chars()
            .skip(35)
            .collect::<String>()
    };
    assert!(row(&before).starts_with("\u{2502}1 2 3"), "{before}");
    assert!(
        row(&after).starts_with("\u{2503}4 5 6"),
        "focused: thick {after}"
    );
    keys(&mut e, "0");
    assert!(
        screen(&e)
            .lines()
            .nth(1)
            .unwrap()
            .starts_with("\u{2503}01 2"),
        "typing returns to the source"
    );
}

#[test]
fn the_output_pane_scrolls() {
    let mut e = Editor::open(&scratch("vscroll")).unwrap();
    keys(&mut e, "1\n2\n3\n4\n5\n6\n7");
    ctrl(&mut e, 'r');
    e.handle(KeyEvent::new(KeyCode::BackTab, KeyModifiers::SHIFT));
    press(&mut e, KeyCode::Down);
    press(&mut e, KeyCode::Down);
    let s = screen(&e);
    let bottom: Vec<&str> = s.lines().skip(6).take(4).collect();
    assert!(bottom[0].starts_with("\u{2503}3"), "{s}");
}

#[test]
fn results_show_as_grids_with_type_and_shape() {
    let mut e = Editor::open(&scratch("grids")).unwrap();
    keys(&mut e, "2 3 r_eshape r_ange 6");
    ctrl(&mut e, 'r');
    let s = screen(&e);
    assert!(s.contains("Int 2 3"), "{s}");
    assert!(s.contains("\u{2502} 1 2 3 \u{2502}"), "{s}");
}

#[test]
fn a_program_using_a_library_checks_and_runs() {
    let path = scratch("library");
    std::fs::write(&path, "\"s:\" u_se< \"Stats\"\ns:m_ean 1 2 3\n").unwrap();
    let mut e = Editor::open(&path).unwrap();
    assert!(screen(&e).contains("Float"), "{}", screen(&e));
    ctrl(&mut e, 'r');
    assert!(screen(&e).contains("2.0  : Float"), "{}", screen(&e));
}

fn cell(e: &Editor, x: u16, y: u16) -> ratatui::buffer::Cell {
    let mut t = Terminal::new(TestBackend::new(70, 12)).unwrap();
    t.draw(|f| e.draw(f)).unwrap();
    t.backend().buffer()[(x, y)].clone()
}

#[test]
fn ctrl_t_zooms_the_focused_pane_and_tab_switches_the_view() {
    let mut e = Editor::open(&scratch("zoom")).unwrap();
    keys(&mut e, "1 + 2");
    let all = screen(&e);
    assert!(all.contains("ASCII") && all.contains("Rendered") && all.contains("Types"));
    ctrl(&mut e, 't');
    let only = |s: &str, title: &str| {
        ["ASCII", "Rendered", "Types"]
            .iter()
            .all(|t| s.contains(t) == (*t == title))
    };
    assert!(only(&screen(&e), "ASCII"), "{}", screen(&e));
    assert_eq!(
        cell(&e, 69, 0).symbol(),
        "\u{2513}",
        "one pane, the full width"
    );
    press(&mut e, KeyCode::Tab);
    assert!(only(&screen(&e), "Rendered"), "{}", screen(&e));
    press(&mut e, KeyCode::Tab);
    assert!(only(&screen(&e), "Types"), "{}", screen(&e));
    press(&mut e, KeyCode::Tab);
    assert!(only(&screen(&e), "ASCII"));
    ctrl(&mut e, 't');
    assert_eq!(screen(&e), all, "back to three panes");
}

#[test]
fn the_focused_pane_has_a_thick_border_and_a_marked_title() {
    let mut e = Editor::open(&scratch("thick")).unwrap();
    assert_eq!(cell(&e, 0, 0).symbol(), "\u{250f}", "thick corner");
    assert_eq!(cell(&e, 35, 0).symbol(), "\u{250c}", "plain corner");
    assert!(screen(&e).contains("\u{25b6} ASCII"), "{}", screen(&e));
    press(&mut e, KeyCode::Tab);
    assert_eq!(cell(&e, 0, 0).symbol(), "\u{250c}");
    assert_eq!(cell(&e, 35, 0).symbol(), "\u{250f}");
    assert!(screen(&e).contains("\u{25b6} Rendered"));
}

#[test]
fn the_cursor_cell_is_drawn_reversed_in_both_panes() {
    use ratatui::style::Modifier;
    let mut e = Editor::open(&scratch("cursor")).unwrap();
    keys(&mut e, "ab");
    press(&mut e, KeyCode::Left);
    let (ascii, rendered) = (cell(&e, 2, 1), cell(&e, 37, 1));
    assert!(ascii.modifier.contains(Modifier::REVERSED), "{ascii:?}");
    assert_eq!(ascii.symbol(), "b");
    assert!(
        rendered.modifier.contains(Modifier::REVERSED),
        "{rendered:?}"
    );
    assert!(!cell(&e, 1, 1).modifier.contains(Modifier::REVERSED));
}
