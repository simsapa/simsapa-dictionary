use std::error::Error;
use std::fs::{self, File};
use std::io::{self, Write};
use std::process::Command;

use std::collections::BTreeMap;
use std::path::PathBuf;

use handlebars::{self, Handlebars};
use walkdir::WalkDir;
use regex::Regex;
use deunicode::deunicode;

use crate::app::{AppStartParams, ZipWith};
use crate::dict_word::DictWord;
use crate::error::ToolError;
use crate::helpers::{self, is_hidden, md2html};
use crate::letter_groups::{LetterGroups, LetterGroup};
use crate::pali;

pub const DICTIONARY_METADATA_SEP: &str = "--- DICTIONARY METADATA ---";
pub const DICTIONARY_WORD_ENTRIES_SEP: &str = "--- DICTIONARY WORD ENTRIES ---";

#[derive(Serialize, Deserialize)]
pub struct Ebook {
    pub meta: EbookMetadata,
    pub output_format: OutputFormat,
    pub dict_words: BTreeMap<String, DictWord>,
    pub entries_manifest: Vec<EntriesManifest>,
    pub asset_files_string: BTreeMap<String, String>,
    pub asset_files_byte: BTreeMap<String, Vec<u8>>,

    #[serde(skip)]
    pub output_path: PathBuf,

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
pub struct EbookMetadata {

