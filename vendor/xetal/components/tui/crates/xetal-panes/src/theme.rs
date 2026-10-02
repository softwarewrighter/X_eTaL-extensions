//! Colors by token class (the same palette as `xetal render --color`):
//! system functions blue (symbols light blue), the program's green,
//! a library's cyan; a quote has the color of what it quotes.

use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Span;
use ratatui::widgets::{Block, BorderType};
use xetal_view::Class;

pub fn style(class: Class) -> Style {
    let fg = |c: Color| Style::default().fg(c);
    match class {
        Class::Builtin => fg(Color::Blue),
        Class::Macro => fg(Color::Yellow).add_modifier(Modifier::BOLD),
        Class::UserFunc => fg(Color::Green),
        Class::LibFunc => fg(Color::Cyan),
        Class::LambdaArg => fg(Color::Magenta),
        Class::Number | Class::Exponent => fg(Color::Yellow),
        Class::String => fg(Color::LightYellow),
        Class::Symbol => fg(Color::LightBlue),
        Class::Quote => Style::default().add_modifier(Modifier::BOLD),
        Class::Comment => Style::default().add_modifier(Modifier::DIM),
        Class::Error => fg(Color::Red).add_modifier(Modifier::UNDERLINED),
        Class::Variable | Class::Punct | Class::Unit | Class::Space => Style::default(),
    }
}

/// A pane's frame: the focused pane has a thick bright border and a
/// reversed title marked with a triangle; the others are dim.
pub fn frame(title: &str, focused: bool) -> Block<'static> {
    if !focused {
        let dim = Style::default().fg(Color::DarkGray);
        return Block::bordered()
            .border_style(dim)
            .title(format!(" {title} "));
    }
    let lit = Style::default()
        .fg(Color::Yellow)
        .add_modifier(Modifier::BOLD);
    let marked = Span::styled(
        format!(" \u{25b6} {title} "),
        lit.add_modifier(Modifier::REVERSED),
    );
    Block::bordered()
        .border_type(BorderType::Thick)
        .border_style(lit)
        .title(marked)
}
