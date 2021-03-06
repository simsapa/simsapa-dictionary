use std::error::Error;
use std::fs::{self, File};
use std::io::{self, Write};
use std::process::Command;

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::convert::TryInto;

use handlebars::{self, Handlebars};
use walkdir::WalkDir;
use regex::Regex;
use deunicode::deunicode;
use xlsxwriter::{Workbook, Worksheet, FormatColor, Format};
use serde_json::{Value, Map};

use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;

use crate::app::{self, AppStartParams, ZipWith};
use pali_dict_core::dict_word::{DictWord, DictWordMarkdown, DictWordXlsx};
use crate::error::ToolError;
use crate::helpers::{self, is_hidden, md2html, uppercase_first_letter};
use pali_dict_core::letter_groups::{LetterGroups, LetterGroup};
use pali_dict_core::pali;
use crate::db_schema;
use crate::db_models::{DbDictionary, NewDictionary, DbDictWord, NewDictWord, DbMeaning, NewMeaning,
DbGrammar, NewGrammar, DbExample, NewExample};

pub const DICTIONARY_METADATA_SEP: &str = "--- DICTIONARY METADATA ---";
pub const DICTIONARY_WORD_ENTRIES_SEP: &str = "--- DICTIONARY WORD ENTRIES ---";

#[derive(Serialize, Deserialize)]
pub struct Dictionary {
    pub meta: DictMetadata,
    pub output_format: OutputFormat,

    /// Words as serialized from the input formats, Markdown or XLSX. The map key is `word_header.url_id`.
    pub dict_words_input: BTreeMap<String, DictWordMarkdown>,

    /// Words as processed for rendering in the templates. The map key is `word_header.url_id`.
    pub dict_words_render: BTreeMap<String, DictWord>,

    /// Collects the list of valid word names which can be linked to.
    #[serde(skip)]
    pub valid_words: Vec<String>,

    #[serde(skip)]
    pub words_to_url: BTreeMap<String, String>,

    pub entries_manifest: Vec<EntriesManifest>,
    pub asset_files_string: BTreeMap<String, String>,
    pub asset_files_byte: BTreeMap<String, Vec<u8>>,

    #[serde(skip)]
    pub output_path: PathBuf,

    #[serde(skip)]
    pub entries_template: Option<PathBuf>,

    /// The folder of the first source input file.
    #[serde(skip)]
    pub source_dir: PathBuf,

    /// Build base dir is 'ebook-build' in the folder of the source input file.
    #[serde(skip)]
    pub build_base_dir: Option<PathBuf>,

    #[serde(skip)]
    pub mimetype_path: Option<PathBuf>,
    #[serde(skip)]
    pub meta_inf_dir: Option<PathBuf>,
    #[serde(skip)]
    pub oebps_dir: Option<PathBuf>,
    #[serde(skip)]
    pub templates: Handlebars,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct DictMetadata {

    pub title: String,

    #[serde(default)]
    pub dict_label: String,

    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub creator: String,
    #[serde(default)]
    pub email: String,
    /// Source URL
    #[serde(default)]
    pub source: String,
    #[serde(default)]
    pub cover_path: String,
    #[serde(default)]
    pub book_id: String,
    #[serde(default)]
    pub version: String,
    #[serde(default)]
    pub created_date_human: String,
    #[serde(default)]
    pub created_date_opf: String,
    #[serde(default)]
    pub word_prefix: String,
    #[serde(default)]
    pub word_prefix_velthuis: bool,
    #[serde(default)]
    pub add_velthuis: bool,
    #[serde(default)]
    pub allow_raw_html: bool,
    #[serde(default)]
    pub dont_generate_synonyms: bool,
}

#[derive(Serialize, Deserialize, Copy, Clone)]
pub enum OutputFormat {
    Epub,
    Mobi,
    BabylonGls,
    StardictXmlPlain,
    StardictXmlHtml,
    LaTeXPlain,
    C5Plain,
    C5Html,
    TeiPlain,
    TeiFormatted,
}

#[derive(Serialize, Deserialize)]
pub struct EntriesManifest {
    id: String,
    href: String,
}

#[derive(Serialize, Deserialize)]
pub struct LetterGroupTemplateData {
    group: LetterGroup,
    meta: DictMetadata,
}

impl Dictionary {
    pub fn new(
        output_format: OutputFormat,
        allow_raw_html: bool,
        source_dir: &Path,
        output_path: &Path,
        entries_template: Option<PathBuf>,
        ) -> Self
    {
        // asset_files_string
        let mut afs: BTreeMap<String, String> = BTreeMap::new();
        // asset_files_byte
        let mut afb: BTreeMap<String, Vec<u8>> = BTreeMap::new();
        let mut h = Handlebars::new();
        h.set_strict_mode(true);
        h.register_escape_fn(helpers::light_html_escape);

        h.register_helper("markdown", Box::new(helpers::markdown_helper));
        h.register_helper("countitems", Box::new(helpers::countitems));
        h.register_helper("to_velthuis", Box::new(helpers::to_velthuis));
        h.register_helper("word_title", Box::new(helpers::word_title));
        h.register_helper("cover_media_type", Box::new(helpers::cover_media_type));
        h.register_helper("headword_plain", Box::new(helpers::headword_plain));
        h.register_helper("word_list", Box::new(helpers::word_list));
        h.register_helper("word_list_plain", Box::new(helpers::word_list_plain));
        h.register_helper("word_list_tei", Box::new(helpers::word_list_tei));
        h.register_helper("grammar_text", Box::new(helpers::grammar_text));
        h.register_helper("grammar_text_plain", Box::new(helpers::grammar_text_plain));
        h.register_helper("phonetic_transliteration", Box::new(helpers::phonetic_transliteration));
        h.register_helper("phonetic_transliteration_plain", Box::new(helpers::phonetic_transliteration_plain));

        // Can't loop because the arg of include_str! must be a string literal.

        let k = "content-page.xhtml".to_string();
        afs.insert(
            k.clone(),
            include_str!("../assets/content-page.xhtml").to_string(),
        );
        reg_tmpl(&mut h, &k, &afs);

        let k = "about.md".to_string();
        afs.insert(
            k.clone(),
            include_str!("../assets/OEBPS/about.md").to_string(),
        );
        reg_tmpl(&mut h, &k, &afs);

        let k = "copyright.md".to_string();
        afs.insert(
            k.clone(),
            include_str!("../assets/OEBPS/copyright.md").to_string(),
        );
        reg_tmpl(&mut h, &k, &afs);

        let k = "entries-epub.xhtml".to_string();
        afs.insert(
            k.clone(),
            include_str!("../assets/OEBPS/entries-epub.xhtml").to_string(),
        );
        reg_tmpl(&mut h, &k, &afs);

        let k = "entries-mobi.xhtml".to_string();
        afs.insert(
            k.clone(),
            include_str!("../assets/OEBPS/entries-mobi.xhtml").to_string(),
        );
        reg_tmpl(&mut h, &k, &afs);

        let k = "htmltoc.xhtml".to_string();
        afs.insert(
            k.clone(),
            include_str!("../assets/OEBPS/htmltoc.xhtml").to_string(),
        );
        reg_tmpl(&mut h, &k, &afs);

        let k = "toc.ncx".to_string();
        afs.insert(
            k.clone(),
            include_str!("../assets/OEBPS/toc.ncx").to_string(),
        );
        reg_tmpl(&mut h, &k, &afs);

        let k = "package.opf".to_string();
        afs.insert(
            k.clone(),
            include_str!("../assets/OEBPS/package.opf").to_string(),
        );
        reg_tmpl(&mut h, &k, &afs);

        let k = "cover.xhtml".to_string();
        afs.insert(
            k.clone(),
            include_str!("../assets/OEBPS/cover.xhtml").to_string(),
        );
        reg_tmpl(&mut h, &k, &afs);

        let k = "titlepage.xhtml".to_string();
        afs.insert(
            k.clone(),
            include_str!("../assets/OEBPS/titlepage.xhtml").to_string(),
        );
        reg_tmpl(&mut h, &k, &afs);

        let k = "stardict_textual_plain.xml".to_string();
        afs.insert(
            k.clone(),
            include_str!("../assets/stardict_textual_plain.xml").to_string(),
        );
        reg_tmpl(&mut h, &k, &afs);

        let k = "stardict_textual_html.xml".to_string();
        afs.insert(
            k.clone(),
            include_str!("../assets/stardict_textual_html.xml").to_string(),
        );
        reg_tmpl(&mut h, &k, &afs);

        let k = "c5_plain.txt".to_string();
        afs.insert(
            k.clone(),
            include_str!("../assets/c5_plain.txt").to_string(),
        );
        reg_tmpl(&mut h, &k, &afs);

        let k = "c5_html.txt".to_string();
        afs.insert(
            k.clone(),
            include_str!("../assets/c5_html.txt").to_string(),
        );
        reg_tmpl(&mut h, &k, &afs);

        let k = "freedict-tei_plain.xml".to_string();
        afs.insert(
            k.clone(),
            include_str!("../assets/freedict-tei_plain.xml").to_string(),
        );
        reg_tmpl(&mut h, &k, &afs);

        let k = "freedict-tei_formatted.xml".to_string();
        afs.insert(
            k.clone(),
            include_str!("../assets/freedict-tei_formatted.xml").to_string(),
        );
        reg_tmpl(&mut h, &k, &afs);

        let k = "latex_plain.tex".to_string();
        afs.insert(
            k.clone(),
            include_str!("../assets/latex_plain.tex").to_string(),
        );
        reg_tmpl(&mut h, &k, &afs);

        // binary storage (not templates)

        afb.insert(
            "default_cover.jpg".to_string(),
            include_bytes!("../assets/OEBPS/default_cover.jpg").to_vec(),
        );

        afb.insert(
            "style.css".to_string(),
            include_bytes!("../assets/OEBPS/style.css").to_vec(),
        );

        afb.insert(
            "container.xml".to_string(),
            include_bytes!("../assets/META-INF/container.xml").to_vec(),
        );

        afb.insert(
            "com.apple.ibooks.display-options.xml".to_string(),
            include_bytes!("../assets/META-INF/com.apple.ibooks.display-options.xml").to_vec(),
        );

        let mut meta = DictMetadata::default();
        meta.allow_raw_html = allow_raw_html;

        Dictionary {
            meta,
            output_format,
            dict_words_input: BTreeMap::new(),
            dict_words_render: BTreeMap::new(),
            valid_words: Vec::new(),
            words_to_url: BTreeMap::new(),
            entries_manifest: Vec::new(),
            asset_files_string: afs,
            asset_files_byte: afb,
            output_path: output_path.to_path_buf(),
            source_dir: source_dir.to_path_buf(),
            entries_template,
            build_base_dir: None,
            mimetype_path: None,
            meta_inf_dir: None,
            oebps_dir: None,
            templates: h,
        }
    }

    pub fn reuse_metadata(&mut self) -> Result<(), Box<dyn Error>> {
        let (meta_txt, _) = app::split_metadata_and_entries(&self.output_path)?;
        self.meta = app::parse_str_to_metadata(&meta_txt)?;
        Ok(())
    }

    /// Add transliterations to help searching:
    /// - given with the transliteration attribute
    /// - velthuis
    /// - ascii
    pub fn process_add_transliterations(&mut self) {
        if self.meta.dont_generate_synonyms {
            return;
        }

        info!("process_add_transliterations()");

        for (_, dict_word) in self.dict_words_input.iter_mut() {

            if !dict_word.word_header.transliteration.is_empty() {
                dict_word.word_header.inflections.push(dict_word.word_header.transliteration.clone());
            }

            if self.meta.add_velthuis {
                let s = pali::to_velthuis(&dict_word.word_header.word);
                if !dict_word.word_header.inflections.contains(&s) && s != dict_word.word_header.word {
                    dict_word.word_header.inflections.push(s);
                }
            }

            {
                let s = deunicode(&dict_word.word_header.word);
                if !s.contains("[?]") && !dict_word.word_header.inflections.contains(&s) && s != dict_word.word_header.word {
                    dict_word.word_header.inflections.push(s);
                }
            }
        }
    }

