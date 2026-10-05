//! What every page needs: the files, the resolver, where each item is
//! used, and how a target becomes a URL.

use std::collections::BTreeMap;

use xetal_doc::DocFile;
use xetal_dochtml::{anchor, code, page};
use xetal_doclink::{Resolver, Target, links, uses};

pub(crate) struct Ctx<'a> {
    pub files: &'a [DocFile],
    pub r: Resolver<'a>,
    /// Per item, the (file, line) places it is used.
    pub uses: BTreeMap<Target, Vec<(usize, usize)>>,
}

impl<'a> Ctx<'a> {
    pub(crate) fn new(files: &'a [DocFile]) -> Self {
        let r = Resolver::new(files);
        let uses = uses(&r);
        Ctx { files, r, uses }
    }

    /// The URL of `target`, from any page (the site is flat).
    pub(crate) fn href(&self, target: Target) -> String {
        let file = |f: usize| page(&self.files[f].name);
        match target {
            Target::Item { file: f, item } => {
                format!(
                    "{}.html#{}",
                    file(f),
                    anchor(&self.files[f].items[item].name)
                )
            }
            Target::File(f) => format!("{}.html", file(f)),
            Target::Builtin(b) => format!("builtins.html#{}", anchor(b)),
        }
    }

    /// `text`, written in file `file`, drawn decorated and linked.
    pub(crate) fn draw(&self, file: usize, text: &str) -> String {
        code(text, &links(&self.r, file, text), &|t| self.href(t))
    }

    /// A name drawn decorated, not linked.
    pub(crate) fn name(&self, text: &str) -> String {
        code(text, &[], &|_| String::new())
    }
}
