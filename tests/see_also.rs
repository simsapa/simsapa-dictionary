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

    let sources: Vec<PathBuf> = vec![PathBuf::from("./tests/data/see_also/see_also_simple.md")];

    app::process_markdown_list(sources, &mut ebook).unwrap();

    ebook.process_see_also_from_definition();

    let res = ebook.entries_as_markdown();
    let expect = fs::read_to_string(&PathBuf::from("./tests/data/see_also/see_also_simple_expect.md")).unwrap();

    assert_eq!(res, expect);
}

/*
#[test]
fn see_also_from_definition_parens() {
    let mut ebook = Ebook::new(OutputFormat::Epub, &PathBuf::from("ebook.epub"));

    let sources: Vec<PathBuf> = vec![PathBuf::from("./tests/data/see_also/see_also_parens.md")];

    app::process_markdown_list(sources, &mut ebook).unwrap();

    ebook.process_see_also_from_definition();

    let res = ebook.entries_as_markdown();
    //let mut file = File::create(&PathBuf::from("./tests/data/see_also/see_also_parens_expect.md")).unwrap();
    //file.write_all(res.as_bytes()).unwrap();

    let expect = fs::read_to_string(&PathBuf::from("./tests/data/see_also/see_also_parens_expect.md")).unwrap();
    let expect = "".to_string();

    assert_eq!(res, expect);
}
*/
