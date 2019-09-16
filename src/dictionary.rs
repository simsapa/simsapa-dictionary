use std::default::Default;
use std::process::exit;
use std::fs::File;
use std::io::prelude::*;

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use handlebars::{self, Handlebars};

use crate::markdown::{md2html, markdown_helper};

/// Data store for one dictionary source. Words must be unique, as they are used as the HashMap
/// key.
pub struct Dictionary {
    pub data: DictData,
    pub asset_file_strings: BTreeMap<String, String>,
    pub asset_file_bytes: BTreeMap<String, Vec<u8>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DictData {
    pub dict_header: DictHeader,
    pub entries: BTreeMap<String, DictWord>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DictHeader {
    pub title: String,
    pub dict_label: String,
    pub from_lang: String,
    pub to_lang: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DictWord {
    pub word_header: DictWordHeader,
    pub definition_md: String,
    pub definition_html: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DictWordHeader {
    pub word: String,
    pub summary: String,
    pub grammar: String,
}

impl Dictionary {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, new_word: DictWord) {
        if self.data.entries.contains_key(&new_word.word_header.word) {
            warn!("SKIPPING. Double: '{}' in '{}'. Entries should be unique for word within one dictionary.",
                &new_word.word_header.word,
                &self.data.dict_header.dict_label);

        // TODO insert with modified key
        /*
          let ww = NewDictWord {
          word:             &new_word.word,
          definition_plain: &format!("{} (FIXME: double entry)", new_word.definition_plain),
          definition_html:  &format!("{} (FIXME: double entry)", new_word.definition_html),
          summary:    &new_word.summary,
          grammar:    &new_word.grammar,
          dict_label: &new_word.dict_label,
          from_lang:  &new_word.from_lang,
          to_lang:    &new_word.to_lang,
          };
          info!("Inserting word, double entry: {}", ww.word);
          let _ = create_new_dict_word(&conn, &ww);
        *created_dict_words_count += 1;
        */
        } else {
            info!("Inserting word: {}", new_word.word_header.word);

            if self
                .data
                .entries
                .insert(new_word.word_header.word.clone(), new_word)
                .is_some()
            {
                error!("Unhandled double word, new value replacing the old.");
            }
        }
    }

    pub fn get(&self, word: &str) -> Option<&DictWord> {
        self.data.entries.get(word)
    }

    pub fn len(&self) -> usize {
        self.data.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.entries.is_empty()
    }

    pub fn write_markdown(&self, path: &PathBuf) {
        let mut file = File::create(path).unwrap();

        // Write TOME header with separator.

        let header = toml::to_string(&self.data.dict_header).expect("Can't serialize.");
        let content = format!(
            r#"--- DICTIONARY HEADER ---

``` toml
{}
```

--- DICTIONARY WORD ENTRIES ---

"#,
            &header.trim(),
        );

        file.write_all(content.as_bytes()).unwrap();

        // Write entries.

        let content = self
            .data
            .entries
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
            let file_content = h.render(filename, &self.data).unwrap();

            let mut file = File::create(dir_path.join(filename)).unwrap();
            file.write_all(file_content.as_bytes()).unwrap();
        }

        // Render Handlebar templates wrapped in content-page.xhtml template.

        // entries.xhtml
        // nav.xhtml
        // titlepage.xhtml

        let s = self.asset_file_strings.get("content-page.xhtml").unwrap();
        h.register_template_string("content-page.xhtml", s).unwrap();

        for filename in ["entries.xhtml", "nav.xhtml", "titlepage.xhtml"].iter() {
            let s = self.asset_file_strings.get(&filename.to_string()).unwrap();

            h.register_template_string(filename, s).unwrap();
            let content_html = h.render(filename, &self.data).unwrap();

            let mut d: BTreeMap<String, String> = BTreeMap::new();
            d.insert("page_title".to_string(), self.data.dict_header.title.clone());
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
            d.insert("page_title".to_string(), self.data.dict_header.title.clone());
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

impl DictWord {
    pub fn as_markdown_and_toml_string(&self) -> String {
        let header = toml::to_string(&self.word_header).expect("Can't serialize word header to TOML.");
        let res = format!(
            r#"``` toml
{}
```

{}"#,
            &header.trim(),
            &self.definition_md.trim()
        );

        res
    }

    pub fn from_markdown(s: &str) -> DictWord {
        let a = s.replace("``` toml", "");
        let parts: Vec<&str> = a.split("```").collect();

        let word_header: DictWordHeader = toml::from_str(parts.get(0).unwrap()).unwrap();

        DictWord {
            word_header,
            definition_md: parts.get(1).unwrap().to_string(),
            definition_html: "".to_string(),
        }
    }
}

impl Default for Dictionary {
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

        Dictionary {
            data: DictData::default(),
            asset_file_strings,
            asset_file_bytes,
        }
    }
}

impl Default for DictData {
    fn default() -> Self {
        DictData {
            dict_header: DictHeader::default(),
            entries: BTreeMap::new(),
        }
    }
}

impl Default for DictHeader {
    fn default() -> Self {
        DictHeader {
            title: "Dictionary".to_string(),
            dict_label: "ABCD".to_string(),
            from_lang: "pli".to_string(),
            to_lang: "en".to_string(),
        }
    }
}

