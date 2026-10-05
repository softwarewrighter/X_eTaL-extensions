//! The terminal's screen (Saga 25), after web-sw-tos's `ui.rs`: text
//! written as lines of styled character cells, wrapped at the width,
//! kept in a bounded scrollback, shown as the last rows that fit; and
//! a fixed grid with a cursor for programs that place and style text
//! (QD6), obeying the screen functions' ANSI sequences.

mod ansi;
mod cell;
mod grid;
mod screen;

pub use cell::{Cell, Color, Style};
pub use grid::Grid;
pub use screen::Screen;
