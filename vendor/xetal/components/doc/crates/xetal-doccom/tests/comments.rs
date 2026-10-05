//! Doc comments (lang-choices S9): which `##` block documents what.

use xetal_doccom::{Doc, Example, doc_above, file_doc, section_at};

const LIB: &str = "\
## A small library of greetings.
## Two functions.

# u_se< is not here: a plain comment, never a doc.

### Saying hello

## The greeting for a name.
## >> l:h_ello \"Ann\"
## Hello, Ann
l:h_ello := { n -> \"Hello, \" c_at n }

## Shout it.
##
## Upper case would be nicer.
l:s_hout := { n -> n c_at \"!\" }
helper := 3

### Numbers

## Twice.
## >> l:t_wice 2
## 4
## >> l:t_wice 1 2
## 2 4
l:t_wice := { _r * 2 }
";

fn line_of(needle: &str) -> usize {
    LIB.lines()
        .position(|l| l.starts_with(needle))
        .expect("line")
        + 1
}

#[test]
fn the_first_block_followed_by_a_blank_line_documents_the_file() {
    let doc = file_doc(LIB).expect("file doc");
    assert_eq!(doc.text, "A small library of greetings.\nTwo functions.");
}

#[test]
fn a_block_directly_above_a_definition_documents_it() {
    let doc = doc_above(LIB, line_of("l:h_ello")).expect("doc");
    assert_eq!(doc.text, "The greeting for a name.");
    assert_eq!(
        doc.examples,
        vec![Example {
            code: "l:h_ello \"Ann\"".into(),
            output: "Hello, Ann".into()
        }]
    );
}

#[test]
fn a_bare_double_hash_is_a_paragraph_break() {
    let doc = doc_above(LIB, line_of("l:s_hout")).expect("doc");
    assert_eq!(doc.text, "Shout it.\n\nUpper case would be nicer.");
}

#[test]
fn a_definition_without_a_block_has_no_doc() {
    assert_eq!(doc_above(LIB, line_of("helper")), None);
}

#[test]
fn examples_end_at_the_next_example() {
    let doc = doc_above(LIB, line_of("l:t_wice")).expect("doc");
    assert_eq!(doc.examples.len(), 2);
    assert_eq!(
        doc.examples[1],
        Example {
            code: "l:t_wice 1 2".into(),
            output: "2 4".into()
        }
    );
}

#[test]
fn sections_group_the_definitions_after_them() {
    assert_eq!(
        section_at(LIB, line_of("l:h_ello")).as_deref(),
        Some("Saying hello")
    );
    assert_eq!(
        section_at(LIB, line_of("helper")).as_deref(),
        Some("Saying hello")
    );
    assert_eq!(
        section_at(LIB, line_of("l:t_wice")).as_deref(),
        Some("Numbers")
    );
}

#[test]
fn a_plain_comment_is_never_a_doc() {
    let text = "# plain\nx := 1\n";
    assert_eq!(doc_above(text, 2), None);
    assert_eq!(file_doc(text), None);
}

#[test]
fn a_first_block_attached_to_a_definition_documents_it_not_the_file() {
    let text = "## Only for x.\nx := 1\n";
    assert_eq!(file_doc(text), None);
    assert_eq!(
        doc_above(text, 2),
        Some(Doc {
            text: "Only for x.".into(),
            examples: vec![]
        })
    );
}

#[test]
fn double_hash_without_a_space_is_not_a_doc_comment() {
    assert_eq!(doc_above("##x\nx := 1\n", 2), None);
}

#[test]
fn the_file_doc_is_at_the_top_after_any_shebang() {
    let text = "#!/usr/bin/env xetal\n\n## The program.\n\nx := 1\n";
    assert_eq!(
        file_doc(text).map(|d| d.text).as_deref(),
        Some("The program.")
    );
    let later = "x := 1\n\n## Not the file's.\n\ny := 2\n";
    assert_eq!(file_doc(later), None);
}

#[test]
fn an_expected_error_is_shown_as_the_output() {
    let text = "## >> l:h_ello 1\n## error[type-mismatch]\nl:h_ello := { n -> n }\n";
    let doc = doc_above(text, 3).expect("doc");
    assert_eq!(doc.examples[0].output, "error[type-mismatch]");
}
