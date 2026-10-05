//! The items of one load (a program or library and the `.xtl`
//! libraries it imports): each definition's place, type, doc comment
//! and uses, grouped by the file it is written in.

use std::collections::{HashMap, HashSet};

use xetal_base::Diagnostic;
use xetal_core::{Expr, Item as CoreItem};
use xetal_doccom::{doc_above, file_doc, section_at};
use xetal_program::{Loaded, located};
use xetal_sources::Sources;

use crate::model::{DocFile, Item, Use, imports_of, kind_of};
use crate::uses::used;

/// A definition found in the Core program.
struct Def<'p> {
    name: String,
    file: usize,
    written: String,
    ty: String,
    value: &'p Expr,
}

/// The files of `loaded`, each with the items written in it.
pub(crate) fn files_of(loaded: &mut Loaded) -> Result<Vec<DocFile>, Diagnostic> {
    let lines =
        xetal_types::check_program(&mut loaded.program).map_err(|d| located(&loaded.sources, d))?;
    let s = &loaded.sources;
    let defs = definitions(&loaded.program.items, &lines, s);
    let index: HashMap<&str, (usize, &str)> = defs
        .iter()
        .map(|d| (d.name.as_str(), (d.file, d.written.as_str())))
        .collect();
    let tops: HashSet<String> = defs.iter().map(|d| d.name.clone()).collect();
    let mut files: Vec<DocFile> = (0..s.file_count()).map(|i| doc_file(s, i)).collect();
    for d in &defs {
        let uses = used(d.value, &tops)
            .iter()
            .filter_map(|n| index.get(n.as_str()).map(|t| (n, *t)))
            .map(|(n, (file, item))| Use {
                written: s.as_written(d.file, n),
                file: s.name(file).to_string(),
                item: item.to_string(),
            })
            .collect::<std::collections::BTreeSet<Use>>()
            .into_iter()
            .collect();
        let public = d.written.contains(':') || files[d.file].kind == "program";
        files[d.file].items.push(item(s, d, public, uses));
    }
    Ok(files)
}

/// The definitions of a program with their types (one line per item).
fn definitions<'p>(items: &'p [CoreItem], lines: &[String], s: &Sources) -> Vec<Def<'p>> {
    items
        .iter()
        .zip(lines)
        .filter_map(|(item, line)| match item {
            CoreItem::Def { name, value } | CoreItem::Let { name, value, .. } => {
                let file = s.locate(value.span.start).index;
                let ty = line.split_once(" : ").map_or("", |(_, t)| t);
                Some(Def {
                    name: name.clone(),
                    file,
                    written: s.as_written(file, name),
                    ty: s.as_written(file, ty),
                    value,
                })
            }
            _ => None,
        })
        .collect()
}

/// A file of `s`, its doc comment read, no items yet.
fn doc_file(s: &Sources, i: usize) -> DocFile {
    let name = s.name(i);
    let kind = match (name.ends_with("System.xtlm"), name.ends_with(".xtlm")) {
        (true, _) => "system macros",
        (false, true) => "macro library",
        _ if xetal_program::is_library(s.written_text(i)) => "library",
        _ => "program",
    };
    DocFile {
        name: name.to_string(),
        kind,
        doc: file_doc(s.written_text(i)),
        imports: imports_of(name, s.written_text(i)),
        items: Vec::new(),
        text: s.written_text(i).to_string(),
    }
}

/// One definition as an item of its file.
fn item(s: &Sources, d: &Def, public: bool, uses: Vec<Use>) -> Item {
    let text = s.written_text(d.file);
    let (start, end) = (
        s.locate(d.value.span.start),
        s.locate(d.value.span.end.saturating_sub(1)),
    );
    let source: Vec<&str> = text
        .lines()
        .skip(start.line - 1)
        .take(end.line + 1 - start.line)
        .collect();
    Item {
        name: d.written.clone(),
        kind: kind_of(&d.written),
        public,
        ty: d.ty.clone(),
        line: start.line,
        section: section_at(text, start.line),
        doc: doc_above(text, start.line),
        source: source.join("\n"),
        uses,
    }
}