    pub fn use_cli_overrides(&mut self, app_params: &AppStartParams) {
        if let Some(ref title) = app_params.title {
            self.meta.title = title.clone();
        }

        if let Some(ref cover_path) = app_params.cover_path {
            self.meta.cover_path = cover_path.clone();
        }

        if let Some(ref prefix) = app_params.word_prefix {
            self.meta.word_prefix = prefix.clone();
        }

        self.meta.word_prefix_velthuis = app_params.word_prefix_velthuis;
        self.meta.allow_raw_html = app_params.allow_raw_html;
        self.meta.dont_generate_synonyms = app_params.dont_generate_synonyms;

        if let Some(ref dict_label) = app_params.dict_label {
            self.meta.dict_label = dict_label.clone();
            for (_key, word) in self.dict_words_input.iter_mut() {
                word.word_header.dict_label = dict_label.clone();
            }
        }
    }

    pub fn add_word(&mut self, new_word: DictWordMarkdown) {
        let mut new_word = new_word;

        // Sanitize the word name.

        // Trim whitespace.
        new_word.word_header.word = new_word.word_header.word.trim().to_string();

        // Remove the root prefix used by some authors, but keep it as an inflection, so that it
        // may be still matched when searching Goldendict.

        let a = new_word.word_header.word.clone();
        if a.contains('√') {
            let w = new_word.word_header.word.trim_start_matches('√').to_string();
            new_word.word_header.inflections.push(a);
            new_word.word_header.word = w;
        }

        // Remove the root sign from `grammar_roots`.

        for w in new_word.word_header.grammar_roots.iter_mut() {
            *w = w.trim_start_matches('√').to_string();
        }

        // If the word contains a trailing digit, strip it and use it as `meaning_order`.
        lazy_static! {
            static ref RE_TRAIL_NUM: Regex = Regex::new(r" *([0-9]+) *$").unwrap();
        }

        let mut w = "".to_string();
        for caps in RE_TRAIL_NUM.captures_iter(&new_word.word_header.word) {
            let a = caps.get(1).unwrap().as_str().to_string();

            match a.trim().to_string().parse::<usize>() {
                Ok(x) => new_word.word_header.meaning_order = x,
                Err(_) => break,
            }

            w = new_word.word_header.word
                .trim_end_matches(&a)
                .trim()
                .to_string();
        }
        if !w.is_empty() {
            new_word.word_header.word = w;
        }

        if new_word.word_header.meaning_order == 0 {
            new_word.word_header.meaning_order = 1;
        }
        new_word.set_url_id();

        if !self.valid_words.contains(&new_word.word_header.word) {
            self.valid_words.push(new_word.word_header.word.clone());
        }

        while self.dict_words_input.contains_key(&new_word.word_header.url_id) {
            new_word.word_header.meaning_order += 1;
            new_word.set_url_id();
        }

        let id = new_word.word_header.url_id.clone();
        let w = self.dict_words_input.insert(new_word.word_header.url_id.clone(), new_word);
        if w.is_some() {
            error!("Unhandled double word '{}', new value replacing the old.", id);
        }
    }

    pub fn get_word(&self, word: &str) -> Option<&DictWordMarkdown> {
        self.dict_words_input.get(word)
    }

    pub fn len(&self) -> usize {
        self.dict_words_input.len()
    }

    pub fn is_empty(&self) -> bool {
        self.dict_words_input.is_empty()
    }

    pub fn entries_as_markdown(&self) -> String {
        info!("entries_as_markdown()");
        self.dict_words_input
            .values()
            .map(|i| i.as_markdown_and_toml_string())
            .collect::<Vec<String>>()
            .join("\n\n")
    }

    pub fn write_markdown(&self) -> Result<(), Box<dyn Error>> {
        info!("write_markdown()");

        let mut file = File::create(&self.output_path)?;

        // Write TOML metadata with separator.

        let meta = toml::to_string(&self.meta)?;
        let content = format!(
            "{}\n\n``` toml\n{}\n```\n\n{}\n\n",
            &DICTIONARY_METADATA_SEP,
            &meta.trim(),
            &DICTIONARY_WORD_ENTRIES_SEP,
        );

        file.write_all(content.as_bytes())?;

        // Write entries.

        file.write_all(self.entries_as_markdown().as_bytes())?;

        Ok(())
    }

    pub fn create_ebook_build_folders(&mut self) -> Result<(), Box<dyn Error>> {
        info!("create_ebook_build_folders()");

        if self.output_path.exists() {
            fs::remove_file(&self.output_path)?;
        }

        // Store full paths (canonicalized) in the Dictionary attribs. canonicalize() requires that the
        // path should exist, so take the parent of output_path first before canonicalize().

        let parent = match self.output_path.parent() {
            Some(p) => {
                let s = p.to_str().unwrap();
                if s.trim().is_empty() {
                    PathBuf::from(".")
                } else {
                    p.to_path_buf()
                }
            }
            None => PathBuf::from("."),
        };
        let build_base_dir = match parent.canonicalize() {
            Ok(p) => p.join("ebook-build"),
            Err(e) => {
                let msg = format!("Can't canonicalize: {:?}\nError: {:?}", parent, e);
                return Err(Box::new(ToolError::Exit(msg)));
            }
        };
        if !build_base_dir.exists() {
            match fs::create_dir(&build_base_dir) {
                Ok(_) => {}
                Err(e) => {
                    let msg = format!(
                        "Can't create directory: {:?}\nError: {:?}",
                        build_base_dir, e
                    );
                    return Err(Box::new(ToolError::Exit(msg)));
                }
            };
        }
        self.build_base_dir = Some(build_base_dir.clone());

        if let OutputFormat::Epub = self.output_format {
            self.mimetype_path = Some(build_base_dir.join("mimetype"));

            let meta_inf_dir = build_base_dir.join("META-INF");
            if !meta_inf_dir.exists() {
                fs::create_dir(&meta_inf_dir)?;
            }
            self.meta_inf_dir = Some(meta_inf_dir);
        }

        let oebps_dir = build_base_dir.join("OEBPS");
        if !oebps_dir.exists() {
            fs::create_dir(&oebps_dir)?;
        }
        self.oebps_dir = Some(oebps_dir);

        Ok(())
    }

    /// Write entries split in letter groups.
    pub fn write_entries(&mut self) -> Result<(), Box<dyn Error>> {
        info!("write_entries()");

        let w: Vec<DictWordMarkdown> = self.dict_words_input.values().cloned().collect();
        let mut letter_groups = LetterGroups::new_from_dict_words(&w);

        info!("Writing {} letter groups ...", letter_groups.len());

        let template_name = match self.output_format {
            OutputFormat::Epub => "entries-epub.xhtml",
            OutputFormat::Mobi => "entries-mobi.xhtml",
            _ => {
                let msg = "🔥 Only Epub or Mobi makes sense here.".to_string();
                return Err(Box::new(ToolError::Exit(msg)));
            }
        };

        for (order_idx, group) in letter_groups.groups.values_mut().enumerate() {
            info!("{}...", order_idx + 1);
            if order_idx == 0 {
                group.title = self.meta.title.clone();
            }

            let data = LetterGroupTemplateData {
                group: group.clone(),
                meta: self.meta.clone(),
            };

            let content_html = match self.templates.render(template_name, &data) {
                Ok(x) => x,
                Err(e) => {
                    error!("Can't render template {}, {:?}", template_name, e);
                    "FIXME: Template rendering error.".to_string()
                }
            };

            let mut d: BTreeMap<String, String> = BTreeMap::new();
            d.insert("page_title".to_string(), self.meta.title.clone());
            d.insert("content_html".to_string(), content_html);
            let file_content = self.templates.render("content-page.xhtml", &d)?;

            // The file names are not sequential (00, 01, 02 ...), they are identified by the index
            // number of the Pali letter from pali::romanized_pali_letter_index().

            let group_file_name = format!("entries-{:02}.xhtml", group.letter_index);
            self.entries_manifest.push(EntriesManifest {
                id: format!("item_entries_{:02}", group.letter_index),
                href: group_file_name.clone(),
            });

            let dir = self.oebps_dir.as_ref().ok_or("missing oebps_dir")?;
            let mut file = File::create(dir.join(group_file_name))?;
            file.write_all(file_content.as_bytes())?;
        }

        Ok(())
    }

    /// Write package.opf.
    pub fn write_package(&mut self) -> Result<(), Box<dyn Error>> {
        info!("write_package()");

        let filename = "package.opf";
        let file_content = self.templates.render(filename, &self)?;

        let dir = self.oebps_dir.as_ref().ok_or("missing oebps_dir")?;
        let mut file = File::create(dir.join(filename))?;
        file.write_all(file_content.as_bytes())?;

        Ok(())
    }

    /// Write htmltoc.xhtml.
    pub fn write_html_toc(&mut self) -> Result<(), Box<dyn Error>> {
        info!("write_html_toc()");

        let filename = "htmltoc.xhtml".to_string();

        let content_html = match self.templates.render(&filename, &self) {
            Ok(x) => x,
            Err(e) => {
                error!("Can't render template {}, {:?}", filename, e);
                "FIXME: Template rendering error.".to_string()
            }
        };

        let mut d: BTreeMap<String, String> = BTreeMap::new();
        d.insert("page_title".to_string(), self.meta.title.clone());
        d.insert("content_html".to_string(), content_html);
        let file_content = self.templates.render("content-page.xhtml", &d)?;

        let dir = self.oebps_dir.as_ref().ok_or("missing oebps_dir")?;
        let mut file = File::create(dir.join(filename))?;
        file.write_all(file_content.as_bytes())?;

        Ok(())
    }

    /// Write toc.ncx.
    pub fn write_ncx_toc(&mut self) -> Result<(), Box<dyn Error>> {
        info!("write_ncx_toc()");

        let filename = "toc.ncx".to_string();

        let file_content = match self.templates.render(&filename, &self) {
            Ok(x) => x,
            Err(e) => {
                error!("Can't render template {}, {:?}", filename, e);
                "FIXME: Template rendering error.".to_string()
            }
        };

        let dir = self.oebps_dir.as_ref().ok_or("missing oebps_dir")?;
        let mut file = File::create(dir.join(filename))?;
        file.write_all(file_content.as_bytes())?;

        Ok(())
    }

    /// Write cover.xhtml.
    pub fn write_cover(&mut self) -> Result<(), Box<dyn Error>> {
        info!("write_cover()");

        let filename = "cover.xhtml".to_string();

        let file_content = match self.templates.render(&filename, &self) {
            Ok(x) => x,
            Err(e) => {
                error!("Can't render template {}, {:?}", filename, e);
                "FIXME: Template rendering error.".to_string()
            }
        };

        let dir = self.oebps_dir.as_ref().ok_or("missing oebps_dir")?;
        let mut file = File::create(dir.join(filename))?;
        file.write_all(file_content.as_bytes())?;

        Ok(())
    }

