use std::path::PathBuf;
use std::fs;
//use std::fs::File;
//use std::io::Write;

extern crate simsapa_dictionary;

use simsapa_dictionary::app;
use simsapa_dictionary::ebook::{Ebook, OutputFormat};

#[test]
fn see_also_from_definition_simple() {
    let mut ebook = Ebook::new(OutputFormat::Epub, &PathBuf::from("ebook.epub"));

    let sources: Vec<PathBuf> = vec![PathBuf::from("./tests/see_also_simple.md")];

    app::process_markdown_list(sources, &mut ebook).unwrap();

    ebook.process_see_also_from_definition();

    let res = ebook.entries_as_markdown();
    let expect = fs::read_to_string(&PathBuf::from("./tests/see_also_simple_expect.md")).unwrap();

    assert_eq!(res, expect);
}
