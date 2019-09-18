use std::process::exit;
use std::fs::File;
use std::io::prelude::*;

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use chrono::prelude::*;
use handlebars::{self, Handlebars};

use crate::dict_word::{DictWord, DictWordHeader};
use crate::helpers::{md2html, markdown_helper};

pub const DICTIONARY_METADATA_SEP: &str = "--- DICTIONARY METADATA ---";
pub const DICTIONARY_WORD_ENTRIES_SEP: &str = "--- DICTIONARY WORD ENTRIES ---";

#[derive(Serialize, Deserialize)]
pub struct Ebook {
    pub meta: EbookMetadata,
    pub dict_words: BTreeMap<String, DictWord>,
    pub asset_file_strings: BTreeMap<String, String>,
    pub asset_file_bytes: BTreeMap<String, Vec<u8>>,
}

#[derive(Serialize, Deserialize)]
pub struct EbookMetadata {
    pub title: String,
    pub description: String,
    pub creator: String,
    pub source: String,
    pub cover_path: String,
    pub book_id: String,
    pub date: String,
}

impl Ebook {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_word(&mut self, new_word: DictWord) {
        let w_key = format!("{} {}", new_word.word_header.word, new_word.word_header.dict_label);

        if self.dict_words.contains_key(&w_key) {
            warn!("Double word: '{}'. Entries should be unique for word within one dictionary.", &w_key);

          let ww = DictWord {
              word_header: DictWordHeader {
                  dict_label: new_word.word_header.dict_label,
                  word: format!("{} FIXME: double", new_word.word_header.word),
                  summary: new_word.word_header.summary,
                  grammar: new_word.word_header.grammar,
              },
              definition_md: new_word.definition_md
          };
          let ww_key = format!("{} double", &w_key);
          self.dict_words.insert(ww_key, ww);

        } else {
            let w = self.dict_words.insert(w_key.clone(), new_word);
            if w.is_some() {
                error!("Unhandled double word '{}', new value replacing the old.", w_key);
            }
        }
    }

    pub fn get_word(&self, word: &str) -> Option<&DictWord> {
        self.dict_words.get(word)
    }

    pub fn len(&self) -> usize {
        self.dict_words.len()
    }

    pub fn is_empty(&self) -> bool {
        self.dict_words.is_empty()
    }

    pub fn write_markdown(&self, path: &PathBuf) {
        let mut file = File::create(path).unwrap();

        // Write TOML metadata with separator.

        let meta = toml::to_string(&self.meta).expect("Can't serialize.");
        let content = format!(
            "{}\n\n``` toml\n{}\n```\n\n{}\n\n",
            &DICTIONARY_METADATA_SEP,
            &meta.trim(),
            &DICTIONARY_WORD_ENTRIES_SEP,
        );

        file.write_all(content.as_bytes()).unwrap();

        // Write entries.

        let content = self
            .dict_words
            .values()
            .map(|i| i.as_markdown_and_toml_string())
            .collect::<Vec<String>>()
            .join("\n\n");

        file.write_all(content.as_bytes()).unwrap();
    }

