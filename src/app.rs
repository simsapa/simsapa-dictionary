use std::default::Default;
use std::error::Error;
use std::fs;
use std::process::Command;
use std::path::{Path, PathBuf};

use regex::Regex;
use walkdir::WalkDir;
use chrono::prelude::*;

use crate::ebook::{Ebook, EbookMetadata, EbookFormat, DICTIONARY_METADATA_SEP, DICTIONARY_WORD_ENTRIES_SEP};
use crate::dict_word::{DictWord, DictWordHeader};
use crate::helpers::{is_hidden, ensure_parent, ensure_parent_all};
use crate::error::ToolError;

pub struct AppStartParams {
    pub ebook_format: EbookFormat,
    pub json_path: Option<PathBuf>,
    pub nyanatiloka_root: Option<PathBuf>,
    pub markdown_paths: Option<Vec<PathBuf>>,
    pub output_path: Option<PathBuf>,
    pub mobi_compression: usize,
    pub kindlegen_path: Option<PathBuf>,
    pub title: Option<String>,
    pub dict_label: Option<String>,
    pub dont_run_kindlegen: bool,
    pub dont_remove_generated_files: bool,
    pub run_command: RunCommand,
    pub show_logs: bool,
    pub zip_with: ZipWith,
    pub used_first_arg: bool,
}

#[derive(Clone, Copy, Debug)]
pub enum RunCommand {
    NoOp,
    SuttaCentralJsonToMarkdown,
    NyanatilokaToMarkdown,
    MarkdownToEbook,
}

#[derive(Clone, Copy, Debug)]
pub enum ZipWith {
    ZipLib,
    ZipCli,
}

impl Default for AppStartParams {
    fn default() -> Self {

        // Zip cli tool is not usually available on Windows, so we zip with lib there.
        //
        // lise-henry/epub-builder notes that zipping epub with the lib sometimes gave her errors
        // with the Kobo reader, and relies on the cli zip when available.
        //
        // Hence on Linux and Mac we zip with the cli zip.

        let zip_with = if cfg!(target_os = "windows") {
            ZipWith::ZipLib
        } else {
            ZipWith::ZipCli
        };

        AppStartParams {
            ebook_format: EbookFormat::Epub,
            json_path: None,
            nyanatiloka_root: None,
            markdown_paths: None,
            output_path: None,
            kindlegen_path: None,
            title: None,
            dict_label: None,
            mobi_compression: 0,
            dont_run_kindlegen: false,
            dont_remove_generated_files: false,
            run_command: RunCommand::NoOp,
            show_logs: false,
            zip_with,
            used_first_arg: false,
        }
    }
}

/// Parse the 1st argument if given, and set the default action if applicable. Default action is
/// to take a Markdown file and generate a MOBI dict.
pub fn process_first_arg() -> Option<AppStartParams> {
    info!("process_first_arg()");
    let mut args = std::env::args();

    // There must be exactly two args: 0. as the bin path, 1. as the first arg.
    if args.len() != 2 {
        return None;
    }

    let _bin_path = args.next();
    let markdown_path = if let Some(a) = args.next() {
        PathBuf::from(a)
    } else {
        return None;
    };

    if !markdown_path.exists() {
        return None;
    }

    if "md" != markdown_path.extension().unwrap() {
        return None;
    }

    let mut params = AppStartParams::default();
    params.used_first_arg = true;
    params.markdown_paths = Some(vec![ensure_parent(&markdown_path)]);

    params.kindlegen_path = look_for_kindlegen();

    params.ebook_format = if params.kindlegen_path.is_some() {
        EbookFormat::Mobi
    } else {
        EbookFormat::Epub
    };

    let filename = markdown_path.file_name().unwrap();
    let dir = markdown_path.parent().unwrap();

    let file_ext = if params.kindlegen_path.is_some() {
        "mobi"
    } else {
        "epub"
    };
    params.output_path = Some(dir.join(PathBuf::from(filename).with_extension(file_ext)));

    params.run_command = RunCommand::MarkdownToEbook;

    Some(params)
}

