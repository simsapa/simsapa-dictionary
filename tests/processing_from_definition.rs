use std::path::PathBuf;
use std::fs;
//use std::fs::File;
//use std::io::Write;

extern crate simsapa_dictionary;

use simsapa_dictionary::app;
use simsapa_dictionary::ebook::{Ebook, OutputFormat};

#[test]
fn processing_from_definition() {
    let mut ebook = Ebook::new(OutputFormat::Epub, &PathBuf::from("."), &PathBuf::from("ebook.epub"));

    let sources: Vec<PathBuf> = vec![PathBuf::from("./tests/data/processing/processing.md")];

    app::process_markdown_list(sources, &mut ebook).unwrap();

    ebook.process_also_written_as();
    ebook.process_strip_repeat_word_title();
    ebook.process_grammar_note();
    ebook.process_see_also_from_definition(false);

    let res = ebook.entries_as_markdown();
    //let mut file = File::create(&PathBuf::from("./tests/data/processing/processing_expect.md")).unwrap();
    //file.write_all(res.as_bytes()).unwrap();
    //let expect = "".to_string();

    let expect = fs::read_to_string(&PathBuf::from("./tests/data/processing/processing_expect.md")).unwrap();

    assert_eq!(res, expect);
}
