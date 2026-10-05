//! The doc blocks of a file that hold examples: the file's own, then
//! each item's, in order.

use xetal_doc::DocFile;
use xetal_doccom::Example;

/// (item, examples) for each block with examples; None is the file.
pub(crate) fn blocks(file: &DocFile) -> Vec<(Option<String>, Vec<Example>)> {
    let own = file.doc.iter().map(|d| (None, d.examples.clone()));
    let items = file
        .items
        .iter()
        .map(|i| (Some(i.name.clone()), i.examples().to_vec()));
    own.chain(items).filter(|(_, e)| !e.is_empty()).collect()
}
