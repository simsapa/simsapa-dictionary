use std::process::{exit, Command};
use std::fs::{self, File};
use std::io::{self, Write};

use std::collections::BTreeMap;
use std::path::PathBuf;

use walkdir::WalkDir;
use handlebars::{self, Handlebars};

use crate::dict_word::{DictWord, DictWordHeader};
use crate::letter_groups::LetterGroups;
use crate::helpers::{md2html, markdown_helper, is_hidden};
use crate::app::ZipWith;

pub const DICTIONARY_METADATA_SEP: &str = "--- DICTIONARY METADATA ---";
pub const DICTIONARY_WORD_ENTRIES_SEP: &str = "--- DICTIONARY WORD ENTRIES ---";

#[derive(Serialize, Deserialize)]
pub struct Ebook {
    pub meta: EbookMetadata,
    pub ebook_format: EbookFormat,
    pub dict_words: BTreeMap<String, DictWord>,
    pub entries_manifest: Vec<EntriesManifest>,
    pub asset_files_string: BTreeMap<String, String>,
    pub asset_files_byte: BTreeMap<String, Vec<u8>>,

    #[serde(skip)]
    pub output_path: PathBuf,
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

#[derive(Serialize, Deserialize)]
pub struct EbookMetadata {
    pub title: String,
    pub description: String,
    pub creator: String,
    pub source: String,
    pub cover_path: String,
    pub book_id: String,
    pub created_date_human: String,
    pub created_date_opf: String,
    pub is_epub: bool,
    pub is_mobi: bool,
}

#[derive(Serialize, Deserialize)]
pub enum EbookFormat {
    Epub,
    Mobi
}

#[derive(Serialize, Deserialize)]
pub struct EntriesManifest {
    id: String,
    href: String,
}

impl Ebook {
    pub fn new(ebook_format: EbookFormat, output_path: &PathBuf) -> Self {
        // asset_files_string
        let mut afs: BTreeMap<String, String> = BTreeMap::new();
        // asset_files_byte
        let mut afb: BTreeMap<String, Vec<u8>> = BTreeMap::new();
        let mut h = Handlebars::new();

        h.register_helper("markdown", Box::new(markdown_helper));

        // Can't loop because the arg of include_str! must be a string literal.

        let k = "content-page.xhtml".to_string();
        afs.insert(k.clone(), include_str!("../assets/content-page.xhtml").to_string());
        reg_tmpl(&mut h, &k, &afs);

        let k = "about.md".to_string();
        afs.insert(k.clone(), include_str!("../assets/OEBPS/about.md").to_string());
        reg_tmpl(&mut h, &k, &afs);

        let k = "copyright.md".to_string();
        afs.insert(k.clone(), include_str!("../assets/OEBPS/copyright.md").to_string());
        reg_tmpl(&mut h, &k, &afs);

        let k = "entries-epub.xhtml".to_string();
        afs.insert(k.clone(), include_str!("../assets/OEBPS/entries-epub.xhtml").to_string());
        reg_tmpl(&mut h, &k, &afs);

        let k = "entries-mobi.xhtml".to_string();
        afs.insert(k.clone(), include_str!("../assets/OEBPS/entries-mobi.xhtml").to_string());
        reg_tmpl(&mut h, &k, &afs);

        let k = "htmltoc.xhtml".to_string();
        afs.insert(k.clone(), include_str!("../assets/OEBPS/htmltoc.xhtml").to_string());
        reg_tmpl(&mut h, &k, &afs);

        let k = "toc.ncx".to_string();
        afs.insert(k.clone(), include_str!("../assets/OEBPS/toc.ncx").to_string());
        reg_tmpl(&mut h, &k, &afs);

        let k = "package.opf".to_string();
        afs.insert(k.clone(), include_str!("../assets/OEBPS/package.opf").to_string());
        reg_tmpl(&mut h, &k, &afs);

        let k = "cover.xhtml".to_string();
        afs.insert(k.clone(), include_str!("../assets/OEBPS/cover.xhtml").to_string());
        reg_tmpl(&mut h, &k, &afs);

        let k = "titlepage.xhtml".to_string();
        afs.insert(k.clone(), include_str!("../assets/OEBPS/titlepage.xhtml").to_string());
        reg_tmpl(&mut h, &k, &afs);

        afb.insert("cover.jpg".to_string(), include_bytes!("../assets/OEBPS/cover.jpg").to_vec());

        afb.insert("style.css".to_string(), include_bytes!("../assets/OEBPS/style.css").to_vec());

        afb.insert("container.xml".to_string(), include_bytes!("../assets/META-INF/container.xml").to_vec());

        afb.insert("com.apple.ibooks.display-options.xml".to_string(), include_bytes!("../assets/META-INF/com.apple.ibooks.display-options.xml").to_vec());

        let mut meta = EbookMetadata::default();
        match ebook_format {
            EbookFormat::Epub => {
                meta.is_epub = true;
                meta.is_mobi = false;
            },
            EbookFormat::Mobi => {
                meta.is_epub = false;
                meta.is_mobi = true;
            }
        }

        Ebook {
            meta,
            ebook_format,
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
        let w_key = format!("{} {}", new_word.word_header.word, new_word.word_header.dict_label);

        if self.dict_words.contains_key(&w_key) {
            warn!("Double word: '{}'. Entries should be unique for word within one dictionary.", &w_key);

            let ww = DictWord {
                word_header: DictWordHeader {
                    dict_label: new_word.word_header.dict_label,
                    word: format!("{} FIXME: double", new_word.word_header.word),
                    summary: new_word.word_header.summary,
                    grammar: new_word.word_header.grammar,
                    inflections: new_word.word_header.inflections,
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

    pub fn write_markdown(&self) {
        info!("write_markdown()");

        let mut file = File::create(&self.output_path).unwrap();

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

    pub fn create_ebook_build_folders(&mut self) {
        info!("create_ebook_build_folders()");

        if self.output_path.exists() {
            fs::remove_file(&self.output_path).expect("Can't remove old output file.");
        }

        // Store full paths (canonicalized) in the Ebook attribs. canonicalize() requires that the
        // path should exist, so take the parent of output_path first before canonicalize().

        let build_base_dir = self.output_path.parent().unwrap().canonicalize().unwrap().join("ebook-build");
        if !build_base_dir.exists() {
            fs::create_dir(&build_base_dir).unwrap();
        }
        self.build_base_dir = Some(build_base_dir.clone());

        if let EbookFormat::Epub = self.ebook_format {
            self.mimetype_path = Some(build_base_dir.join("mimetype"));

            let meta_inf_dir = build_base_dir.join("META-INF");
            if !meta_inf_dir.exists() {
                fs::create_dir(&meta_inf_dir).unwrap();
            }
            self.meta_inf_dir = Some(meta_inf_dir);
        }

        let oebps_dir = build_base_dir.join("OEBPS");
        if !oebps_dir.exists() {
            fs::create_dir(&oebps_dir).unwrap();
        }
        self.oebps_dir = Some(oebps_dir);
    }

    /// Write entries split in letter groups.
    pub fn write_entries(&mut self) {
        info!("write_entries()");

        let w: Vec<DictWord> = self.dict_words.values().cloned().collect();
        let mut groups = LetterGroups::new_from_dict_words(&w);

        info!("Writing {} letter groups ...", groups.len());

        let template_name = match self.ebook_format {
            EbookFormat::Epub => "entries-epub.xhtml",
            EbookFormat::Mobi => "entries-mobi.xhtml",
        };

        for (order_idx, group) in groups.groups.values_mut().enumerate() {

            if order_idx == 0 {
                group.title = self.meta.title.clone();
            }

            let content_html = match self.templates.render(template_name, &group) {
                Ok(x) => x,
                Err(e) => {
                    error!("Can't render template {}, {:?}", template_name, e);
                    "FIXME: Template rendering error.".to_string()
                }
            };

            let mut d: BTreeMap<String, String> = BTreeMap::new();
            d.insert("page_title".to_string(), self.meta.title.clone());
            d.insert("content_html".to_string(), content_html);
            let file_content = self.templates.render("content-page.xhtml", &d).unwrap();

            // The file names will be identified by index number, not the group letter.
            // entries-00.xhtml, entries-01.xhtml and so on.

            let group_file_name = format!("entries-{:02}.xhtml", order_idx);
            self.entries_manifest.push(
                EntriesManifest {
                    id: format!("item_entries_{:02}", order_idx),
                    href: group_file_name.clone(),
                });

            let dir = self.oebps_dir.as_ref().expect("missing oebps_dir");
            let mut file = File::create(dir.join(group_file_name)).unwrap();
            file.write_all(file_content.as_bytes()).unwrap();
        }
    }

    /// Write package.opf.
    pub fn write_package(&mut self) {
        info!("write_package()");

        let filename = "package.opf";
        let file_content = self.templates.render(filename, &self).unwrap();

        let dir = self.oebps_dir.as_ref().expect("missing oebps_dir");
        let mut file = File::create(dir.join(filename)).unwrap();
        file.write_all(file_content.as_bytes()).unwrap();
    }

    /// Write htmltoc.xhtml.
    pub fn write_html_toc(&mut self) {
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
        let file_content = self.templates.render("content-page.xhtml", &d).unwrap();

        let dir = self.oebps_dir.as_ref().expect("missing oebps_dir");
        let mut file = File::create(dir.join(filename)).unwrap();
        file.write_all(file_content.as_bytes()).unwrap();
    }

    /// Write toc.ncx.
    pub fn write_ncx_toc(&mut self) {
        info!("write_ncx_toc()");

        let filename = "toc.ncx".to_string();

        let file_content = match self.templates.render(&filename, &self) {
            Ok(x) => x,
            Err(e) => {
                error!("Can't render template {}, {:?}", filename, e);
                "FIXME: Template rendering error.".to_string()
            }
        };

        let dir = self.oebps_dir.as_ref().expect("missing oebps_dir");
        let mut file = File::create(dir.join(filename)).unwrap();
        file.write_all(file_content.as_bytes()).unwrap();
    }

    /// Write cover.xhtml.
    pub fn write_cover(&mut self) {
        info!("write_cover()");

        let filename = "cover.xhtml".to_string();

        let file_content = match self.templates.render(&filename, &self) {
            Ok(x) => x,
            Err(e) => {
                error!("Can't render template {}, {:?}", filename, e);
                "FIXME: Template rendering error.".to_string()
            }
        };

        let dir = self.oebps_dir.as_ref().expect("missing oebps_dir");
        let mut file = File::create(dir.join(filename)).unwrap();
        file.write_all(file_content.as_bytes()).unwrap();
    }

    /// Write titlepage.xhtml.
    pub fn write_titlepage(&mut self) {
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
        let file_content = self.templates.render("content-page.xhtml", &d).unwrap();

        let dir = self.oebps_dir.as_ref().expect("missing oebps_dir");
        let mut file = File::create(dir.join(filename)).unwrap();
        file.write_all(file_content.as_bytes()).unwrap();
    }

    /// Write about.xhtml.
    pub fn write_about(&mut self) {
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
        let file_content = self.templates.render("content-page.xhtml", &d).unwrap();

        let dest_name = filename.replace(".md", ".xhtml");
        let dir = self.oebps_dir.as_ref().expect("missing oebps_dir");
        let mut file = File::create(dir.join(dest_name)).unwrap();
        file.write_all(file_content.as_bytes()).unwrap();
    }

    /// Write copyright.xhtml.
    pub fn write_copyright(&mut self) {
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
        let file_content = self.templates.render("content-page.xhtml", &d).unwrap();

        let dest_name = filename.replace(".md", ".xhtml");
        let dir = self.oebps_dir.as_ref().expect("missing oebps_dir");
        let mut file = File::create(dir.join(dest_name)).unwrap();
        file.write_all(file_content.as_bytes()).unwrap();
    }

    /// Copy static assets.
    pub fn copy_static(&self) {
        info!("copy_static()");

        let dir = self.oebps_dir.as_ref().expect("missing oebps_dir");
        for filename in ["cover.jpg", "style.css"].iter() {
            let file_content = self.asset_files_byte.get(&filename.to_string()).unwrap();
            let mut file = File::create(dir.join(filename)).unwrap();
            file.write_all(file_content).unwrap();
        }
    }

    pub fn write_mimetype(&self) {
        info!("write_mimetype()");

        let p = self.mimetype_path.as_ref().expect("missing mimetype_path");
        let mut file = File::create(&p).unwrap();
        file.write_all(b"application/epub+zip").unwrap();
    }

    pub fn write_meta_inf_files(&self) {
        info!("write_meta_inf_files()");

        let dir = self.meta_inf_dir.as_ref().expect("missing meta_inf_dir");
        for filename in ["container.xml", "com.apple.ibooks.display-options.xml"].iter() {
            let file_content = self.asset_files_byte.get(&filename.to_string()).unwrap();
            let mut file = File::create(dir.join(filename)).unwrap();
            file.write_all(file_content).unwrap();
        }
    }

    pub fn write_oebps_files(&mut self) {
        info!("write_oebps_files()");

        if let EbookFormat::Epub = self.ebook_format {
            self.write_cover();
        }

        self.write_entries();
        self.write_package();
        self.write_html_toc();
        self.write_ncx_toc();

        self.write_titlepage();
        self.write_about();
        self.write_copyright();

        self.copy_static();
    }

    fn zip_with_shell(&self) {
        info!("zip_with_shell()");

        let d = self.build_base_dir.as_ref().unwrap();
        let dir: &str = d.to_str().unwrap();
        let epub_name: &str = self.output_path.file_name().unwrap().to_str().unwrap();

        let shell_cmd = format!(r#"cd "{}" && zip -X0 "../{}" mimetype && zip -rg "../{}" META-INF -x \*.DS_Store && zip -rg "../{}" OEBPS -x \*.DS_Store"#, dir, epub_name, epub_name, epub_name);

        let output = match Command::new("sh").arg("-c").arg(shell_cmd).output() {
            Ok(o) => o,
            Err(e) => {
                error!("🔥 Failed to Zip: {:?}", e);
                exit(2);
            }
        };

        if output.status.success() {
            info!("🔎 Zip successful.");
        } else {
            error!("🔥 Zip exited with an error.");
        }
    }

    fn zip_with_lib(&self) {
        info!("zip_with_lib()");

        let mut buf: Vec<u8> = Vec::new();
        {
            let mut w = std::io::Cursor::new(&mut buf);
            let mut zip = zip::ZipWriter::new(&mut w);

            // NOTE: Path names in Epub Zip files must always use '/' forward-slash, even on Windows.

            // mimetype file first, not compressed
            {
                let o = zip::write::FileOptions::default().compression_method(zip::CompressionMethod::Stored);
                zip.start_file("mimetype", o).unwrap();
                zip.write_all(b"application/epub+zip").unwrap();
            }

            // META-INF folder
            //
            // Only has two files, which are not templated, no need to read them. Retreive from
            // asset_files_byte.

            {
                let o = zip::write::FileOptions::default();

                zip.start_file("META-INF/com.apple.ibooks.display-options.xml", o).unwrap();
                zip.write_all(self.asset_files_byte.get("com.apple.ibooks.display-options.xml").unwrap()).unwrap();

                zip.start_file("META-INF/container.xml", o).unwrap();
                zip.write_all(self.asset_files_byte.get("container.xml").unwrap()).unwrap();
            }

            // OEBPS folder
            //
            // Walk the contents. Not recursive, we are storing them all in one folder.

            {
                let o = zip::write::FileOptions::default();
                let dir = self.oebps_dir.as_ref().expect("missing oebps dir");
                let walker = WalkDir::new(dir).into_iter();

                // is_hidden will also catch .DS_Store
                for entry in walker.filter_entry(|e| !is_hidden(e)) {
                    let entry = entry.unwrap();

                    // First entry will be the OEBPS folder.
                    if entry.file_name().to_str().unwrap() == "OEBPS" {
                        continue;
                    }
                    if entry.path().is_dir() {
                        info!("Skipping dir entry '{}'", entry.path().to_str().unwrap());
                        continue;
                    }

                    let contents: Vec<u8> = fs::read(&entry.path()).expect("Can't read the file.");

                    let name = entry.file_name().to_str().unwrap();
                    // not using .join() to avoid getting a back-slash on Windows
                    zip.start_file(format!("OEBPS/{}", name), o).unwrap();
                    zip.write_all(&contents).unwrap();
                }
            }

            zip.finish().unwrap();
        }

        let mut file = File::create(&self.output_path).unwrap();
        file.write_all(&buf).unwrap();
    }

    pub fn zip_files_as_epub(&self, zip_with: ZipWith) {
        info!("zip_files_as_epub()");

        match zip_with {
            ZipWith::ZipLib => self.zip_with_lib(),
            ZipWith::ZipCli => self.zip_with_shell(),
        }
    }

    pub fn run_kindlegen(&self, kindlegen_path: &PathBuf, mobi_compression: usize) {
        info!("run_kindlegen()");

        let oebps_dir = self.oebps_dir.as_ref().unwrap();
        let opf_path = oebps_dir.join(PathBuf::from("package.opf"));
        let output_file_name = self.output_path.file_name().unwrap();

        let mut k = kindlegen_path.to_str().unwrap().trim();
        if cfg!(target_os = "windows") {
            k = clean_windows_str_path(k);
        }

        let bin_cmd = format!("{} \"{}\" -c{} -dont_append_source -o {}",
            k,
            opf_path.to_str().unwrap(),
            mobi_compression,
            output_file_name.to_str().unwrap());

        info!("bin_cmd: {:?}", bin_cmd);

        info!("🔎 Running KindleGen ...");
        if mobi_compression == 2 {
            info!("Note that compression level 2 can take some time to complete.");
        }

        let output = if cfg!(target_os = "windows") {
            match Command::new("cmd").arg("/C").arg(bin_cmd).output() {
                Ok(o) => o,
                Err(e) => {
                    error!("🔥 Failed to run KindleGen: {:?}", e);
                    exit(2);
                }
            }
        } else {
            match Command::new("sh").arg("-c").arg(bin_cmd).output() {
                Ok(o) => o,
                Err(e) => {
                    error!("🔥 Failed to run KindleGen: {:?}", e);
                    exit(2);
                }
            }
        };

        if output.status.success() {
            info!("🔎 KindleGen finished successfully.");
        } else {
            error!("🔥 KindleGen exited with an error.");
            io::stdout().write_all(&output.stdout).unwrap();
            io::stderr().write_all(&output.stderr).unwrap();
            exit(2);
        }

        // Move the generate MOBI to its path. KindleGen puts the MOBI in the same folder with package.opf.
        fs::rename(oebps_dir.join(output_file_name), &self.output_path).unwrap();
    }

    pub fn remove_generated_files(&mut self) {
        if let Some(dir) = self.build_base_dir.as_ref() {
            fs::remove_dir_all(dir).unwrap();
        }
        self.set_paths_to_none();
    }

    fn set_paths_to_none(&mut self) {
        // TODO these should be grouped in one type, all depending on base dir being created.
        self.build_base_dir = None;
        self.mimetype_path = None;
        self.meta_inf_dir = None;
        self.oebps_dir = None;
    }
}

fn reg_tmpl(h: &mut Handlebars, k: &str, afs: &BTreeMap<String, String>) {
    h.register_template_string(k, afs.get(k).unwrap()).unwrap();
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
            created_date_human: "".to_string(),
            created_date_opf: "".to_string(),
            is_epub: true,
            is_mobi: false,
        }
    }
}

fn clean_windows_str_path(p: &str) -> &str {
    p.trim_start_matches("\\\\?\\")
}
