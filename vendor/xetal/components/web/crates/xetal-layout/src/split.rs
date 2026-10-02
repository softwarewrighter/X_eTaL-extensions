//! Where the panes split: the ASCII pane's share of the width and the
//! output pane's share of the height, in percent.

/// Which split a divider moves.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Axis {
    Columns,
    Rows,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Split {
    pub source: f64,
    pub output: f64,
}

impl Default for Split {
    fn default() -> Self {
        Split {
            source: 50.0,
            output: 32.0,
        }
    }
}

/// No pane is ever squeezed below this share, nor grown above its
/// complement.
const LEAST: f64 = 10.0;

fn clamped(percent: f64) -> f64 {
    percent.clamp(LEAST, 100.0 - LEAST)
}

impl Split {
    /// The CSS custom properties the panes' grid is laid out by.
    pub fn style(&self) -> String {
        format!("--source: {}%; --output: {}%;", self.source, self.output)
    }

    /// The split with a divider dragged to `at` (a client coordinate)
    /// over panes spanning `start..start + length` along that axis:
    /// the ASCII pane gets the width left of it, the output the height
    /// below it.
    pub fn dragged(self, axis: Axis, at: f64, start: f64, length: f64) -> Split {
        if length <= 0.0 {
            return self;
        }
        let before = (at - start) / length * 100.0;
        match axis {
            Axis::Columns => Split {
                source: clamped(before),
                ..self
            },
            Axis::Rows => Split {
                output: clamped(100.0 - before),
                ..self
            },
        }
    }

    /// The split as saved, for example "62.5 20".
    pub fn to_text(&self) -> String {
        format!("{} {}", self.source, self.output)
    }

    /// A saved split, or the default when the text is not one.
    pub fn from_text(text: &str) -> Split {
        let numbers: Vec<f64> = text.split(' ').filter_map(|n| n.parse().ok()).collect();
        match numbers[..] {
            [source, output] => Split {
                source: clamped(source),
                output: clamped(output),
            },
            _ => Split::default(),
        }
    }
}
