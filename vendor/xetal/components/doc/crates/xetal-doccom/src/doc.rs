//! A doc comment's prose and examples.

/// One `## >> code` example and the output shown under it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Example {
    pub code: String,
    pub output: String,
}

/// A doc comment: its prose (paragraphs split by a blank line) and its
/// examples, in order.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Doc {
    pub text: String,
    pub examples: Vec<Example>,
}

impl Doc {
    /// The doc read from a block's lines, each with its `##` removed.
    pub(crate) fn from_lines(lines: &[&str]) -> Doc {
        let mut doc = Doc::default();
        let mut prose: Vec<&str> = Vec::new();
        let mut example: Option<(String, Vec<&str>)> = None;
        for line in lines {
            if let Some(code) = line.strip_prefix(">> ") {
                doc.close(example.take());
                example = Some((code.trim().to_string(), Vec::new()));
            } else if let (Some((_, out)), false) = (example.as_mut(), line.is_empty()) {
                out.push(line);
            } else {
                doc.close(example.take());
                prose.push(line);
            }
        }
        doc.close(example);
        doc.text = prose.join("\n").trim_matches('\n').to_string();
        doc
    }

    fn close(&mut self, example: Option<(String, Vec<&str>)>) {
        if let Some((code, out)) = example {
            let output = out.join("\n");
            self.examples.push(Example { code, output });
        }
    }
}