    pub fn write_oepbs_files(&self, dir_path: &Path) {
        if !dir_path.is_dir() {
            error!("dir_path must be a directory.");
            exit(2);
        }

        let mut h = Handlebars::new();
        h.register_helper("markdown", Box::new(markdown_helper));

        // Render direct Handlebar templates.

        // package.opf

        {
            let filename = "package.opf";
            let s = self.asset_file_strings.get(&filename.to_string()).unwrap();

            h.register_template_string(filename, s).unwrap();
            let file_content = h.render(filename, &self).unwrap();

            let mut file = File::create(dir_path.join(filename)).unwrap();
            file.write_all(file_content.as_bytes()).unwrap();
        }

        // Render Handlebar templates wrapped in content-page.xhtml template.

        let s = self.asset_file_strings.get("content-page.xhtml").unwrap();
        h.register_template_string("content-page.xhtml", s).unwrap();

        // entries.xhtml
        // nav.xhtml
        // titlepage.xhtml

        for filename in ["entries.xhtml", "nav.xhtml", "titlepage.xhtml"].iter() {
            let s = self.asset_file_strings.get(&filename.to_string()).unwrap();

            h.register_template_string(filename, s).unwrap();
            let content_html = match h.render(filename, &self) {
                Ok(x) => x,
                Err(e) => {
                    error!("Can't render template {}, {:?}", filename, e);
                    "FIXME: Template rendering error.".to_string()
                }
            };

            let mut d: BTreeMap<String, String> = BTreeMap::new();
            d.insert("page_title".to_string(), self.meta.title.clone());
            d.insert("content_html".to_string(), content_html);
            let file_content = h.render("content-page.xhtml", &d).unwrap();

            let mut file = File::create(dir_path.join(filename)).unwrap();
            file.write_all(file_content.as_bytes()).unwrap();
        }

        // Render Markdown content wrapped in content-page.xhtml template.

        // about.md
        // copyright.md

        for filename in ["about.md", "copyright.md"].iter() {
            let content_md = self.asset_file_strings.get(&filename.to_string()).unwrap();
            let content_html = md2html(&content_md);

            let mut d: BTreeMap<String, String> = BTreeMap::new();
            d.insert("page_title".to_string(), self.meta.title.clone());
            d.insert("content_html".to_string(), content_html);
            let file_content = h.render("content-page.xhtml", &d).unwrap();

            let dest_name = filename.replace(".md", ".xhtml");
            let mut file = File::create(dir_path.join(dest_name)).unwrap();
            file.write_all(file_content.as_bytes()).unwrap();
        }

        // Copy static assets.

        // cover.jpg
        // style.css

        for filename in ["cover.jpg", "style.css"].iter() {
            let file_content = self.asset_file_bytes.get(&filename.to_string()).unwrap();
            let mut file = File::create(dir_path.join(filename)).unwrap();
            file.write_all(file_content).unwrap();
        }

    }
}

impl Default for Ebook {
    fn default() -> Self {

        let mut asset_file_strings: BTreeMap<String, String> = BTreeMap::new();
        let mut asset_file_bytes: BTreeMap<String, Vec<u8>> = BTreeMap::new();

        asset_file_strings.insert(
            "content-page.xhtml".to_string(),
            include_str!("../assets/content-page.xhtml").to_string());

        asset_file_strings.insert(
            "about.md".to_string(),
            include_str!("../assets/OEPBS/about.md").to_string());

        asset_file_strings.insert(
            "copyright.md".to_string(),
            include_str!("../assets/OEPBS/copyright.md").to_string());

        asset_file_strings.insert(
            "entries.xhtml".to_string(),
            include_str!("../assets/OEPBS/entries.xhtml").to_string());

        asset_file_strings.insert(
            "nav.xhtml".to_string(),
            include_str!("../assets/OEPBS/nav.xhtml").to_string());

        asset_file_strings.insert(
            "package.opf".to_string(),
            include_str!("../assets/OEPBS/package.opf").to_string());

        asset_file_strings.insert(
            "titlepage.xhtml".to_string(),
            include_str!("../assets/OEPBS/titlepage.xhtml").to_string());

        asset_file_bytes.insert(
            "cover.jpg".to_string(),
            include_bytes!("../assets/OEPBS/cover.jpg").to_vec());

        asset_file_bytes.insert(
            "style.css".to_string(),
            include_bytes!("../assets/OEPBS/style.css").to_vec());

        // TODO build manifest

        Ebook {
            meta: EbookMetadata::default(),
            dict_words: BTreeMap::new(),
            asset_file_strings,
            asset_file_bytes,
        }
    }
}

impl Default for EbookMetadata {
    fn default() -> Self {
        EbookMetadata {
            title: "Dictionary".to_string(),
            description: "Pali - English".to_string(),
            creator: "Simsapa Dhamma Reader".to_string(),
            source: "https://simsapa.github.io".to_string(),
            cover_path: "cover.jpg".to_string(),
            book_id: "SimsapaPaliDictionary".to_string(),
            date: Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        }
    }
}