    /// Write titlepage.xhtml.
    pub fn write_titlepage(&mut self) -> Result<(), Box<dyn Error>> {
        info!("write_titlepage()");

        let filename = "titlepage.xhtml".to_string();

        let content_html = match self.templates.render(&filename, &self) {
            Ok(x) => x,
            Err(e) => {
                error!("Can't render template {}, {:?}", filename, e);
                "FIXME: Template rendering error.".to_string()
            }
        };

        let mut d: BTreeMap<String, String> = BTreeMap::new();
        d.insert("page_title".to_string(), self.meta.title.clone());
        d.insert("content_html".to_string(), content_html);
        let file_content = self.templates.render("content-page.xhtml", &d)?;

        let dir = self.oebps_dir.as_ref().ok_or("missing oebps_dir")?;
        let mut file = File::create(dir.join(filename))?;
        file.write_all(file_content.as_bytes())?;

        Ok(())
    }

    /// Write about.xhtml.
    pub fn write_about(&mut self) -> Result<(), Box<dyn Error>> {
        info!("write_about()");

        let filename = "about.md".to_string();

        let content_md = match self.templates.render(&filename, &self) {
            Ok(x) => x,
            Err(e) => {
                error!("Can't render template {}, {:?}", filename, e);
                "FIXME: Template rendering error.".to_string()
            }
        };

        let content_html = md2html(&content_md, self.meta.allow_raw_html);

        let mut d: BTreeMap<String, String> = BTreeMap::new();
        d.insert("page_title".to_string(), self.meta.title.clone());
        d.insert("content_html".to_string(), content_html);
        let file_content = self.templates.render("content-page.xhtml", &d)?;

        let dest_name = filename.replace(".md", ".xhtml");
        let dir = self.oebps_dir.as_ref().ok_or("missing oebps_dir")?;
        let mut file = File::create(dir.join(dest_name))?;
        file.write_all(file_content.as_bytes())?;

        Ok(())
    }

    /// Write copyright.xhtml.
    pub fn write_copyright(&mut self) -> Result<(), Box<dyn Error>> {
        info!("write_copyright()");

        let filename = "copyright.md".to_string();

        let content_md = match self.templates.render(&filename, &self) {
            Ok(x) => x,
            Err(e) => {
                error!("Can't render template {}, {:?}", filename, e);
                "FIXME: Template rendering error.".to_string()
            }
        };

        let content_html = md2html(&content_md, self.meta.allow_raw_html);

        let mut d: BTreeMap<String, String> = BTreeMap::new();
        d.insert("page_title".to_string(), self.meta.title.clone());
        d.insert("content_html".to_string(), content_html);
        let file_content = self.templates.render("content-page.xhtml", &d)?;

        let dest_name = filename.replace(".md", ".xhtml");
        let dir = self.oebps_dir.as_ref().ok_or("missing oebps_dir")?;
        let mut file = File::create(dir.join(dest_name))?;
        file.write_all(file_content.as_bytes())?;

        Ok(())
    }

    /// Copy static assets.
    pub fn copy_static(&self) -> Result<(), Box<dyn Error>> {
        info!("copy_static()");

        let oebps_dir = self.oebps_dir.as_ref().ok_or("missing oebps_dir")?;

        // cover image
        {
            // Cover path is relative to the folder of the source input file.
            let rel_cover = PathBuf::from(self.meta.cover_path.clone());
            let filename = PathBuf::from(rel_cover.file_name().unwrap());
            let p = self.source_dir.join(rel_cover);
            if p.exists() {
                // If the file is found, copy that.
                fs::copy(&p, oebps_dir.join(filename))?;
            } else {
                // If not found, try looking it up in the embedded assets.
                let file_content = self
                    .asset_files_byte
                    .get(filename.to_str().unwrap())
                    .ok_or_else(|| format!("cover_path not found: {}", self.meta.cover_path))?;
                let mut file = File::create(oebps_dir.join(filename))?;
                file.write_all(file_content)?;
            }
        }

        // stylesheet
        {
            let filename = "style.css";
            let file_content = self
                .asset_files_byte
                .get(&filename.to_string())
                .ok_or("style.css not found")?;
            let mut file = File::create(oebps_dir.join(filename))?;
            file.write_all(file_content)?;
        }

        Ok(())
    }

    pub fn write_mimetype(&self) -> Result<(), Box<dyn Error>> {
        info!("write_mimetype()");

        let p = self.mimetype_path.as_ref().ok_or("missing mimetype_path")?;
        let mut file = File::create(&p)?;
        file.write_all(b"application/epub+zip")?;

        Ok(())
    }

    pub fn write_meta_inf_files(&self) -> Result<(), Box<dyn Error>> {
        info!("write_meta_inf_files()");

        let dir = self.meta_inf_dir.as_ref().ok_or("missing meta_inf_dir")?;
        for filename in ["container.xml", "com.apple.ibooks.display-options.xml"].iter() {
            let file_content = self
                .asset_files_byte
                .get(&(*filename).to_string())
                .ok_or("missing get key")?;
            let mut file = File::create(dir.join(filename))?;
            file.write_all(file_content)?;
        }

        Ok(())
    }

    pub fn write_oebps_files(&mut self) -> Result<(), Box<dyn Error>> {
        info!("write_oebps_files()");

        self.copy_static()?;

        // The cover path is used without folder.
        self.meta.cover_path = PathBuf::from(self.meta.cover_path.clone())
            .file_name().unwrap()
            .to_str().unwrap()
            .to_string();

        if let OutputFormat::Epub = self.output_format {
            self.write_cover()?;
        }

        self.write_entries()?;
        self.write_package()?;
        self.write_html_toc()?;
        self.write_ncx_toc()?;

        self.write_titlepage()?;
        self.write_about()?;
        self.write_copyright()?;

        Ok(())
    }