/// Look for the kindlegen executable. Either in the current working directory, or the system PATH.
fn look_for_kindlegen() -> Option<PathBuf> {
    // Try if it is in the local folder.
    let path = if cfg!(target_os = "windows") {
        PathBuf::from(".").join(PathBuf::from("kindlegen.exe"))
    } else {
        PathBuf::from(".").join(PathBuf::from("kindlegen"))
    };

    if path.exists() {
        Some(path)
    } else {
        // Try if it is available from the system PATH.

        let output = if cfg!(target_os = "windows") {
            match Command::new("cmd").arg("/C").arg("where kindlegen.exe").output() {
                Ok(o) => o,
                Err(e) => {
                    warn!("🔥 Failed to find KindleGen: {:?}", e);
                    return None;
                }
            }
        } else {
            match Command::new("sh").arg("-c").arg("which kindlegen").output() {
                Ok(o) => o,
                Err(e) => {
                    warn!("🔥 Failed to find KindleGen: {:?}", e);
                    return None;
                }
            }
        };

        if output.status.success() {
            // Output ends with a newline, must be trimmed.
            let s = std::str::from_utf8(&output.stdout).unwrap().trim();
            info!("🔎 Found KindleGen in: {}", s);
            Some(PathBuf::from(s))
        } else {
            warn!("🔥 Failed to find KindleGen.");
            None
        }
    }
}

#[allow(clippy::cognitive_complexity)]
pub fn process_cli_args(matches: clap::ArgMatches) -> Result<AppStartParams, Box<dyn Error>> {
    let mut params = AppStartParams::default();

    if let Some(sub_matches) = matches.subcommand_matches("suttacentral_json_to_markdown") {

        if let Ok(x) = sub_matches
            .value_of("json_path")
            .unwrap()
            .parse::<String>()
        {
            let path = PathBuf::from(&x);
            if path.exists() {
                params.json_path = Some(path);
            } else {
                let msg = format!("🔥 Path does not exist: {:?}", &path);
                return Err(Box::new(ToolError::Exit(msg)));
            }
        }

        if let Ok(x) = sub_matches
            .value_of("markdown_path")
            .unwrap()
            .parse::<String>()
        {
            params.markdown_paths = Some(vec![PathBuf::from(&x)]);
        }

        if let Ok(x) = sub_matches
            .value_of("dict_label")
            .unwrap()
            .parse::<String>()
        {
            params.dict_label = Some(x);
        }

        params.run_command = RunCommand::SuttaCentralJsonToMarkdown;

    } else if let Some(sub_matches) = matches.subcommand_matches("nyanatiloka_to_markdown") {

        if let Ok(x) = sub_matches
            .value_of("nyanatiloka_root")
            .unwrap()
            .parse::<String>()
        {
            let path = PathBuf::from(&x);
            if path.is_dir() {
                params.nyanatiloka_root = Some(path);
            } else {
                let msg = format!("🔥 Path does not exist: {:?}", &path);
                return Err(Box::new(ToolError::Exit(msg)));
            }
        }

        if let Ok(x) = sub_matches
            .value_of("markdown_path")
            .unwrap()
            .parse::<String>()
        {
            params.markdown_paths = Some(vec![PathBuf::from(&x)]);
        }

        if let Ok(x) = sub_matches
            .value_of("dict_label")
            .unwrap()
            .parse::<String>()
        {
            params.dict_label = Some(x);
        }

        params.run_command = RunCommand::NyanatilokaToMarkdown;

    } else if let Some(sub_matches) = matches.subcommand_matches("markdown_to_ebook") {

        if sub_matches.is_present("ebook_format") {
            if let Ok(x) = sub_matches
                .value_of("ebook_format")
                    .unwrap()
                    .parse::<String>()
            {
                let s = x.trim().to_lowercase();
                if s == "epub" {
                    params.ebook_format = EbookFormat::Epub;
                } else if s == "mobi" {
                    params.ebook_format = EbookFormat::Mobi;
                } else {
                    params.ebook_format = EbookFormat::Epub;
                }
            }

        }

        if !sub_matches.is_present("markdown_path") && !sub_matches.is_present("markdown_paths_list") {
            let msg = "🔥 Either 'markdown_path' or 'markdown_paths_list' must be used with command 'markdown_to_mobi'.".to_string();
            return Err(Box::new(ToolError::Exit(msg)));
        }

        if sub_matches.is_present("markdown_path") {
            if let Ok(x) = sub_matches
                .value_of("markdown_path")
                    .unwrap()
                    .parse::<String>()
            {
                let path = PathBuf::from(&x);
                if path.exists() {
                    params.markdown_paths = Some(vec![path]);
                } else {
                    let msg = format!("🔥 Path does not exist: {:?}", &path);
                    return Err(Box::new(ToolError::Exit(msg)));
                }
            }
        }

        if sub_matches.is_present("title") {
            if let Ok(x) = sub_matches
                .value_of("title")
                    .unwrap()
                    .parse::<String>()
            {
                params.title = Some(x);
            }
        }

        if sub_matches.is_present("dict_label") {
            if let Ok(x) = sub_matches
                .value_of("dict_label")
                    .unwrap()
                    .parse::<String>()
            {
                params.dict_label = Some(x);
            }
        }

        if sub_matches.is_present("markdown_paths_list") {
            if let Ok(x) = sub_matches
                .value_of("markdown_paths_list")
                    .unwrap()
                    .parse::<String>()
            {
                let list_path = PathBuf::from(&x);
                let s = match fs::read_to_string(&list_path) {
                    Ok(s) => s,
                    Err(e) => {
                        let msg = format!("🔥 Can't read path. {:?}", e);
                        return Err(Box::new(ToolError::Exit(msg)));
                    },
                };
                let s = s.trim();

                let paths: Vec<PathBuf> = s.split('\n').map(PathBuf::from).collect();
                for path in paths.iter() {
                    if !path.exists() {
                        let msg = format!("🔥 Path does not exist: {:?}", &path);
                        return Err(Box::new(ToolError::Exit(msg)));
                    }
                }

                params.markdown_paths = Some(paths);
            }
        }

        match sub_matches.value_of("output_path") {
            Some(x) => params.output_path = Some(ensure_parent(&PathBuf::from(&x))),

            None => {
                let a = params.markdown_paths.as_ref().ok_or("empty paths")?;
                let p = ensure_parent(a.get(0).unwrap());
                let filename = p.file_name().unwrap();
                let dir = p.parent().unwrap();
                match params.ebook_format {
                    EbookFormat::Epub => {
                        let p = dir.join(PathBuf::from(filename).with_extension("epub"));
                        params.output_path = Some(ensure_parent(&p));
                    },
                    EbookFormat::Mobi => {
                        let p = dir.join(PathBuf::from(filename).with_extension("mobi"));
                        params.output_path = Some(ensure_parent(&p));
                    },
                }
            }
        }

        if sub_matches.is_present("mobi_compression") {
            if let Ok(x) = sub_matches
                .value_of("mobi_compression")
                    .unwrap()
                    .parse::<usize>()
            {
                params.mobi_compression = x;
            }
        }

        if sub_matches.is_present("dont_run_kindlegen") {
            params.dont_run_kindlegen = true;
        } else {
            // Only checking when we will need to run KindleGen.

            if sub_matches.is_present("kindlegen_path") {
                if let Ok(x) = sub_matches
                    .value_of("kindlegen_path")
                        .unwrap()
                        .parse::<String>()
                {
                    let path = PathBuf::from(&x);
                    if path.exists() {
                        params.kindlegen_path = Some(path);
                    } else {
                        let msg = format!("🔥 Path does not exist: {:?}", &path);
                        return Err(Box::new(ToolError::Exit(msg)));
                    }
                }
            } else {
                params.kindlegen_path = look_for_kindlegen();
            }
        }

        if sub_matches.is_present("dont_remove_generated_files") {
            params.dont_remove_generated_files = true;
        }

        if sub_matches.is_present("zip_with_lib") {
            params.zip_with = ZipWith::ZipLib;
        }

        if sub_matches.is_present("zip_with_cli") {
            params.zip_with = ZipWith::ZipCli;
        }

        params.run_command = RunCommand::MarkdownToEbook;
    }

    if matches.is_present("show_logs") {
        params.show_logs = true;
    }

    params.markdown_paths = if let Some(paths) = params.markdown_paths {
        Some(ensure_parent_all(&paths))
    } else {
        None
    };

    Ok(params)
}