    pub title: String,

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
    pub use_velthuis: bool,
}

#[derive(Serialize, Deserialize, Copy, Clone)]
pub enum OutputFormat {
    Epub,
    Mobi,
    BabylonGls,
    StardictXml,
}

#[derive(Serialize, Deserialize)]
pub struct EntriesManifest {
    id: String,
    href: String,
}

#[derive(Serialize, Deserialize)]
pub struct LetterGroupTemplateData {
    group: LetterGroup,
    meta: EbookMetadata,
}

impl Ebook {
    pub fn new(output_format: OutputFormat, output_path: &PathBuf) -> Self {
        // asset_files_string
        let mut afs: BTreeMap<String, String> = BTreeMap::new();
        // asset_files_byte
        let mut afb: BTreeMap<String, Vec<u8>> = BTreeMap::new();
        let mut h = Handlebars::new();

        h.register_helper("markdown", Box::new(helpers::markdown_helper));
        h.register_helper("to_velthuis", Box::new(helpers::to_velthuis));
        h.register_helper("word_list", Box::new(helpers::word_list));
        h.register_helper("grammar_phonetic_transliteration", Box::new(helpers::grammar_phonetic_transliteration));

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

        let k = "stardict_textual.xml".to_string();
        afs.insert(
            k.clone(),
            include_str!("../assets/stardict_textual.xml").to_string(),
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

        Ebook {
            meta: EbookMetadata::default(),
            output_format,
            dict_words: BTreeMap::new(),
            entries_manifest: Vec::new(),
            asset_files_string: afs,
            asset_files_byte: afb,
            output_path: output_path.to_path_buf(),
            build_base_dir: None,
            mimetype_path: None,
            meta_inf_dir: None,
            oebps_dir: None,
            templates: h,
        }
    }

    pub fn add_word(&mut self, new_word: DictWord) {
        let mut new_word = new_word;

        let label = if new_word.word_header.dict_label.is_empty() {
            "unlabeled".to_string()
        } else {
            new_word.word_header.dict_label.clone()
        };
        let grammar = if new_word.word_header.grammar.is_empty() {
            "uncategorized".to_string()
        } else {
            new_word.word_header.grammar.clone()
        };
        let w_key = format!(
            "{} {} {}",
            new_word.word_header.word, grammar, label
        );

        // Add transliterations to help searching:
        // - given with the transliteration attribute
        // - velthuis
        // - ascii

        if !new_word.word_header.transliteration.is_empty() {
            new_word.word_header.inflections.push(new_word.word_header.transliteration.clone());
        }

        if self.meta.use_velthuis {
            let s = pali::to_velthuis(&new_word.word_header.word);
            if !new_word.word_header.inflections.contains(&s) && s != new_word.word_header.word {
                new_word.word_header.inflections.push(s);
            }
        }

        {
            let s = deunicode(&new_word.word_header.word);
            if !new_word.word_header.inflections.contains(&s) && s != new_word.word_header.word {
                new_word.word_header.inflections.push(s);
            }
        }

        if self.dict_words.contains_key(&w_key) {
            warn!(
                "Double word: '{}'. Entries should be unique for word within one dictionary.",
                &w_key
            );

            new_word.word_header.word = format!("{} FIXME: double", new_word.word_header.word);
            let double_key = format!("{} double", &w_key);
            self.dict_words.insert(double_key, new_word);
        } else {
            let w = self.dict_words.insert(w_key.clone(), new_word);
            if w.is_some() {
                error!(
                    "Unhandled double word '{}', new value replacing the old.",
                    w_key
                );
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

    pub fn entries_as_markdown(&self) -> String {
        info!("entries_as_markdown()");
        self.dict_words
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

        // Store full paths (canonicalized) in the Ebook attribs. canonicalize() requires that the
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

        let w: Vec<DictWord> = self.dict_words.values().cloned().collect();
        let mut groups = LetterGroups::new_from_dict_words(&w);

        info!("Writing {} letter groups ...", groups.len());

        let template_name = match self.output_format {
            OutputFormat::Epub => "entries-epub.xhtml",
            OutputFormat::Mobi => "entries-mobi.xhtml",
            _ => {
                let msg = "🔥 Only Epub or Mobi makes sense here.".to_string();
                return Err(Box::new(ToolError::Exit(msg)));
            }
        };

        for (order_idx, group) in groups.groups.values_mut().enumerate() {
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

            // The file names will be identified by index number, not the group letter.
            // entries-00.xhtml, entries-01.xhtml and so on.

            let group_file_name = format!("entries-{:02}.xhtml", order_idx);
            self.entries_manifest.push(EntriesManifest {
                id: format!("item_entries_{:02}", order_idx),
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

        let content_html = md2html(&content_md);

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

        let content_html = md2html(&content_md);

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

        let dir = self.oebps_dir.as_ref().ok_or("missing oebps_dir")?;
        let base = self.build_base_dir.as_ref().ok_or("missing build_base_dir")?;

        // cover image
        {
            let filename = self.meta.cover_path.clone();
            // Cover path is relative to the folder of the source input file, which is the parent
            // of the build base dir.
            let p = base.parent().unwrap().join(PathBuf::from(filename.clone()));
            if p.exists() {
                // If the file is found, copy that.
                fs::copy(&p, dir.join(filename))?;
            } else {
                // If not found, try looking it up in the embedded assets.
                let file_content = self
                    .asset_files_byte
                    .get(&filename.to_string())
                    .ok_or("missing get key")?;
                let mut file = File::create(dir.join(filename))?;
                file.write_all(file_content)?;
            }
        }

        // stylesheet
        {
            let filename = "style.css";
            let file_content = self
                .asset_files_byte
                .get(&filename.to_string())
                .ok_or("missing get key")?;
            let mut file = File::create(dir.join(filename))?;
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
                .get(&filename.to_string())
                .ok_or("missing get key")?;
            let mut file = File::create(dir.join(filename))?;
            file.write_all(file_content)?;
        }

        Ok(())
    }

    pub fn write_oebps_files(&mut self) -> Result<(), Box<dyn Error>> {
        info!("write_oebps_files()");

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

        self.copy_static()?;

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
        kindlegen_path: &PathBuf,
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
        for (_, word) in self.dict_words.iter() {
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

            // grammar, phonetic, transliteration
            let s = helpers::format_grammar_phonetic_transliteration(
                &word.word_header.word,
                &word.word_header.grammar,
                &word.word_header.phonetic,
                &word.word_header.transliteration,
                self.meta.use_velthuis);

            text.push_str(&s);

            // definition
            text.push_str(&helpers::md2html(&word.definition_md));

            // synonyms
            if !word.word_header.synonyms.is_empty() {
                let s: String = word.word_header.synonyms.iter()
                    .map(|i| format!("<a href=\"bword://{}\">{}</a>", i, i))
                    .collect::<Vec<String>>()
                    .join(", ");

                text.push_str(&format!("<p>Synonyms: {}</p>", &s));
            }

            // antonyms
            if !word.word_header.antonyms.is_empty() {
                let s: String = word.word_header.antonyms.iter()
                    .map(|i| format!("<a href=\"bword://{}\">{}</a>", i, i))
                    .collect::<Vec<String>>()
                    .join(", ");

                text.push_str(&format!("<p>Antonyms: {}</p>", &s));
            }

            // see also
            if !word.word_header.see_also.is_empty() {
                let s: String = word.word_header.see_also.iter()
                    .map(|i| format!("<a href=\"bword://{}\">{}</a>", i, i))
                    .collect::<Vec<String>>()
                    .join(", ");

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

        let template = "stardict_textual.xml".to_string();
        let content = match self.templates.render(&template, &self) {
            Ok(x) => x,
            Err(e) => {
                error!("Can't render template {}, {:?}", template, e);
                "FIXME: Template rendering error.".to_string()
            }
        };

        let mut file = File::create(&self.output_path)?;
        file.write_all(content.as_bytes())?;

        Ok(())
    }

    pub fn create_stardict(&mut self) -> Result<(), Box<dyn Error>> {
        self.write_stardict_xml()?;
        Ok(())
    }

    fn set_paths_to_none(&mut self) {
        // TODO these should be grouped in one type, all depending on base dir being created.
        self.build_base_dir = None;
        self.mimetype_path = None;
        self.meta_inf_dir = None;
        self.oebps_dir = None;
    }

    pub fn process_see_also_from_definition(&mut self) {
        // [abbha(t)](/define/abbha(t))
        // [abhu(ṃ)](/define/abhu(ṃ)
        // FIXME (n) (t)
        let re_define = Regex::new(r"\[([^\]]+)\]\(/define/([^\(\)]+)\)").unwrap();
        // We're going to temporarily replace links as [[abbha]]
        let re_bracket_links = Regex::new(r"\[\[([^]]+)\]\]").unwrap();
        // (see also *[abbuhati](/define/abbuhati)* and *[abbūhati](/define/abbūhati)*)
        // (see *[abbha](/define/abbha)*)
        let re_see_also = Regex::new(r" *\(see ([^\)]+)\)").unwrap();

        // words with and without italics (stars) have to be covered
        // word must be min. 3 chars long
        // (also *anūpanāhi(n)*)
        // (also *abhisāpeti*)
        // (see *[upanāhi(n)](/define/upanāhi(n))*)
        // (see *[upaparikkha(t)](/define/upaparikkha(t))*)
        // FIXME (n) (t) (\([a-z]\)?)

        let re_also_one_plain = Regex::new(r"\(also +([^\*\(\),]{3,})\)").unwrap();
        let re_also_one_italics = Regex::new(r"\(also +\*([^\*\(\),]{3,})\*\)").unwrap();
        // (also *abhisaṅkhaṭa* and *abhisaṅkhita*)
        let re_also_two_plain = Regex::new(r"\(also +([^\*\(\), ]{3,})(, +| +and +|, +and +| +& +)([^\*\(\)]{3,})\)").unwrap();
        let re_also_two_italics = Regex::new(r"\(also +\*([^\*\(\), ]{3,})\*(, +| +and +|, +and +| +& +)\*([^\*\(\)]{3,})\*\)").unwrap();
        // (also *apabyūhati*, *apaviyūhati*, and *apabbūhati*)
        let re_also_three_plain = Regex::new(r"\(also +([^\*\(\), ]{3,}), +([^\*\(\), ]{3,})(, +| +and +|, +and +| +& +)([^\*\(\)]{3,})\)").unwrap();
        let re_also_three_italics = Regex::new(r"\(also +\*([^\*\(\), ]{3,})\*, +\*([^\*\(\), ]{3,})\*(, +| +and +|, +and +| +& +)\*([^\*\(\)]{3,})\*\)").unwrap();

        // FIXME have to check valid words

        for (_, w) in self.dict_words.iter_mut() {
            let mut def: String = w.definition_md.clone();

            // (also *abhisāpeti*) -> (see [abhisāpeti](/define/abhisāpeti))
            def = re_also_three_italics.replace_all(&def, "(see [$1](/define/$1), [$2](/define/$2) and [$4](/define/$4))").to_string();
            def = re_also_three_plain.replace_all(&def, "(see [$1](/define/$1), [$2](/define/$2) and [$4](/define/$4))").to_string();

            def = re_also_two_italics.replace_all(&def, "(see [$1](/define/$1) and [$3](/define/$3))").to_string();
            def = re_also_two_plain.replace_all(&def, "(see [$1](/define/$1) and [$3](/define/$3))").to_string();

            def = re_also_one_italics.replace_all(&def, "(see [$1](/define/$1))").to_string();
            def = re_also_one_plain.replace_all(&def, "(see [$1](/define/$1))").to_string();

            // Collect /define links from the text and add to see_also list.

            for link in re_define.captures_iter(&def) {
                w.word_header.see_also.push(link[2].to_string());
            }

            // [wordlabel](/define/wordlink) -> [[wordlink]]
            def = re_define.replace_all(&def, "[[$2]]").to_string();
            // Remove 'See also' from the text.
            def = re_see_also.replace_all(&def, "").to_string();
            // [[wordlink]] -> [wordlink](bword://wordlink)
            def = re_bracket_links.replace_all(&def, "[$1](bword://$1)").to_string();
            // replace remaining /define links as bword://
            def = re_define.replace_all(&def, "[$1](bword://$2)").to_string();

            w.definition_md = def;
        }
    }
}

fn reg_tmpl(h: &mut Handlebars, k: &str, afs: &BTreeMap<String, String>) {
    h.register_template_string(k, afs.get(k).unwrap()).unwrap();
}

impl Default for Ebook {
    fn default() -> Self {
        Ebook::new(OutputFormat::Epub, &PathBuf::from("ebook.epub"))
    }
}

impl Default for EbookMetadata {
    fn default() -> Self {
        EbookMetadata {
            title: "Dictionary".to_string(),
            description: "Pali - English".to_string(),
            creator: "Simsapa Dhamma Reader".to_string(),
            email: "person@example.com".to_string(),
            source: "https://simsapa.github.io".to_string(),
            cover_path: "default_cover.jpg".to_string(),
            book_id: "SimsapaPaliDictionary".to_string(),
            version: "0.1.0".to_string(),
            created_date_human: "".to_string(),
            created_date_opf: "".to_string(),
            word_prefix: "".to_string(),
            use_velthuis: false,
        }
    }
}