    fn zip_with_shell(&self) -> Result<(), Box<dyn Error>> {
        info!("zip_with_shell()");

        let d = self
            .build_base_dir
            .as_ref()
            .ok_or("missing build_base_dir")?;
        let dir: &str = d.to_str().unwrap();
        let n = self.output_path.file_name().ok_or("mising file_name")?;
        let epub_name: &str = n.to_str().unwrap();

        let shell_cmd = format!(r#"cd "{}" && zip -X0 "../{}" mimetype && zip -rg "../{}" META-INF -x \*.DS_Store && zip -rg "../{}" OEBPS -x \*.DS_Store"#, dir, epub_name, epub_name, epub_name);

        let output = match Command::new("sh").arg("-c").arg(shell_cmd).output() {
            Ok(o) => o,
            Err(e) => {
                let msg = format!("🔥 Failed to Zip: {:?}", e);
                return Err(Box::new(ToolError::Exit(msg)));
            }
        };

        if output.status.success() {
            info!("🔎 Zip successful.");
        } else {
            error!("🔥 Zip exited with an error.");
        }

        Ok(())
    }

    fn zip_with_lib(&self) -> Result<(), Box<dyn Error>> {
        info!("zip_with_lib()");

        let mut buf: Vec<u8> = Vec::new();
        {
            let mut w = std::io::Cursor::new(&mut buf);
            let mut zip = zip::ZipWriter::new(&mut w);

            // NOTE: Path names in Epub Zip files must always use '/' forward-slash, even on Windows.

            // mimetype file first, not compressed
            {
                let o = zip::write::FileOptions::default()
                    .compression_method(zip::CompressionMethod::Stored);
                zip.start_file("mimetype", o)?;
                zip.write_all(b"application/epub+zip")?;
            }

            // META-INF folder
            //
            // Only has two files, which are not templated, no need to read them. Retreive from
            // asset_files_byte.

            {
                let o = zip::write::FileOptions::default();

                zip.start_file("META-INF/com.apple.ibooks.display-options.xml", o)?;
                zip.write_all(
                    self.asset_files_byte
                        .get("com.apple.ibooks.display-options.xml")
                        .unwrap(),
                )?;

                zip.start_file("META-INF/container.xml", o)?;
                zip.write_all(self.asset_files_byte.get("container.xml").unwrap())?;
            }

            // OEBPS folder
            //
            // Walk the contents. Not recursive, we are storing them all in one folder.

            {
                let o = zip::write::FileOptions::default();
                let dir = self.oebps_dir.as_ref().ok_or("missing oebps dir")?;
                let walker = WalkDir::new(dir).into_iter();

                // is_hidden will also catch .DS_Store
                for entry in walker.filter_entry(|e| !is_hidden(e)) {
                    let entry = entry?;

                    // First entry will be the OEBPS folder.
                    if entry.file_name().to_str().unwrap() == "OEBPS" {
                        continue;
                    }
                    if entry.path().is_dir() {
                        info!("Skipping dir entry '{}'", entry.path().to_str().unwrap());
                        continue;
                    }

                    let contents: Vec<u8> = fs::read(&entry.path())?;

                    let name = entry.file_name().to_str().unwrap();
                    // not using .join() to avoid getting a back-slash on Windows
                    zip.start_file(format!("OEBPS/{}", name), o)?;
                    zip.write_all(&contents)?;
                }
            }

            zip.finish()?;
        }

        let mut file = File::create(&self.output_path)?;
        file.write_all(&buf)?;

        Ok(())
    }

    pub fn zip_files_as_epub(&self, zip_with: ZipWith) -> Result<(), Box<dyn Error>> {
        info!("zip_files_as_epub()");

        match zip_with {
            ZipWith::ZipLib => self.zip_with_lib(),
            ZipWith::ZipCli => self.zip_with_shell(),
        }
    }

    pub fn run_kindlegen(
        &self,
        kindlegen_path: &Path,
        mobi_compression: usize,
    ) -> Result<(), Box<dyn Error>> {
        info!("run_kindlegen()");

        let oebps_dir = self.oebps_dir.as_ref().ok_or("missing oebps_dir")?;
        let opf_path = oebps_dir.join(PathBuf::from("package.opf"));
        let output_file_name = self.output_path.file_name().ok_or("can't get file_name")?;

        info!("🔎 Running KindleGen ...");
        if mobi_compression == 2 {
            println!("NOTE: Using compression level 2 (Kindle huffdic). This can take some time to complete.");
        }

        let output = if cfg!(target_os = "windows") {
            match Command::new("cmd")
                .arg("/C")
                .arg(kindlegen_path)
                .arg(opf_path)
                .arg(format!("-c{}", mobi_compression))
                .arg("-dont_append_source")
                .arg("-o")
                .arg(output_file_name)
                .output()
            {
                Ok(o) => o,
                Err(e) => {
                    let msg = format!("🔥 Failed to run KindleGen: {:?}", e);
                    return Err(Box::new(ToolError::Exit(msg)));
                }
            }
        } else {
            // sh expects a command string after -c.
            let cmd_string = format!(
                "{} \"{}\" -c{} -dont_append_source -o \"{}\"",
                kindlegen_path.to_str().unwrap(),
                opf_path.to_str().unwrap(),
                mobi_compression,
                output_file_name.to_str().unwrap()
            );

            match Command::new("sh").arg("-c").arg(cmd_string).output() {
                Ok(o) => o,
                Err(e) => {
                    let msg = format!("🔥 Failed to run KindleGen: {:?}", e);
                    return Err(Box::new(ToolError::Exit(msg)));
                }
            }
        };

        io::stdout().write_all(&output.stdout)?;
        io::stderr().write_all(&output.stderr)?;

        if output.status.success() {
            info!("🔎 KindleGen finished successfully.");
        } else {
            let msg = "🔥 KindleGen exited with an error.".to_string();
            return Err(Box::new(ToolError::Exit(msg)));
        }

        // Move the generate MOBI to its path. KindleGen puts the MOBI in the same folder with package.opf.
        fs::rename(oebps_dir.join(output_file_name), &self.output_path)?;

        Ok(())
    }

    pub fn remove_generated_files(&mut self) -> Result<(), Box<dyn Error>> {
        if let Some(dir) = self.build_base_dir.as_ref() {
            fs::remove_dir_all(dir)?;
        }
        self.set_paths_to_none();

        Ok(())
    }

    pub fn create_ebook(&mut self, app_params: &AppStartParams) -> Result<(), Box<dyn Error>> {
        self.create_ebook_build_folders()?;

        match self.output_format {
            OutputFormat::Epub => {
                self.write_mimetype()?;
                self.write_meta_inf_files()?;
                self.write_oebps_files()?;
                self.zip_files_as_epub(app_params.zip_with)?;
            }

            OutputFormat::Mobi => {
                self.write_oebps_files()?;

                if !app_params.dont_run_kindlegen {
                    let kindlegen_path = &app_params
                        .kindlegen_path
                        .as_ref()
                        .ok_or("kindlegen_path is missing.")?;
                    self.run_kindlegen(&kindlegen_path, app_params.mobi_compression)?;
                }
            }

            _ => {
                let msg = "🔥 Use create_ebook() only for Epub and Mobi.".to_string();
                return Err(Box::new(ToolError::Exit(msg)));
            }
        }

        Ok(())
    }

    pub fn write_babylon_source(&self) -> Result<(), Box<dyn Error>> {

        let mut content = String::new();

        // Must start with a blank line.
        content.push_str("\n");

        // Write the header.
        content.push_str(&format!(r#"#stripmethod=keep
#sametypesequence=h
#bookname={}
#author={}
#email={}
#description={}
#website={}
#date={}"#,
&self.meta.title,
&self.meta.creator,
&self.meta.email,
&self.meta.description,
&self.meta.source,
&self.meta.created_date_opf));

        // Write the entries.
        for (_, word) in self.dict_words_input.iter() {
            // Blank line before each entry, including the first.
            content.push_str("\n\n");

            // start with the word
            content.push_str(&word.word_header.word);

            // inflections
            if !word.word_header.inflections.is_empty() {
                content.push_str(&'|'.to_string());
                let inflections = word.word_header.inflections.join(&'|'.to_string());
                content.push_str(&inflections);
            }
            content.push_str(&'\n'.to_string());

            let mut text = String::new();

            if !word.word_header.dict_label.is_empty() {
                text.push_str(&format!("<p>[{}]</p>", &word.word_header.dict_label));
            }

            // let h = serde_json::to_value(&word.word_header).unwrap();

            // FIXME grammar
            // let s = helpers::grammar_text_html();
            // text.push_str(&s);

            // FIXME phonetic, transliteration
            // let s = helpers::format_phonetic_transliteration_html(&h, self.meta.add_velthuis);
            // text.push_str(&s);

            // Also written as
            if !word.word_header.also_written_as.is_empty() {
                let s: String = word.word_header.also_written_as.join(", ");
                text.push_str(&format!("<p>Also written as: {}</p>", &s));
            }

            // Definition
            text.push_str(&helpers::md2html(&word.definition_md, self.meta.allow_raw_html));

            // Synonyms
            if !word.word_header.synonyms.is_empty() {
                let s: String = word.word_header.synonyms.join(", ");
                text.push_str(&format!("<p>Synonyms: {}</p>", &s));
            }

            // Antonyms
            if !word.word_header.antonyms.is_empty() {
                let s: String = word.word_header.antonyms.join(", ");
                text.push_str(&format!("<p>Antonyms: {}</p>", &s));
            }

            // See also
            if !word.word_header.see_also.is_empty() {
                let s: String = word.word_header.see_also.join(", ");
                text.push_str(&format!("<p>See also: {}</p>", &s));
            }

            content.push_str(&text.replace('\n', ""));
        }

        // End with a blank line.
        content.push_str("\n\n");

        let mut file = File::create(&self.output_path)?;
        file.write_all(content.as_bytes())?;

        Ok(())
    }

    pub fn create_babylon(&mut self) -> Result<(), Box<dyn Error>> {
        self.write_babylon_source()?;
        Ok(())
    }

    pub fn write_stardict_xml(&self) -> Result<(), Box<dyn Error>> {
        info!("write_stardict_xml()");

        let mut content = match self.entries_template {
            Some(ref path) => {
                let template_source = match fs::read_to_string(path) {
                    Ok(x) => x,
                    Err(e) => {
                        let msg = format!("Can't read file: {:?}, {:?}", path, e);
                        return Err(Box::new(ToolError::Exit(msg)));
                    }
                };

                let mut h = Handlebars::new();
                h.set_strict_mode(true);
                h.register_escape_fn(helpers::light_html_escape);
                match h.render_template(&template_source, &self) {
                    Ok(x) => x,
                    Err(e) => {
                        let msg = format!("Can't render template: {:?}", e);
                        return Err(Box::new(ToolError::Exit(msg)));
                    }
                }
            }

            None => {
                let template = match self.output_format {
                    OutputFormat::StardictXmlPlain => "stardict_textual_plain.xml".to_string(),
                    OutputFormat::StardictXmlHtml => "stardict_textual_html.xml".to_string(),
                    _ => {
                        let msg = "🔥 Only StardictXmlPlain or StardictXmlHtml makes sense here.".to_string();
                        return Err(Box::new(ToolError::Exit(msg)));
                    }
                };
                match self.templates.render(&template, &self) {
                    Ok(x) => x,
                    Err(e) => {
                        let msg = format!("Can't render template {}, {:?}", template, e);
                        return Err(Box::new(ToolError::Exit(msg)));
                    }
                }
            }
        };

        // NOTE: ampersand '&' must be escaped in XML content.

        let re_def_whitespace = Regex::new(r#"(<definition type="[a-z]+">)[ \n]+(.*?)[ \n]*(</definition>)"#).unwrap();
        content = re_def_whitespace.replace_all(&content, "$1$2$3").to_string();

        content = clean_output_content(&content);

        let mut file = File::create(&self.output_path)?;
        file.write_all(content.as_bytes())?;

        Ok(())
    }

    pub fn create_stardict(&mut self) -> Result<(), Box<dyn Error>> {
        self.write_stardict_xml()?;
        Ok(())
    }

    pub fn write_latex(&self) -> Result<(), Box<dyn Error>> {
        info!("write_latex()");

        let mut content = match self.entries_template {
            Some(ref path) => {
                let template_source = match fs::read_to_string(path) {
                    Ok(x) => x,
                    Err(e) => {
                        let msg = format!("Can't read file: {:?}, {:?}", path, e);
                        return Err(Box::new(ToolError::Exit(msg)));
                    }
                };

                let mut h = Handlebars::new();
                h.set_strict_mode(true);
                h.register_escape_fn(helpers::light_html_escape);
                match h.render_template(&template_source, &self) {
                    Ok(x) => x,
                    Err(e) => {
                        let msg = format!("Can't render template: {:?}", e);
                        return Err(Box::new(ToolError::Exit(msg)));
                    }
                }
            }

            None => {
                let template = "latex_plain.tex".to_string();
                match self.templates.render(&template, &self) {
                    Ok(x) => x,
                    Err(e) => {
                        let msg = format!("Can't render template {}, {:?}", template, e);
                        return Err(Box::new(ToolError::Exit(msg)));
                    }
                }
            }
        };

        // NOTE: ampersand '&' must be escaped in LaTeX.
        content = content.replace("&", "\\&");

        let re_def_whitespace = Regex::new(r#"(<definition type="[a-z]+">)[ \n]+(.*?)[ \n]*(</definition>)"#).unwrap();
        content = re_def_whitespace.replace_all(&content, "$1$2$3").to_string();

        content = clean_output_content(&content);

        let mut file = File::create(&self.output_path)?;
        file.write_all(content.as_bytes())?;

        Ok(())
    }

    pub fn create_latex(&mut self) -> Result<(), Box<dyn Error>> {
        self.write_latex()?;
        Ok(())
    }

    pub fn write_c5(&self) -> Result<(), Box<dyn Error>> {
        info!("write_c5()");

        let mut content = match self.entries_template {
            Some(ref path) => {
                let template_source = match fs::read_to_string(path) {
                    Ok(x) => x,
                    Err(e) => {
                        let msg = format!("Can't read file: {:?}, {:?}", path, e);
                        return Err(Box::new(ToolError::Exit(msg)));
                    }
                };

                let mut h = Handlebars::new();
                h.set_strict_mode(true);
                h.register_escape_fn(helpers::light_html_escape);
                match h.render_template(&template_source, &self) {
                    Ok(x) => x,
                    Err(e) => {
                        let msg = format!("Can't render template: {:?}", e);
                        return Err(Box::new(ToolError::Exit(msg)));
                    }
                }
            }

            None => {
                let template = match self.output_format {
                    OutputFormat::C5Plain => "c5_plain.txt".to_string(),
                    OutputFormat::C5Html => "c5_html.txt".to_string(),
                    _ => {
                        let msg = "🔥 Only C5Plain or C5Html makes sense here.".to_string();
                        return Err(Box::new(ToolError::Exit(msg)));
                    }
                };

                match self.templates.render(&template, &self) {
                    Ok(x) => x,
                    Err(e) => {
                        let msg = format!("Can't render template {}, {:?}", template, e);
                        return Err(Box::new(ToolError::Exit(msg)));
                    }
                }
            }
        };

        if let OutputFormat::C5Plain = self.output_format {
            content = content.replace("&amp;", "&");
        }

        content = clean_output_content(&content);

        let mut file = File::create(&self.output_path)?;
        file.write_all(content.as_bytes())?;

        Ok(())
    }

    pub fn create_c5(&mut self) -> Result<(), Box<dyn Error>> {
        self.write_c5()?;
        Ok(())
    }

    pub fn write_tei(&self) -> Result<(), Box<dyn Error>> {
        info!("write_tei()");

        let template = match self.output_format {
            OutputFormat::TeiPlain => "freedict-tei_plain.xml".to_string(),
            OutputFormat::TeiFormatted => "freedict-tei_formatted.xml".to_string(),
            _ => {
                let msg = "🔥 Only TeiPlain or TeiFormatted makes sense here.".to_string();
                return Err(Box::new(ToolError::Exit(msg)));
            }
        };

        let mut content = match self.templates.render(&template, &self) {
            Ok(x) => x,
            Err(e) => {
                error!("Can't render template {}, {:?}", template, e);
                "FIXME: Template rendering error.".to_string()
            }
        };

        if let OutputFormat::TeiPlain = self.output_format {
            content = content.replace("&amp;", "&");
        }

        // Remove double blanks from the output, empty attributes leave empty spaces when rendering
        // the template.
        let re_double_blanks = Regex::new(r"\n\n+").unwrap();
        content = re_double_blanks.replace_all(&content, "\n\n").to_string();

        let mut file = File::create(&self.output_path)?;
        file.write_all(content.as_bytes())?;

        Ok(())
    }

    pub fn create_tei(&mut self) -> Result<(), Box<dyn Error>> {
        self.write_tei()?;
        Ok(())
    }

    pub fn create_json(&mut self) -> Result<(), Box<dyn Error>> {
        info!("create_json()");

        // Write Entries as DictWordXlsx.

        {
            let entries_xlsx = &self.dict_words_input
                .values()
                .cloned()
                .map(|i| DictWordXlsx::from_dict_word_markdown(&i))
                .collect::<Vec<DictWordXlsx>>();
            let content = serde_json::to_string(&entries_xlsx)?;

            let mut file = File::create(&self.output_path)?;
            file.write_all(content.as_bytes())?;
        }

        // Write Metadata.

        {
            let content = serde_json::to_string(&self.meta)?;

            let a = PathBuf::from(self.output_path.file_name().unwrap());
            let mut b = a.file_stem().unwrap().to_str().unwrap().to_string();
            b.push_str("-metadata.json");
            let path = self.output_path.with_file_name(b);

            let mut file = File::create(&path)?;
            file.write_all(content.as_bytes())?;
        }

        Ok(())
    }

    pub fn insert_to_sqlite(&mut self, app_params: &AppStartParams) -> Result<(), Box<dyn Error>> {
        info!("insert_to_sqlite()");

        let conn = if let Some(p) = &app_params.output_path {

            SqliteConnection::establish(p.to_str().unwrap()).expect("Error connecting to database.")

        } else {
            let msg = "🔥 Database path is missing.".to_string();
            return Err(Box::new(ToolError::Exit(msg)));
        };

        if self.dict_words_render.is_empty() {
            warn!{"🔥 There are not words to insert."};
            return Ok(());
        }

        // NOTE This is assuming that we are processing a single (markdown or json) file, which
        // represents a single dictionary.

        let (_, w) = self.dict_words_render.first_key_value().unwrap();
        let db_dictionary = Dictionary::get_or_insert_dictionary(&conn, &w.dict_label, &self.meta.title);

        for (_, w) in self.dict_words_render.iter() {

            let new_word = NewDictWord {
                dictionary_id:    &db_dictionary.id,
                word:             &w.word,
                word_nom_sg:      &w.word_nom_sg,
                inflections:      &w.inflections.join(", "),
                phonetic:         &w.phonetic,
                transliteration:  &w.transliteration,
                url_id:           &w.url_id,
            };

            let db_word = match Dictionary::insert_dict_word_if_doesnt_exist(&conn, &new_word) {
                Ok(x) => x,
                Err(_) => {
                    warn!{"Word exists, skipping: {:?}", &w.word};
                    continue;
                },
            };

            for m in w.meanings.iter() {
                let new_meaning = NewMeaning {
                    dict_word_id: &db_word.id,
                    meaning_order: &(m.meaning_order as i32),
                    definition_md: &m.definition_md,
                    summary: &m.summary,
                    synonyms: &m.synonyms.join(", "),
                    antonyms: &m.antonyms.join(", "),
                    homonyms: &m.homonyms.join(", "),
                    also_written_as: &m.also_written_as.join(", "),
                    see_also: &m.see_also.join(", "),
                    comment: &m.comment,
                    is_root: &m.is_root,
                    root_language: &m.root_language,
                    root_groups: &m.root_groups.join(", "),
                    root_sign: &m.root_sign,
                    root_numbered_group: &m.root_numbered_group,
                };

                let db_meaning = Dictionary::insert_new_meaning(&conn, &new_meaning);

                let new_grammar = NewGrammar {
                    meaning_id: &db_meaning.id,
                    roots: &m.grammar.roots.join(", "),
                    prefix_and_root: &m.grammar.prefix_and_root,
                    construction: &m.grammar.construction,
                    base_construction: &m.grammar.base_construction,
                    compound_type: &m.grammar.compound_type,
                    compound_construction: &m.grammar.compound_construction,
                    comment: &m.grammar.comment,
                    speech: &m.grammar.speech,
                    case: &m.grammar.case,
                    num: &m.grammar.num,
                    gender: &m.grammar.gender,
                    person: &m.grammar.person,
                    voice: &m.grammar.voice,
                    object: &m.grammar.object,
                    transitive: &m.grammar.transitive,
                    negative: &m.grammar.negative,
                    verb: &m.grammar.verb,
                };

                let _db_grammar = Dictionary::insert_new_grammar(&conn, &new_grammar);

                for ex in m.examples.iter() {
                    let new_example = NewExample {
                        meaning_id: &db_meaning.id,
                        source_ref: &ex.source_ref,
                        source_title: &ex.source_title,
                        text_md: &ex.text_md,
                        translation_md: &ex.translation_md,
                    };

                    let _db_example = Dictionary::insert_new_example(&conn, &new_example);
                }
            }
        }

        Ok(())
    }

    fn get_or_insert_dictionary(
        conn: &SqliteConnection,
        d_label: &str,
        d_title: &str)
        -> DbDictionary
    {
        use db_schema::dictionaries::dsl::*;
        use db_schema::dictionaries;

        let mut items = dictionaries
            .filter(label.eq(d_label))
            .load::<DbDictionary>(conn)
            .expect("Error loading the dictionary.");

        if !items.is_empty() {
            return items.pop().unwrap();
        }

        let new_dictionary = NewDictionary {
            label: d_label,
            title: d_title,
        };

        diesel::insert_into(dictionaries::table)
            .values(new_dictionary)
            .execute(conn)
            .expect("Error inserting the dictionary.");

        items = dictionaries
            .filter(label.eq(d_label))
            .load::<DbDictionary>(conn)
            .expect("Error loading the dictionary.");

        items.pop().unwrap()
    }

    fn insert_dict_word_if_doesnt_exist<'a>(
        conn: &SqliteConnection,
        new_word: &'a NewDictWord)
        -> Result<DbDictWord, DbDictWord>
    {
        use db_schema::dict_words::dsl::*;
        use db_schema::dict_words;

        let a = dict_words
            .filter(url_id.eq(new_word.url_id))
            .load::<DbDictWord>(conn)
            .expect("Error loading the word.");

        if !a.is_empty() {
            return Err(a[0].clone());
        }

        diesel::insert_into(dict_words::table)
            .values(new_word)
            .execute(conn)
            .expect("Error inserting the word.");

        let mut items = dict_words
            .filter(url_id.eq(new_word.url_id))
            .load::<DbDictWord>(conn)
            .expect("Error loading the inserted word.");

        Ok(items.pop().unwrap())
    }

    fn insert_new_meaning<'a>(
        conn: &SqliteConnection,
        new_meaning: &'a NewMeaning)
        -> DbMeaning
    {
        use db_schema::meanings::dsl::*;
        use db_schema::meanings;

        diesel::insert_into(meanings::table)
            .values(new_meaning)
            .execute(conn)
            .expect("Error inserting the meaning.");

        meanings.order(id.desc()).first(conn)
            .expect("Error loading the inserted meaning.")
    }

    fn insert_new_grammar<'a>(
        conn: &SqliteConnection,
        new_grammar: &'a NewGrammar)
        -> DbGrammar
    {
        use db_schema::grammars::dsl::*;
        use db_schema::grammars;

        diesel::insert_into(grammars::table)
            .values(new_grammar)
            .execute(conn)
            .expect("Error inserting the grammar.");

        grammars.order(id.desc()).first(conn)
            .expect("Error loading the inserted grammar.")
    }

    fn insert_new_example<'a>(
        conn: &SqliteConnection,
        new_example: &'a NewExample)
        -> DbExample
    {
        use db_schema::examples::dsl::*;
        use db_schema::examples;

        diesel::insert_into(examples::table)
            .values(new_example)
            .execute(conn)
            .expect("Error inserting the example.");

        examples.order(id.desc()).first(conn)
            .expect("Error loading the inserted example.")
    }

    pub fn create_render_json(&mut self) -> Result<(), Box<dyn Error>> {
        info!("create_render_json()");

        {
            let entries = &self.dict_words_render
                .values()
                .cloned()
                .collect::<Vec<DictWord>>();
            let content = serde_json::to_string(&entries)?;

            let mut file = File::create(&self.output_path)?;
            file.write_all(content.as_bytes())?;
        }

        // Write Metadata.

        {
            let content = serde_json::to_string(&self.meta)?;

            let a = PathBuf::from(self.output_path.file_name().unwrap());
            let mut b = a.file_stem().unwrap().to_str().unwrap().to_string();
            b.push_str("-metadata.json");
            let path = self.output_path.with_file_name(b);

            let mut file = File::create(&path)?;
            file.write_all(content.as_bytes())?;
        }

        Ok(())
    }

    pub fn create_xlsx(&mut self) -> Result<(), Box<dyn Error>> {
        info!("create_xlsx()");

        let workbook: Workbook = Workbook::new(&self.output_path.to_str().unwrap());
        let mut words_sheet: Worksheet = workbook.add_worksheet(Some("Words"))?;
        let mut roots_sheet: Worksheet = workbook.add_worksheet(Some("Roots"))?;
        let mut metadata_sheet: Worksheet = workbook.add_worksheet(Some("Metadata"))?;

        let header_format: Format = workbook.add_format()
            .set_bg_color(FormatColor::Custom(0xDBDBDB))
            .set_font_color(FormatColor::Black);

        // Convert the Markdown to DictWordXlsx, serialize it to JSON, and create the values in the
        // worksheets.

        {
            let entries_xlsx = &self.dict_words_input
                .values()
                .cloned()
                .map(|i| DictWordXlsx::from_dict_word_markdown(&i))
                .collect::<Vec<DictWordXlsx>>();

            let entries_json: Vec<Value> = serde_json::to_value(&entries_xlsx)?
                .as_array().unwrap().to_vec();

            let word_entries: Vec<Map<String, Value>> = entries_json.iter()
                .filter_map(|i| {
                    let e: &Map<String, Value> = i.as_object().unwrap();
                    if e.get("is_root").unwrap().as_bool().unwrap() {
                        None
                    } else {
                        Some(e.clone())
                    }
                })
            .collect();

            let root_entries: Vec<Map<String, Value>> = entries_json.iter()
                .filter_map(|i| {
                    let e: &Map<String, Value> = i.as_object().unwrap();
                    if e.get("is_root").unwrap().as_bool().unwrap() {
                        Some(e.clone())
                    } else {
                        None
                    }
                })
            .collect();

            let meta_json = serde_json::to_value(&self.meta).unwrap();
            let metadata_entries: Vec<Map<String, Value>> = vec![
                meta_json.as_object().unwrap().clone()
            ];

            // By default, keys are ordered alphabetically.
            //
            // An array has to specify a meaningful order of keys which helps when authoring
            // the entries.

            let words_sheet_columns: Vec<String> = vec![
                "word".to_string(),
                "meaning_order".to_string(),
                "word_nom_sg".to_string(),
                "dict_label".to_string(),
                "inflections".to_string(),
                "phonetic".to_string(),
                "transliteration".to_string(),
                "example_count".to_string(),
                "definition_md".to_string(),
                "summary".to_string(),
                "synonyms".to_string(),
                "antonyms".to_string(),
                "homonyms".to_string(),
                "also_written_as".to_string(),
                "see_also".to_string(),
                "comment".to_string(),
                "gr_roots".to_string(),
                "gr_prefix_and_root".to_string(),
                "gr_construction".to_string(),
                "gr_base_construction".to_string(),
                "gr_compound_type".to_string(),
                "gr_compound_construction".to_string(),
                "gr_comment".to_string(),
                "gr_speech".to_string(),
                "gr_case".to_string(),
                "gr_num".to_string(),
                "gr_gender".to_string(),
                "gr_person".to_string(),
                "gr_voice".to_string(),
                "gr_object".to_string(),
                "gr_transitive".to_string(),
                "gr_negative".to_string(),
                "gr_verb".to_string(),
                "ex_1_source_ref".to_string(),
                "ex_1_source_title".to_string(),
                "ex_1_text_md".to_string(),
                "ex_1_translation_md".to_string(),
                "ex_2_source_ref".to_string(),
                "ex_2_source_title".to_string(),
                "ex_2_text_md".to_string(),
                "ex_2_translation_md".to_string(),
            ];

            let roots_sheet_columns: Vec<String> = vec![
                "word".to_string(),
                "meaning_order".to_string(),
                "root_language".to_string(),
                "root_groups".to_string(),
                "root_sign".to_string(),
                "root_numbered_group".to_string(),
                "definition_md".to_string(),
            ];

            let metadata_sheet_columns: Vec<String> = vec![
                "title".to_string(),
                "description".to_string(),
                "creator".to_string(),
                "source".to_string(),
                "cover_path".to_string(),
                "book_id".to_string(),
                "add_velthuis".to_string(),
            ];

            // The Words sheet should include all fields, except 5:
            // - is_root
            // - root_language
            // - root_groups
            // - root_sign
            // - root_numbered_group
            {
                let e: &Map<String, Value> = entries_json[0].as_object().unwrap();
                if words_sheet_columns.len() != e.keys().len() - 5 {
                    let msg = "🔥 Column numbers don't match.".to_string();
                    return Err(Box::new(ToolError::Exit(msg)));
                }
            }

            Dictionary::fill_sheet(&mut words_sheet, &word_entries, &words_sheet_columns, &header_format)?;

            Dictionary::fill_sheet(&mut roots_sheet, &root_entries, &roots_sheet_columns, &header_format)?;

            Dictionary::fill_sheet(&mut metadata_sheet, &metadata_entries, &metadata_sheet_columns, &header_format)?;

        }

        workbook.close()?;

        Ok(())
    }

    fn fill_sheet(
        sheet: &mut Worksheet,
        entries: &[Map<String, Value>],
        column_names: &[String],
        header_format: &Format)
        -> Result<(), Box<dyn Error>>
    {
        for (entry_idx, e) in entries.iter().enumerate() {

            // Row 0: column names
            for (col_idx, col_name) in column_names.iter().enumerate() {
                sheet.write_string(0, col_idx.try_into().unwrap(), col_name, Some(header_format))?;
            }

            // Row 1, 2, ...: values
            let row = entry_idx + 1;
            for (col_idx, col_name) in column_names.iter().enumerate() {
                let row_u32: u32 = row.try_into().unwrap();
                let col_idx_u16: u16 = col_idx.try_into().unwrap();

                match e.get(col_name).unwrap() {
                    Value::Null => {},
                    Value::Bool(x) => sheet.write_boolean(row_u32, col_idx_u16, *x, None)?,
                    Value::Number(x) => sheet.write_number(row_u32, col_idx_u16, x.as_f64().unwrap(), None)?,
                    Value::String(x) => sheet.write_string(row_u32, col_idx_u16, x, None)?,
                    Value::Array(_) | Value::Object(_) => {
                        error!("🔥 Can't write Array or Object to XLSX.");
                        sheet.write_string(row_u32, col_idx_u16, "FIXME", None)?;
                    },
                }
            }
        }

        Ok(())
    }

    fn set_paths_to_none(&mut self) {
        // TODO these should be grouped in one type, all depending on base dir being created.
        self.build_base_dir = None;
        self.mimetype_path = None;
        self.meta_inf_dir = None;
        self.oebps_dir = None;
    }

    pub fn process_text(&mut self) {
        self.process_strip_html_for_plaintext();
        self.process_add_transliterations();
        self.process_links();
        self.process_define_links();
        self.process_input_to_render();
    }

    pub fn process_tidy(&mut self) {
        info!("process_tidy()");

        // (see *[*kaṭhina*](/define/<i>kaṭhina</i>)*)
        let re_italic_html = Regex::new(r"<i>([^<]+)</i>").unwrap();
        // (see *[*kaṭhina*](/define/<strong>kaṭhina</strong>)*)
        let re_strong_html = Regex::new(r"<strong>([^<]+)</strong>").unwrap();
        // (see also *[kathīka (?)](/define/kathīka (?))*)
        let re_define_question = Regex::new(r"\[[^\]]+\]\(/define/([^\(\) ]+) *\(\?\)\)").unwrap();
        // (see also *[kubbati, ](/define/kubbati, )* and *[kurute](/define/kurute)*)
        let re_define_comma = Regex::new(r"\[[^\]]+\]\(/define/([^\(\), ]+), *\)").unwrap();

        // Strip remaining internal links, i.e. which are not /define or outgoing http links.
        let re_strip_internal = Regex::new(r"\[(?P<label>[^\]]+)\]\((?P<define>/define/|http)?[^\)]+\)").unwrap();

        for (_, dict_word) in self.dict_words_input.iter_mut() {
            let mut s = dict_word.definition_md.clone();

            // strip empty links
            s = s.replace("[]()", "");

            s = re_italic_html.replace_all(&s, "$1").to_string();
            s = re_strong_html.replace_all(&s, "$1").to_string();
            s = re_define_question.replace_all(&s, "[$1](/define/$1)").to_string();
            s = re_define_comma.replace_all(&s, "[$1](/define/$1),").to_string();

            for cap in re_strip_internal.captures_iter(&s.clone()) {
                let link = cap[0].to_string();
                if cap.name("define").is_none() {
                    s = s.replace(&link, &cap["label"]);
                }
            }

            dict_word.definition_md = s;
        }

    }

    pub fn process_also_written_as(&mut self) {
        info!("process_also_written_as()");
        // anūpa(n)āhi(n)
        let re_first_word_parens_mid_end = Regex::new(r"^ *([^ \n]+)\((.)\)([^ \n]+)\((.)\)").unwrap();
        // anūpa(n)āhin
        let re_first_word_parens_mid = Regex::new(r"^ *([^ \n]+)\((.)\)([^ \n]+)").unwrap();
        // anūpanāhi(n)
        let re_first_word_parens_end = Regex::new(r"^ *([^ \n]+)\((.)\)").unwrap();
        // (also written as *aruṇugga*)
        let re_also_written_ital = Regex::new(r"\(also written as \*([^ ]+)\*\)").unwrap();
        let re_also_written_plain = Regex::new(r"\(also written as *([^ ]+)\)").unwrap();

        for (_, dict_word) in self.dict_words_input.iter_mut() {
            //let word = dict_word.word_header.word.clone();
            let mut s = dict_word.definition_md.trim().to_string();

            // First word with parens indicating alternative form.

            if let Some(caps) = re_first_word_parens_mid_end.captures(&s) {
                let w = format!("{}{}{}{}",
                    caps.get(1).unwrap().as_str(),
                    caps.get(2).unwrap().as_str(),
                    caps.get(3).unwrap().as_str(),
                    caps.get(4).unwrap().as_str());
                dict_word.word_header.also_written_as.push(w);
                s = re_first_word_parens_mid_end.replace(&s, "").to_string().trim().to_string();
            }

            if let Some(caps) = re_first_word_parens_mid.captures(&s) {
                let w = format!("{}{}{}",
                    caps.get(1).unwrap().as_str(),
                    caps.get(2).unwrap().as_str(),
                    caps.get(3).unwrap().as_str());
                dict_word.word_header.also_written_as.push(w);
                s = re_first_word_parens_mid.replace(&s, "").to_string().trim().to_string();
            }

            if let Some(caps) = re_first_word_parens_end.captures(&s) {
                let w = format!("{}{}",
                    caps.get(1).unwrap().as_str(),
                    caps.get(2).unwrap().as_str());
                dict_word.word_header.also_written_as.push(w);
                s = re_first_word_parens_end.replace(&s, "").to_string().trim().to_string();
            }

            // (also written as...)

            for cap in re_also_written_ital.captures_iter(&s) {
                dict_word.word_header.also_written_as.push(cap[1].to_string());
            }
            s = re_also_written_ital.replace_all(&s, "").to_string();

            for cap in re_also_written_plain.captures_iter(&s) {
                dict_word.word_header.also_written_as.push(cap[1].to_string());
            }
            s = re_also_written_plain.replace_all(&s, "").to_string();

            dict_word.definition_md = s;
        }

        // TODO: variations in links and see also
        // (also *anūpanāhi(n)*)
        // (see *[upanāhi(n)](/define/upanāhi(n))*)
        // (see *[upaparikkha(t)](/define/upaparikkha(t))*)
        // [abhu(ṃ)](/define/abhu(ṃ))
        //
        // If an alternative form is mentioned in the definition_md:
        // - find which form has an entry
        // - if both forms have an entry, warn the user
        // - merge the longer form into the shorter
        // - add new form to inflections and also_written_as

    }

    pub fn process_strip_repeat_word_title(&mut self) {
        info!("process_strip_repeat_word_title()");

        for (_, dict_word) in self.dict_words_input.iter_mut() {
            let word = dict_word.word_header.word.clone();
            let mut s = dict_word.definition_md.trim().to_string();

            // The simplest case, the whole word
            // - abhijanat, abhikamin
            // Don't match parens variations abhikami(n), which would leave only (n)
            // abhijanat
            s = s.trim_start_matches(&format!("{}\n", word)).trim().to_string();
            // Abhijanat
            s = s.trim_start_matches(&format!("{}\n", uppercase_first_letter(&word))).trim().to_string();

            dict_word.definition_md = s;
        }
    }

    pub fn process_grammar_note(&mut self) {
        info!("process_grammar_note()");

        // grammar abbr., with- or without dot, with- or without parens
        let re_abbr_one = Regex::new(r"^[0-9 ]*\(*(d|f|m|ṃ|n|r|s|t)\.*\)*\.*\b").unwrap();
        let re_abbr_two = Regex::new(r"^[0-9 ]*\(*(ac|fn|id|mf|pl|pp|pr|sg|si)\.*\)*\.*\b").unwrap();
        let re_abbr_three = Regex::new(
            r"^[0-9 ]*\(*(abl|acc|act|adv|aor|dat|fpp|fut|gen|inc|ind|inf|loc|mfn|neg|opt|par)\.*\)*\.*\b",
        )
            .unwrap();
        let re_abbr_four = Regex::new(r"^[0-9 ]*\(*(caus|part|pass|pron)\.*\)*\.*\b").unwrap();
        let re_abbr_more = Regex::new(r"^[0-9 ]*\(*(absol|abstr|accus|compar|desid|feminine|impers|instr|masculine|metaph|neuter|plural|singular|trans)\.*\)*\.*\b").unwrap();

        // ~ā
        let re_suffix_a = Regex::new(r"^\\*~*[aā],* +").unwrap();

        for (_key, dict_word) in self.dict_words_input.iter_mut() {
            let mut def = dict_word.definition_md.trim().to_string();

            let max_iter = 10;
            let mut n_iter = 0;

            loop {
                let mut s = def.clone();

                // (?)
                s = s.trim_start_matches("(?)").trim().to_string();
                s = s.trim_start_matches("?)").trim().to_string();

                // pp space
                s = s.trim_start_matches("pp ").trim().to_string();
                // abbr, start with longer patterns
                s = re_abbr_more.replace(&s, "").trim().to_string();
                s = re_abbr_four.replace(&s, "").trim().to_string();
                s = re_abbr_three.replace(&s, "").trim().to_string();
                s = re_abbr_two.replace(&s, "").trim().to_string();
                s = re_abbr_one.replace(&s, "").trim().to_string();

                // FIXME somehow the above sometimes leaves the closing paren and dot
                s = s.trim_start_matches(')').trim().to_string();
                s = s.trim_start_matches('.').trim().to_string();
                s = s.trim_start_matches(';').trim().to_string();

                // ~ā
                s = re_suffix_a.replace(&s, "").to_string();
                // (& m.)
                s = s.trim_start_matches(r"(& m.)").trim().to_string();
                s = s.trim_start_matches(r"(& f.)").trim().to_string();
                s = s.trim_start_matches(r"(& n.)").trim().to_string();

                // m(fn).
                s = s.trim_start_matches("(& mfn.)").trim().to_string();
                s = s.trim_start_matches("m(fn)").trim().to_string();
                s = s.trim_start_matches('.').trim().to_string();

                // m.a
                s = s.trim_start_matches("m.a").trim().to_string();
                // &
                s = s.trim_start_matches('&').trim().to_string();
                // fpp[.]
                s = s.trim_start_matches("fpp[.]").trim().to_string();

                n_iter += 1;

                if s == def {
                    // stop if there was no change
                    break;
                } else if n_iter == max_iter {
                    // or we hit max_iter
                    info!("max_iter reached: {}", s);
                    def = s;
                    break;
                } else {
                    // apply changes and loop again
                    def = s;
                }
            }

            dict_word.word_header.grammar_comment = dict_word.definition_md
                .trim_end_matches(&def)
                .trim_end_matches(',')
                .trim()
                .to_string();
            dict_word.definition_md = def;
        }
    }

    pub fn process_see_also_from_definition(&mut self, dont_remove_see_also: bool) {
        info!("process_see_also_from_definition()");

        // [ab(b)ha(t)](/define/ab(b)ha(t))
        let re_define_parens_mid_end = Regex::new(r"\[([^\]]+)\]\(/define/([^\(\)]+)\((.)\)([^\(\)]+)\((.)\)\)").unwrap();
        // [ab(b)hat](/define/ab(b)hat)
        let re_define_parens_mid = Regex::new(r"\[([^\]]+)\]\(/define/([^\(\)]+)\((.)\)([^\(\)]+)\)").unwrap();
        // [abhu(ṃ)](/define/abhu(ṃ))
        let re_define_parens_end = Regex::new(r"\[([^\]]+)\]\(/define/([^\(\)]+)\((.)\)\)").unwrap();
        // [abhuṃ](/define/abhuṃ)
        let re_define = Regex::new(r"\[([^\]]+)\]\(/define/([^\(\)]+)\)").unwrap();
        // We're going to temporarily replace links as [[abbha]]
        let re_bracket_links = Regex::new(r"\[\[([^]]+)\]\]").unwrap();
        // (see also *[abbuhati](/define/abbuhati)* and *[abbūhati](/define/abbūhati)*)
        // (see *[abbha](/define/abbha)*)
        let re_see_also = Regex::new(r" *\(see ([^\)]+)\)").unwrap();

        // words with and without italics (stars) have to be covered
        // word must be min. 3 chars long
        // The (n) (t) etc. variations are parsed in process_also_written_as()
        // (also *abhisāpeti*)
        // (also *anūpanāhi(n)*)
        // (see *[upanāhi(n)](/define/upanāhi(n))*)
        // (see *[upaparikkha(t)](/define/upaparikkha(t))*)

        let re_also_one_plain = Regex::new(r"\(also +([^\*\(\),]{3,})\)").unwrap();
        let re_also_one_italics = Regex::new(r"\(also +\*([^\*\(\),]{3,})\*\)").unwrap();
        // (also *abhisaṅkhaṭa* and *abhisaṅkhita*)
        let re_also_two_plain = Regex::new(r"\(also +([^\*\(\), ]{3,})(, +| +and +|, +and +| +& +)([^\*\(\)]{3,})\)").unwrap();
        let re_also_two_italics = Regex::new(r"\(also +\*([^\*\(\), ]{3,})\*(, +| +and +|, +and +| +& +)\*([^\*\(\)]{3,})\*\)").unwrap();
        // (also *apabyūhati*, *apaviyūhati*, and *apabbūhati*)
        let re_also_three_plain = Regex::new(r"\(also +([^\*\(\), ]{3,}), +([^\*\(\), ]{3,})(, +| +and +|, +and +| +& +)([^\*\(\)]{3,})\)").unwrap();
        let re_also_three_italics = Regex::new(r"\(also +\*([^\*\(\), ]{3,})\*, +\*([^\*\(\), ]{3,})\*(, +| +and +|, +and +| +& +)\*([^\*\(\)]{3,})\*\)").unwrap();

        for (_, w) in self.dict_words_input.iter_mut() {
            let mut def: String = w.definition_md.clone();

            // (also *abhisāpeti*) -> (see [abhisāpeti](/define/abhisāpeti))
            def = re_also_three_italics.replace_all(&def, "(see [$1](/define/$1), [$2](/define/$2) and [$4](/define/$4))").to_string();
            def = re_also_three_plain.replace_all(&def, "(see [$1](/define/$1), [$2](/define/$2) and [$4](/define/$4))").to_string();

            def = re_also_two_italics.replace_all(&def, "(see [$1](/define/$1) and [$3](/define/$3))").to_string();
            def = re_also_two_plain.replace_all(&def, "(see [$1](/define/$1) and [$3](/define/$3))").to_string();

            def = re_also_one_italics.replace_all(&def, "(see [$1](/define/$1))").to_string();
            def = re_also_one_plain.replace_all(&def, "(see [$1](/define/$1))").to_string();

            // Collect /define links from the text and add to see_also list.

            for link in re_define_parens_mid_end.captures_iter(&def) {
                let word = format!("{}{}{}{}", link[2].to_string(), link[3].to_string(), link[4].to_string(), link[5].to_string());
                w.word_header.see_also.push(word);
            }

            for link in re_define_parens_mid.captures_iter(&def) {
                let word = format!("{}{}{}", link[2].to_string(), link[3].to_string(), link[4].to_string());
                w.word_header.see_also.push(word);
            }

            for link in re_define_parens_end.captures_iter(&def) {
                let word = format!("{}{}", link[2].to_string(), link[3].to_string());
                w.word_header.see_also.push(word);
            }

            for link in re_define.captures_iter(&def) {
                w.word_header.see_also.push(link[2].to_string());
            }

            // [wordlabel](/define/wordlink) -> [[wordlink]]
            def = re_define_parens_mid_end.replace_all(&def, "[[$2$3$4$5]]").to_string();
            def = re_define_parens_mid.replace_all(&def, "[[$2$3$4]]").to_string();
            def = re_define_parens_end.replace_all(&def, "[[$2$3]]").to_string();
            def = re_define.replace_all(&def, "[[$2]]").to_string();
            // Remove 'See also' from the text.
            if !dont_remove_see_also {
                def = re_see_also.replace_all(&def, "").to_string();
            }
            // [[wordlink]] -> [wordlink](/define/wordlink)
            def = re_bracket_links.replace_all(&def, "[$1](/define/$1)").to_string();

            w.definition_md = def;
        }
    }

    pub fn process_strip_html_for_plaintext(&mut self) {
        info!("process_strip_html_for_plaintext()");

        // Have to match specific tags, sometimes text is wrapped in <...> in the definition as
        // an editorial practice.
        let re_html = Regex::new(r"</*(sup|em|strong|a|i|b) *>").unwrap();

        match self.output_format {
            OutputFormat::StardictXmlPlain | OutputFormat::C5Plain | OutputFormat::TeiPlain => {
                for w in self.dict_words_input.values_mut() {
                    w.definition_md = re_html.replace_all(&w.definition_md, "").to_string();
                }
            },
            _ => {}
        }
    }

    pub fn process_define_links(&mut self) {
        info!("process_define_links()");
        // [abhuṃ](/define/abhuṃ)
        let re_define = Regex::new(r"\[[^0-9\]\(\)]+\]\(/define/(?P<define>[^\(\)]+)\)").unwrap();

        let w: Vec<DictWordMarkdown> = self.dict_words_input.values().cloned().collect();
        let letter_groups = LetterGroups::new_from_dict_words(&w);
        let words_to_url = letter_groups.words_to_url;

        for (_, dict_word) in self.dict_words_input.iter_mut() {
            let def = dict_word.definition_md.clone();
            for cap in re_define.captures_iter(&def) {
                let link = cap[0].to_string();
                let word = cap["define"].to_string();

                let new_link = match self.output_format {
                    OutputFormat::Epub | OutputFormat::Mobi => {
                        match words_to_url.get(&word) {
                            Some(url) => format!("[{}]({})", word, url),
                            None => format!("*{}*", word),
                        }
                    },

                    OutputFormat::StardictXmlHtml | OutputFormat::BabylonGls => {
                        // If it is a valid word entry, replace to bword:// for Stardict and Babylon.
                        if self.valid_words.contains(&word) {
                            format!("[{}](bword://{})", word, word)
                        } else {
                            // If it is not a valid word entry, we will replace it with text.
                            format!("*{}*", word)
                        }
                    }

                    OutputFormat::C5Html | OutputFormat::TeiFormatted => {
                        if self.valid_words.contains(&word) {
                            format!("[{}]({})", word, word)
                        } else {
                            format!("*{}*", word)
                        }
                    }

                    OutputFormat::StardictXmlPlain | OutputFormat::C5Plain | OutputFormat::TeiPlain => {
                        if self.valid_words.contains(&word) {
                            // curly braces are escaped as {{ and }}
                            format!("{{{}}}", word)
                        } else {
                            format!("*{}*", word)
                        }
                    }

                    OutputFormat::LaTeXPlain => word,

                };

                dict_word.definition_md = dict_word.definition_md.replace(&link, &new_link).to_string();
            }
        }
    }

    pub fn process_input_to_render(&mut self) {
        info!("process_input_to_render()");

        // dict_words_input is sorted by key 'word-label-meaning_order'
        for dwi in self.dict_words_input.values() {
            let dwr: DictWord = DictWord::from_dict_word_markdown(dwi);

            // If the url_id already exist, append to the meanings.
            // Otherwise, insert as new.
            if let Some(word) = self.dict_words_render.get_mut(&dwr.url_id) {
                for m in dwr.meanings.iter() {
                    word.meanings.push(m.clone());
                }
            } else {
                self.dict_words_render.insert(dwr.url_id.clone(), dwr);
            };
        }

        // Renumber meaning_order values, they must start from 1 and be continuous.
        for dwr in self.dict_words_render.values_mut() {
            for (n, meaning) in dwr.meanings.iter_mut().enumerate() {
                meaning.meaning_order = n + 1;
            }
            dwr.meanings_count = dwr.meanings.len();
        }
    }

    pub fn process_summary(&mut self) -> Result<(), Box<dyn Error>> {
            let re_links = Regex::new(r"\[([^\]]*)\]\([^\)]*\)").unwrap();
            let re_spaces = Regex::new("  +").unwrap();
            let re_chars = Regex::new(r"[\n\t<>]").unwrap();
            let re_see_markdown_links = Regex::new(r"\(see \*\[([^\]]+)\]\([^\)]+\)\**\)").unwrap();
            let re_markdown_links = Regex::new(r"\[([^\]]+)\]\([^\)]+\)").unwrap();
            let re_markdown = Regex::new(r"[\*\[\]]").unwrap();

            // Don't remove (see ...), so that one can look up the next word when noticing it in the
            // search hits.

            // (from|or|also ...)
            let re_from = Regex::new(r"^\((from|or|also) +[^\)]+\)").unwrap();

            // 1
            // 1.
            let re_num = Regex::new(r"^[0-9]\.*").unwrap();

            // grammar abbr., with- or without dot, with- or without parens
            let re_abbr_one = Regex::new(r"^\(*(d|f|m|ṃ|n|r|s|t)\.*\)*\.*\b").unwrap();
            let re_abbr_two = Regex::new(r"^\(*(ac|fn|id|mf|pl|pp|pr|sg|si)\.*\)*\.*\b").unwrap();
            let re_abbr_three = Regex::new(
                r"^\(*(abl|acc|act|adv|aor|dat|fpp|fut|gen|inc|ind|inf|loc|mfn|neg|opt|par)\.*\)*\.*\b",
            )
                .unwrap();
            let re_abbr_four = Regex::new(r"^\(*(caus|part|pass|pron)\.*\)*\.*\b").unwrap();
            let re_abbr_more = Regex::new(r"^\(*(absol|abstr|accus|compar|desid|feminine|impers|instr|masculine|metaph|neuter|plural|singular|trans)\.*\)*\.*\b").unwrap();

            // ~ā
            let re_suffix_a = Regex::new(r"^\\*~*[aā],* +").unwrap();

            // (~ontī)
            // (-ikā)n.
            let re_suffix = Regex::new(r"^\([~-][^\)]+\)\w*\.*").unwrap();

            // agga-m-agga
            // abhi-uggantvā
            let re_hyphenated_twice = Regex::new(r"^\w+-\w+-\w+\b").unwrap();
            let re_hyphenated_once = Regex::new(r"^\w+-\w+\b").unwrap();

        for (_key, dict_word) in self.dict_words_input.iter_mut() {
            if !dict_word.word_header.summary.is_empty() {
                dict_word.word_header.summary = dict_word.word_header.summary.trim().to_string();
            }

            if !dict_word.word_header.summary.is_empty() {
                return Ok(());
            }

            let mut summary = dict_word.definition_md.trim().to_string();

            // strip links
            summary = re_links.replace_all(&summary, "$1").to_string();

            // newlines to space
            summary = summary.replace("\n", " ");
            // contract multiple spaces
            summary = re_spaces.replace_all(&summary, " ").trim().to_string();

            // remaining html tags
            summary = summary.replace("<sup>", "");
            summary = summary.replace("</sup>", "");
            summary = summary.replace("<i>", "");
            summary = summary.replace("</i>", "");
            summary = summary.replace("<b>", "");
            summary = summary.replace("</b>", "");

            summary = re_chars.replace_all(&summary, " ").trim().to_string();

            // slash escapes
            // un\-angered -> un-angered
            // un\\-angered -> un-angered
            summary = summary.replace(r"\\", "");
            summary = summary.replace(r"\", "");

            // See... with markdown link
            // (see *[abbha](/define/abbha)*) -> (see abbha)
            summary = re_see_markdown_links
                .replace_all(&summary, "(see $1)")
                .trim()
                .to_string();

            // markdown links
            // [abbha](/define/abbha) -> abbha
            summary = re_markdown_links
                .replace_all(&summary, "$1")
                .trim()
                .to_string();

            // remaining markdown markup: *, []
            summary = re_markdown.replace_all(&summary, "").trim().to_string();

            let word = dict_word.word_header.word.clone();

            let max_iter = 10;
            let mut n_iter = 0;

            loop {
                // the whole word
                // abhijanat, abhikamin
                let mut s = summary.trim_start_matches(&word).trim().to_string();

                // part of the word
                // abhijana(t)
                // abhikami(n)
                let (char_idx, _char) = word.char_indices().last().unwrap();
                let w = word[..char_idx].to_string();
                s = s.trim_start_matches(&w).trim().to_string();

                s = re_hyphenated_twice.replace(&s, "").trim().to_string();
                s = re_hyphenated_once.replace(&s, "").trim().to_string();

                s = re_num.replace(&s, "").trim().to_string();
                s = re_suffix.replace(&s, "").trim().to_string();

                s = re_from.replace(&s, "").trim().to_string();

                s = s.trim_start_matches('.').trim().to_string();
                s = s.trim_start_matches(',').trim().to_string();
                s = s.trim_start_matches('-').trim().to_string();

                // (?)
                s = s.trim_start_matches("(?)").trim().to_string();
                s = s.trim_start_matches("?)").trim().to_string();

                // pp space
                s = s.trim_start_matches("pp ").trim().to_string();
                // abbr, start with longer patterns
                s = re_abbr_more.replace(&s, "").trim().to_string();
                s = re_abbr_four.replace(&s, "").trim().to_string();
                s = re_abbr_three.replace(&s, "").trim().to_string();
                s = re_abbr_two.replace(&s, "").trim().to_string();
                s = re_abbr_one.replace(&s, "").trim().to_string();

                // FIXME somehow the above sometimes leaves the closing paren and dot
                s = s.trim_start_matches(')').trim().to_string();
                s = s.trim_start_matches('.').trim().to_string();
                s = s.trim_start_matches(';').trim().to_string();

                // ~ā
                s = re_suffix_a.replace(&s, "").to_string();
                // (& m.)
                s = s.trim_start_matches(r"(& m.)").trim().to_string();
                s = s.trim_start_matches(r"(& f.)").trim().to_string();
                s = s.trim_start_matches(r"(& n.)").trim().to_string();

                // m(fn).
                s = s.trim_start_matches("(& mfn.)").trim().to_string();
                s = s.trim_start_matches("m(fn)").trim().to_string();
                s = s.trim_start_matches('.').trim().to_string();

                // m.a
                s = s.trim_start_matches("m.a").trim().to_string();
                // &
                s = s.trim_start_matches('&').trim().to_string();
                // fpp[.]
                s = s.trim_start_matches("fpp[.]").trim().to_string();

                n_iter += 1;

                if s == summary {
                    // stop if there was no change
                    break;
                } else if n_iter == max_iter {
                    // or we hit max_iter
                    info!("max_iter reached: {}", s);
                    summary = s;
                    break;
                } else {
                    // apply changes and loop again
                    summary = s;
                }
            }

            // cap the length of the final summary

            if !summary.is_empty() {
                let sum_length = 50;
                if summary.char_indices().count() > sum_length {
                    let (char_idx, _char) = summary
                        .char_indices()
                        .nth(sum_length)
                        .ok_or("Bad char index")?;
                    summary = summary[..char_idx].trim().to_string();
                }

                // FIXME empty summary gets this too somehow
                // append ...
                //summary.push_str(" ...");
            }

            dict_word.word_header.summary = summary;

        }

        Ok(())
    }

    pub fn word_to_link(
        valid_words: &[String],
        words_to_url: &BTreeMap<String, String>,
        output_format: OutputFormat,
        w: &str)
        -> String
    {
        match output_format {
            OutputFormat::Epub | OutputFormat::Mobi => {
                match words_to_url.get(w) {
                    Some(url) => format!("<a href=\"{}\">{}</a>", url, w),
                    None => {
                        //info!("not found: {}", w);
                        w.to_string()
                    },
                }
            },

            OutputFormat::BabylonGls | OutputFormat::StardictXmlHtml => {
                if valid_words.contains(&w.to_string()) {
                    format!("<a href=\"bword://{}\">{}</a>", w, w)
                } else {
                    //info!("not found: {}", w);
                    w.to_string()
                }
            }

            OutputFormat::C5Html => {
                if valid_words.contains(&w.to_string()) {
                    format!("<a href=\"{}\">{}</a>", w, w)
                } else {
                    //info!("not found: {}", w);
                    w.to_string()
                }
            }

            OutputFormat::TeiFormatted => {
                match words_to_url.get(w) {
                    Some(url) => {
                        // entries-00.xhtml#abbhuṃ-ncped -> abbhuṃ-ncped
                        let id = url[17..].to_string();
                        format!("<ref target=\"{}\">{}</ref>", id, w)
                    },
                    None => {
                        w.to_string()
                    },
                }
            }

            OutputFormat::StardictXmlPlain | OutputFormat::C5Plain | OutputFormat::TeiPlain => {
                if valid_words.contains(&w.to_string()) {
                    // curly braces are escaped as {{ and }}
                    format!("{{{}}}", w)
                } else {
                    format!("*{}*", w)
                }
            }

            OutputFormat::LaTeXPlain => w.to_string(),
        }
    }

    pub fn all_words_to_links(
        valid_words: &[String],
        words_to_url: &BTreeMap<String, String>,
        output_format: OutputFormat,
        text: &str
        )
        -> String
    {
        lazy_static! {
            static ref RE_WORD_TO_LINK: Regex = Regex::new(r"([^ +>=√\(\)-]+)([ +>=√\(\)-]*)").unwrap();
        }

        let mut linked_text = String::new();

        for caps in RE_WORD_TO_LINK.captures_iter(&text) {
            let word = caps.get(1).unwrap().as_str().to_string();
            let sep = caps.get(2).unwrap().as_str().to_string();

            let w = word.trim_start_matches('√').to_string();
            let link = Dictionary::word_to_link(valid_words, &words_to_url, output_format, &w);

            linked_text.push_str(&word.replace(&w, &link));
            linked_text.push_str(&sep);
        }

        linked_text
    }

    /// Turn word lists into links for valid words.
    ///
    /// Run this before rendering, when no more words are added to `see_also` and other lists.
    pub fn process_links(&mut self) {
        info!("process_links()");

        let w: Vec<DictWordMarkdown> = self.dict_words_input.values().cloned().collect();
        let letter_groups = LetterGroups::new_from_dict_words(&w);
        let words_to_url = letter_groups.words_to_url;

        for (_key, dict_word) in self.dict_words_input.iter_mut() {
            for w in dict_word.word_header.synonyms.iter_mut() {
                *w = Dictionary::word_to_link(&self.valid_words, &words_to_url, self.output_format, w);
                *w = w.replace('&', "&amp;");
            }

            for w in dict_word.word_header.antonyms.iter_mut() {
                *w = Dictionary::word_to_link(&self.valid_words, &words_to_url, self.output_format, w);
                *w = w.replace('&', "&amp;");
            }

            for w in dict_word.word_header.homonyms.iter_mut() {
                *w = Dictionary::word_to_link(&self.valid_words, &words_to_url, self.output_format, w);
                *w = w.replace('&', "&amp;");
            }

            for w in dict_word.word_header.see_also.iter_mut() {
                *w = Dictionary::word_to_link(&self.valid_words, &words_to_url, self.output_format, w);
                *w = w.replace('&', "&amp;");
            }

            for w in dict_word.word_header.also_written_as.iter_mut() {
                *w = Dictionary::word_to_link(&self.valid_words, &words_to_url, self.output_format, w);
                *w = w.replace('&', "&amp;");
            }

            for w in dict_word.word_header.grammar_roots.iter_mut() {
                *w = Dictionary::word_to_link(&self.valid_words, &words_to_url, self.output_format, w);
                *w = w.replace('&', "&amp;");
            }

            // Replace construction words with links.

            dict_word.word_header.grammar_construction = Dictionary::all_words_to_links(
                &self.valid_words,
                &words_to_url,
                self.output_format,
                &dict_word.word_header.grammar_construction
            );

            dict_word.word_header.grammar_base_construction = Dictionary::all_words_to_links(
                &self.valid_words,
                &words_to_url,
                self.output_format,
                &dict_word.word_header.grammar_base_construction
            );

            dict_word.word_header.grammar_compound_construction = Dictionary::all_words_to_links(
                &self.valid_words,
                &words_to_url,
                self.output_format,
                &dict_word.word_header.grammar_compound_construction
            );

        }
    }
}

fn reg_tmpl(h: &mut Handlebars, k: &str, afs: &BTreeMap<String, String>) {
    h.register_template_string(k, afs.get(k).unwrap()).unwrap();
}

impl Default for Dictionary {
    fn default() -> Self {
        Dictionary::new(
            OutputFormat::Epub,
            false,
            &PathBuf::from("."),
            &PathBuf::from("ebook.epub"),
            None,
        )
    }
}

impl Default for DictMetadata {
    fn default() -> Self {
        DictMetadata {
            title: "Dictionary".to_string(),
            dict_label: "dictionary".to_string(),
            description: "".to_string(),
            creator: "".to_string(),
            email: "".to_string(),
            source: "".to_string(),
            cover_path: "default_cover.jpg".to_string(),
            book_id: "SimsapaDictionary".to_string(),
            version: "0.1.0".to_string(),
            created_date_human: "".to_string(),
            created_date_opf: "".to_string(),
            word_prefix: "".to_string(),
            word_prefix_velthuis: false,
            add_velthuis: false,
            allow_raw_html: false,
            dont_generate_synonyms: false,
        }
    }
}

fn clean_output_content(text: &str) -> String {
    let mut content = text.to_string();

    // Remove trailing spaces
    let re_trailing = Regex::new(r" +\n").unwrap();
    content = re_trailing.replace_all(&content, "\n").to_string();

    // Remove empty <p></p>
    let re_empty = Regex::new(r"<p> *</p>").unwrap();
    content = re_empty.replace_all(&content, "").to_string();

    // Remove double blanks from the output, empty attributes leave empty spaces when rendering
    // the template.
    let re_double_blanks = Regex::new(r"\n\n+").unwrap();
    content = re_double_blanks.replace_all(&content, "\n\n").to_string();

    content
}