pub fn process_suttacentral_json(
    json_path: &Option<PathBuf>,
    dict_label: &Option<String>,
    ebook: &mut Ebook,
) {
    let json_path = &json_path.as_ref().expect("json_path is missing.");
    let dict_label = &dict_label.as_ref().expect("dict_label is missing.");

    info! {"=== Begin processing {:?} ===", json_path};

    #[derive(Deserialize)]
    struct Entry {
        word: String,
        text: String,
    }

    let s = fs::read_to_string(json_path).unwrap();
    let entries: Vec<Entry> = serde_json::from_str(&s).unwrap();

    for e in entries.iter() {
        let new_word = DictWord {
            word_header: DictWordHeader {
                dict_label: dict_label.to_string(),
                word: e.word.to_lowercase(),
                summary: "".to_string(),
                grammar: "".to_string(),
                inflections: Vec::new(),
            },
            definition_md: html_to_markdown(&e.text),
        };

        ebook.add_word(new_word)
    }
}

pub fn process_nyanatiloka_entries(
    nyanatiloka_root: &Option<PathBuf>,
    dict_label: &Option<String>,
    ebook: &mut Ebook,
) {
    let nyanatiloka_root = &nyanatiloka_root.as_ref().expect("nyanatiloka_root is missing.");
    let dict_label = &dict_label.as_ref().expect("dict_label is missing.");

    info!{"=== Begin processing {:?} ===", nyanatiloka_root};

    #[derive(Deserialize)]
    struct Entry {
        word: String,
        text: String,
    }

    let mut entries: Vec<Entry> = vec![];

    let folder = nyanatiloka_root.join(Path::new("html_entries"));

    info!("Walking '{:?}'", folder);
    let walker = WalkDir::new(&folder).into_iter();
    for entry in walker.filter_entry(|e| !is_hidden(e)) {
        let entry = entry.unwrap();
        let entry_path = entry.path().to_str().unwrap();
        let entry_file_name = entry.file_name().to_str().unwrap();

        if entry.path().is_dir() {
            info!("Skipping dir entry '{}'", entry.path().to_str().unwrap());
            continue;
        }
        if !entry_file_name.ends_with(".html") {
            continue;
        }

        //info!("Processing: {}", entry_path);

        let text = fs::read_to_string(entry_path).unwrap();

        let mut word = String::new();

        let re = Regex::new("^term-(.+)\\.html").unwrap();
        for cap in re.captures_iter(entry_file_name) {
            word = cap[1].to_string();
        }

        entries.push(Entry {
            word,
            text,
        });
    }

    for e in entries.iter() {
        let new_word = DictWord {
            word_header: DictWordHeader {
                dict_label: dict_label.to_string(),
                word: e.word.to_lowercase(),
                summary: "".to_string(),
                grammar: "".to_string(),
                inflections: Vec::new(),
            },
            definition_md: html_to_markdown(&e.text),
        };

        ebook.add_word(new_word)
    }
}


pub fn process_markdown_list(
    markdown_paths: Vec<PathBuf>,
    ebook: &mut Ebook
) -> Result<(), Box<dyn Error>>
{
    for p in markdown_paths.iter() {
        process_markdown(p, ebook)?;
    }

    Ok(())
}

pub fn process_markdown(
    markdown_path: &PathBuf,
    ebook: &mut Ebook
) -> Result<(), Box<dyn Error>>
{
    info! {"=== Begin processing {:?} ===", markdown_path};

    let s = fs::read_to_string(markdown_path).unwrap();

    // Split the Dictionary header and the DictWord entries.
    let parts: Vec<&str> = s.split(DICTIONARY_WORD_ENTRIES_SEP).collect();

    if parts.len() != 2 {
        let msg = "Bad Markdown input. Can't separate the Dictionary header and DictWord entries.".to_string();
        return Err(Box::new(ToolError::Exit(msg)));
    }

    let a = parts.get(0).unwrap().to_string()
        .replace(DICTIONARY_METADATA_SEP, "")
        .replace("``` toml", "")
        .replace("```", "");

    let mut meta: EbookMetadata = match toml::from_str(&a) {
        Ok(x) => x,
        Err(e) => {
            let msg = format!("🔥 Can't serialize from TOML String: {:?}\nError: {:?}", &a, e);
            return Err(Box::new(ToolError::Exit(msg)));
        }
    };
    meta.created_date_human = Utc::now().to_rfc2822(); // Fri, 28 Nov 2014 12:00:09 +0000
    meta.created_date_opf = Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true);

    match ebook.ebook_format {
        EbookFormat::Epub => {
            meta.is_epub = true;
            meta.is_mobi = false;
        },
        EbookFormat::Mobi => {
            meta.is_epub = false;
            meta.is_mobi = true;
        }
    }

    ebook.meta = meta;

    let a = parts.get(1).unwrap().to_string();
    let entries: Vec<Result<DictWord, Box<dyn Error>>> = a
        .split("``` toml")
        .filter_map(|s| {
            let a = s.trim();
            if !a.is_empty() {
                Some(DictWord::from_markdown(a))
            } else {
                None
            }
        })
        .collect();

    for i in entries.iter() {
        match i {
            Ok(x) => ebook.add_word(x.clone()),
            Err(e) => {
                let msg = format!("{:?}", e);
                return Err(Box::new(ToolError::Exit(msg)));
            },
        }
    }

    Ok(())
}

fn html_to_markdown(html: &str) -> String {
    html2md::parse_html(html)
}

/*
fn html_to_plain(html: &str) -> String {
    let mut plain = html2text::from_read(html.as_bytes(), 100);

    // strip markdown # and > from plain text content

    // at the beginning of the text
    plain = Regex::new("^[#> ]+").unwrap().replace_all(&plain, "").to_string();

    // in the middle of the text
    plain = Regex::new("\n[#> ]+").unwrap().replace_all(&plain, "\n").to_string();

    plain
}
*/

