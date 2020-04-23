use std::default::Default;
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use chrono::prelude::*;
use walkdir::WalkDir;
use regex::Regex;
use calamine::{open_workbook, Xlsx, Reader, RangeDeserializerBuilder};

use crate::dict_word::{DictWordMarkdown, DictWordHeader, DictWordXlsx};
use crate::ebook::{
    Ebook, OutputFormat, EbookMetadata, DICTIONARY_METADATA_SEP, DICTIONARY_WORD_ENTRIES_SEP,
};
use crate::error::ToolError;
use crate::helpers::{ensure_parent, ensure_parent_all, is_hidden};

#[derive(Clone)]
pub struct AppStartParams {
    pub output_format: OutputFormat,
    pub json_path: Option<PathBuf>,
    pub metadata_path: Option<PathBuf>,
    pub nyanatiloka_root: Option<PathBuf>,
    pub source_paths: Option<Vec<PathBuf>>,
    pub output_path: Option<PathBuf>,
    pub entries_template: Option<PathBuf>,
    pub mobi_compression: usize,
    pub kindlegen_path: Option<PathBuf>,
    pub reuse_metadata: bool,
    pub title: Option<String>,
    pub dict_label: Option<String>,
    pub cover_path: Option<String>,
    pub word_prefix: Option<String>,
    pub word_prefix_velthuis: bool,
    pub allow_raw_html: bool,
    pub dont_generate_synonyms: bool,
    pub dont_run_kindlegen: bool,
    pub dont_remove_generated_files: bool,
    pub dont_process: bool,
    pub dont_remove_see_also: bool,
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
    XlsxToEbook,
    MarkdownToBabylon,
    XlsxToBabylon,
    MarkdownToStardict,
    XlsxToStardict,
    MarkdownToJson,
    MarkdownToC5,
    XlsxToC5,
    MarkdownToTei,
    XlsxToTei,
    XlsxToJson,
    JsonToXlsx,
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
            output_format: OutputFormat::Epub,
            json_path: None,
            metadata_path: None,
            nyanatiloka_root: None,
            source_paths: None,
            output_path: None,
            entries_template: None,
            kindlegen_path: None,
            reuse_metadata: false,
            title: None,
            dict_label: None,
            cover_path: None,
            word_prefix: None,
            word_prefix_velthuis: false,
            allow_raw_html: false,
            mobi_compression: 0,
            dont_generate_synonyms: false,
            dont_run_kindlegen: false,
            dont_remove_generated_files: false,
            dont_process: false,
            dont_remove_see_also: false,
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
    let source_path = if let Some(a) = args.next() {
        PathBuf::from(a)
    } else {
        return None;
    };

    if !source_path.exists() {
        return None;
    }

    let mut params = AppStartParams::default();
    params.used_first_arg = true;
    params.source_paths = Some(vec![ensure_parent(&source_path)]);

    // Source must be either .md or .xlsx
    let ext = source_path.extension().unwrap();
    if "md" == ext {
        params.run_command = RunCommand::MarkdownToEbook;
    } else if "xlsx" == ext {
        params.run_command = RunCommand::XlsxToEbook;
    } else {
        return None;
    }

    params.kindlegen_path = look_for_kindlegen();

    params.output_format = if params.kindlegen_path.is_some() {
        OutputFormat::Mobi
    } else {
        OutputFormat::Epub
    };

    let filename = source_path.file_name().unwrap();
    let dir = source_path.parent().unwrap();

    let file_ext = if params.kindlegen_path.is_some() {
        "mobi"
    } else {
        "epub"
    };
    params.output_path = Some(dir.join(PathBuf::from(filename).with_extension(file_ext)));

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
            match Command::new("cmd")
                .arg("/C")
                .arg("where kindlegen.exe")
                .output()
            {
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
fn process_to_ebook(
    params: &mut AppStartParams,
    sub_matches: &clap::ArgMatches<'_>,
    run_command: RunCommand)
    -> Result<(), Box<dyn Error>>
{
    if sub_matches.is_present("output_format") {
        if let Ok(x) = sub_matches
            .value_of("output_format")
                .unwrap()
                .parse::<String>()
        {
            let s = x.trim().to_lowercase();
            if s == "epub" {
                params.output_format = OutputFormat::Epub;
            } else if s == "mobi" {
                params.output_format = OutputFormat::Mobi;
            } else {
                params.output_format = OutputFormat::Epub;
            }
        }
    }

    if !sub_matches.is_present("source_path")
        && !sub_matches.is_present("source_paths_list")
    {
        let msg = "🔥 Either 'source_path' or 'source_paths_list' must be used.".to_string();
        return Err(Box::new(ToolError::Exit(msg)));
    }

    if sub_matches.is_present("source_path") {
        if let Ok(x) = sub_matches
            .value_of("source_path")
                .unwrap()
                .parse::<String>()
        {
            let path = PathBuf::from(&x);
            if path.exists() {
                params.source_paths = Some(vec![path]);
            } else {
                let msg = format!("🔥 Path does not exist: {:?}", &path);
                return Err(Box::new(ToolError::Exit(msg)));
            }
        }
    }

    if sub_matches.is_present("title") {
        if let Ok(x) = sub_matches.value_of("title").unwrap().parse::<String>() {
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

    if sub_matches.is_present("cover_path") {
        if let Ok(x) = sub_matches
            .value_of("cover_path")
                .unwrap()
                .parse::<String>()
        {
            params.cover_path = Some(x);
        }
    }

    if sub_matches.is_present("word_prefix") {
        if let Ok(x) = sub_matches
            .value_of("word_prefix")
                .unwrap()
                .parse::<String>()
        {
            params.word_prefix = Some(x);
        }
    }

    if sub_matches.is_present("word_prefix_velthuis") {
        params.word_prefix_velthuis = true;
    }

    if sub_matches.is_present("source_paths_list") {
        if let Ok(x) = sub_matches
            .value_of("source_paths_list")
                .unwrap()
                .parse::<String>()
        {
            let list_path = PathBuf::from(&x);
            let s = match fs::read_to_string(&list_path) {
                Ok(s) => s,
                Err(e) => {
                    let msg = format!("🔥 Can't read path. {:?}", e);
                    return Err(Box::new(ToolError::Exit(msg)));
                }
            };
            let s = s.trim();

            let paths: Vec<PathBuf> = s.split('\n').map(PathBuf::from).collect();
            for path in paths.iter() {
                if !path.exists() {
                    let msg = format!("🔥 Path does not exist: {:?}", &path);
                    return Err(Box::new(ToolError::Exit(msg)));
                }
            }

            params.source_paths = Some(paths);
        }
    }

    match sub_matches.value_of("output_path") {
        Some(x) => params.output_path = Some(ensure_parent(&PathBuf::from(&x))),

        None => {
            let s = params.source_paths.as_ref().unwrap();
            let a = s.get(0).ok_or("can't use source_paths")?;
            let p = ensure_parent(a);
            let filename = p.file_name().unwrap();
            let dir = p.parent().unwrap();
            match params.output_format {
                OutputFormat::Epub => {
                    let p = dir.join(PathBuf::from(filename).with_extension("epub"));
                    params.output_path = Some(ensure_parent(&p));
                }
                OutputFormat::Mobi => {
                    let p = dir.join(PathBuf::from(filename).with_extension("mobi"));
                    params.output_path = Some(ensure_parent(&p));
                }

                _ => {
                    let msg = "🔥 Only Epub or Mobi makes sense here.".to_string();
                    return Err(Box::new(ToolError::Exit(msg)));
                }
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

    if sub_matches.is_present("allow_raw_html") {
        params.allow_raw_html = true;
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

    params.run_command = run_command;

    Ok(())
}

fn process_to_babylon(
    params: &mut AppStartParams,
    sub_matches: &clap::ArgMatches<'_>,
    run_command: RunCommand)
    -> Result<(), Box<dyn Error>>
{
    params.output_format = OutputFormat::BabylonGls;

    if !sub_matches.is_present("source_path")
        && !sub_matches.is_present("source_paths_list")
    {
        let msg = "🔥 Either 'source_path' or 'source_paths_list' must be used.".to_string();
        return Err(Box::new(ToolError::Exit(msg)));
    }

    if sub_matches.is_present("source_path") {
        if let Ok(x) = sub_matches
            .value_of("source_path")
                .unwrap()
                .parse::<String>()
        {
            let path = PathBuf::from(&x);
            if path.exists() {
                params.source_paths = Some(vec![path]);
            } else {
                let msg = format!("🔥 Path does not exist: {:?}", &path);
                return Err(Box::new(ToolError::Exit(msg)));
            }
        }
    }

    if sub_matches.is_present("title") {
        if let Ok(x) = sub_matches.value_of("title").unwrap().parse::<String>() {
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

    if sub_matches.is_present("source_paths_list") {
        if let Ok(x) = sub_matches
            .value_of("source_paths_list")
                .unwrap()
                .parse::<String>()
        {
            let list_path = PathBuf::from(&x);
            let s = match fs::read_to_string(&list_path) {
                Ok(s) => s,
                Err(e) => {
                    let msg = format!("🔥 Can't read path. {:?}", e);
                    return Err(Box::new(ToolError::Exit(msg)));
                }
            };
            let s = s.trim();

            let paths: Vec<PathBuf> = s.split('\n').map(PathBuf::from).collect();
            for path in paths.iter() {
                if !path.exists() {
                    let msg = format!("🔥 Path does not exist: {:?}", &path);
                    return Err(Box::new(ToolError::Exit(msg)));
                }
            }

            params.source_paths = Some(paths);
        }
    }

    let path = match sub_matches.value_of("output_path") {
        Some(x) => ensure_parent(&PathBuf::from(&x)),

        None => {
            let s = params.source_paths.as_ref().unwrap();
            let a = s.get(0).ok_or("can't use source_paths")?;
            let p = ensure_parent(a);
            let filename = p.file_name().unwrap();
            let dir = p.parent().unwrap();

            let p = dir.join(PathBuf::from(filename).with_extension("gls"));
            ensure_parent(&p)
        }
    };

    // The output filename has to be generated with no spaces. Calling Stardict's babylon
    // converter, it passes the name to dictzip without quoting, and so dictzip can't find the
    // file.

    let filename = path.file_name().unwrap().to_str().unwrap().replace(' ', "-");
    params.output_path = Some(path.with_file_name(filename));

    if sub_matches.is_present("allow_raw_html") {
        params.allow_raw_html = true;
    }

    params.run_command = run_command;

    Ok(())
}

fn process_to_stardict(
    params: &mut AppStartParams,
    sub_matches: &clap::ArgMatches<'_>,
    run_command: RunCommand)
    -> Result<(), Box<dyn Error>>
{
    if sub_matches.is_present("keep_entries_plaintext") {
        params.output_format = OutputFormat::StardictXmlPlain;
    } else {
        params.output_format = OutputFormat::StardictXmlHtml;
    }

    if !sub_matches.is_present("source_path")
        && !sub_matches.is_present("source_paths_list")
    {
        let msg = "🔥 Either 'source_path' or 'source_paths_list' must be used.".to_string();
        return Err(Box::new(ToolError::Exit(msg)));
    }

    if sub_matches.is_present("source_path") {
        if let Ok(x) = sub_matches
            .value_of("source_path")
                .unwrap()
                .parse::<String>()
        {
            let path = PathBuf::from(&x);
            if path.exists() {
                params.source_paths = Some(vec![path]);
            } else {
                let msg = format!("🔥 Path does not exist: {:?}", &path);
                return Err(Box::new(ToolError::Exit(msg)));
            }
        }
    }

    if sub_matches.is_present("entries_template") {
        if let Ok(x) = sub_matches
            .value_of("entries_template")
                .unwrap()
                .parse::<String>()
        {
            let path = PathBuf::from(&x);
            if path.exists() {
                params.entries_template = Some(path);
            } else {
                let msg = format!("🔥 Path does not exist: {:?}", &path);
                return Err(Box::new(ToolError::Exit(msg)));
            }
        }
    }

    if sub_matches.is_present("title") {
        if let Ok(x) = sub_matches.value_of("title").unwrap().parse::<String>() {
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

    if sub_matches.is_present("source_paths_list") {
        if let Ok(x) = sub_matches
            .value_of("source_paths_list")
                .unwrap()
                .parse::<String>()
        {
            let list_path = PathBuf::from(&x);
            let s = match fs::read_to_string(&list_path) {
                Ok(s) => s,
                Err(e) => {
                    let msg = format!("🔥 Can't read path. {:?}", e);
                    return Err(Box::new(ToolError::Exit(msg)));
                }
            };
            let s = s.trim();

            let paths: Vec<PathBuf> = s.split('\n').map(PathBuf::from).collect();
            for path in paths.iter() {
                if !path.exists() {
                    let msg = format!("🔥 Path does not exist: {:?}", &path);
                    return Err(Box::new(ToolError::Exit(msg)));
                }
            }

            params.source_paths = Some(paths);
        }
    }

    let path = match sub_matches.value_of("output_path") {
        Some(x) => ensure_parent(&PathBuf::from(&x)),

        None => {
            let s = params.source_paths.as_ref().unwrap();
            let a = s.get(0).ok_or("can't use source_paths")?;
            let p = ensure_parent(a);
            let filename = p.file_name().unwrap();
            let dir = p.parent().unwrap();

            let p = dir.join(PathBuf::from(filename).with_extension("xml"));
            ensure_parent(&p)
        }
    };

    // The output filename has to be generated with no spaces. Calling stardict-text2bin passes the
    // name to dictzip without quoting, and so dictzip can't find the file.

    let filename = path.file_name().unwrap().to_str().unwrap().replace(' ', "-");
    params.output_path = Some(path.with_file_name(filename));

    if sub_matches.is_present("allow_raw_html") {
        params.allow_raw_html = true;
    }

    if sub_matches.is_present("dont_generate_synonyms") {
        params.dont_generate_synonyms = true;
    }

    params.run_command = run_command;

    Ok(())
}

fn process_markdown_to_json(
    params: &mut AppStartParams,
    sub_matches: &clap::ArgMatches<'_>,
    run_command: RunCommand)
    -> Result<(), Box<dyn Error>>
{
    if let Ok(x) = sub_matches
        .value_of("source_path")
            .unwrap()
            .parse::<String>()
    {
        let path = PathBuf::from(&x);
        if path.exists() {
            params.source_paths = Some(vec![path]);
        } else {
            let msg = format!("🔥 Path does not exist: {:?}", &path);
            return Err(Box::new(ToolError::Exit(msg)));
        }
    }

    if let Ok(x) = sub_matches
        .value_of("output_path")
            .unwrap()
            .parse::<String>()
    {
        params.output_path = Some(PathBuf::from(&x));
    }

    params.run_command = run_command;

    Ok(())
}

fn process_xlsx_to_json(
    params: &mut AppStartParams,
    sub_matches: &clap::ArgMatches<'_>,
    run_command: RunCommand)
    -> Result<(), Box<dyn Error>>
{
    if let Ok(x) = sub_matches
        .value_of("source_path")
            .unwrap()
            .parse::<String>()
    {
        let path = PathBuf::from(&x);
        if path.exists() {
            params.source_paths = Some(vec![path]);
        } else {
            let msg = format!("🔥 Path does not exist: {:?}", &path);
            return Err(Box::new(ToolError::Exit(msg)));
        }
    }

    if let Ok(x) = sub_matches
        .value_of("output_path")
            .unwrap()
            .parse::<String>()
    {
        params.output_path = Some(PathBuf::from(&x));
    }

    params.run_command = run_command;

    Ok(())
}

fn process_json_to_xlsx(
    params: &mut AppStartParams,
    sub_matches: &clap::ArgMatches<'_>,
    run_command: RunCommand)
    -> Result<(), Box<dyn Error>>
{
    if let Ok(x) = sub_matches
        .value_of("source_path")
            .unwrap()
            .parse::<String>()
    {
        let path = PathBuf::from(&x);
        if path.exists() {
            params.source_paths = Some(vec![path]);
        } else {
            let msg = format!("🔥 Path does not exist: {:?}", &path);
            return Err(Box::new(ToolError::Exit(msg)));
        }
    }

    if let Ok(x) = sub_matches
        .value_of("metadata_path")
            .unwrap()
            .parse::<String>()
    {
        let path = PathBuf::from(&x);
        if path.exists() {
            params.metadata_path = Some(path);
        } else {
            let msg = format!("🔥 Path does not exist: {:?}", &path);
            return Err(Box::new(ToolError::Exit(msg)));
        }
    }

    if let Ok(x) = sub_matches
        .value_of("output_path")
            .unwrap()
            .parse::<String>()
    {
        params.output_path = Some(PathBuf::from(&x));
    }

    params.run_command = run_command;

    Ok(())
}

fn process_to_c5(
    params: &mut AppStartParams,
    sub_matches: &clap::ArgMatches<'_>,
    run_command: RunCommand)
    -> Result<(), Box<dyn Error>>
{
    if sub_matches.is_present("keep_entries_plaintext") {
        params.output_format = OutputFormat::C5Plain;
    } else {
        params.output_format = OutputFormat::C5Html;
    }

    if !sub_matches.is_present("source_path")
        && !sub_matches.is_present("source_paths_list")
    {
        let msg = "🔥 Either 'source_path' or 'source_paths_list' must be used.".to_string();
        return Err(Box::new(ToolError::Exit(msg)));
    }

    if sub_matches.is_present("source_path") {
        if let Ok(x) = sub_matches
            .value_of("source_path")
                .unwrap()
                .parse::<String>()
        {
            let path = PathBuf::from(&x);
            if path.exists() {
                params.source_paths = Some(vec![path]);
            } else {
                let msg = format!("🔥 Path does not exist: {:?}", &path);
                return Err(Box::new(ToolError::Exit(msg)));
            }
        }
    }

    if sub_matches.is_present("entries_template") {
        if let Ok(x) = sub_matches
            .value_of("entries_template")
                .unwrap()
                .parse::<String>()
        {
            let path = PathBuf::from(&x);
            if path.exists() {
                params.entries_template = Some(path);
            } else {
                let msg = format!("🔥 Path does not exist: {:?}", &path);
                return Err(Box::new(ToolError::Exit(msg)));
            }
        }
    }

    if sub_matches.is_present("title") {
        if let Ok(x) = sub_matches.value_of("title").unwrap().parse::<String>() {
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

    if sub_matches.is_present("source_paths_list") {
        if let Ok(x) = sub_matches
            .value_of("source_paths_list")
                .unwrap()
                .parse::<String>()
        {
            let list_path = PathBuf::from(&x);
            let s = match fs::read_to_string(&list_path) {
                Ok(s) => s,
                Err(e) => {
                    let msg = format!("🔥 Can't read path. {:?}", e);
                    return Err(Box::new(ToolError::Exit(msg)));
                }
            };
            let s = s.trim();

            let paths: Vec<PathBuf> = s.split('\n').map(PathBuf::from).collect();
            for path in paths.iter() {
                if !path.exists() {
                    let msg = format!("🔥 Path does not exist: {:?}", &path);
                    return Err(Box::new(ToolError::Exit(msg)));
                }
            }

            params.source_paths = Some(paths);
        }
    }

    let path = match sub_matches.value_of("output_path") {
        Some(x) => ensure_parent(&PathBuf::from(&x)),

        None => {
            let s = params.source_paths.as_ref().unwrap();
            let a = s.get(0).ok_or("can't use source_paths")?;
            let p = ensure_parent(a);
            let filename = p.file_name().unwrap();
            let dir = p.parent().unwrap();

            let p = dir.join(PathBuf::from(filename).with_extension("xml"));
            ensure_parent(&p)
        }
    };

    // Create the output filename with no spaces to avoid quoting problems when other tools pass on
    // the filename.

    let filename = path.file_name().unwrap().to_str().unwrap().replace(' ', "-");
    params.output_path = Some(path.with_file_name(filename));

    if sub_matches.is_present("allow_raw_html") {
        params.allow_raw_html = true;
    }

    params.run_command = run_command;

    Ok(())
}

fn process_to_tei(
    params: &mut AppStartParams,
    sub_matches: &clap::ArgMatches<'_>,
    run_command: RunCommand)
    -> Result<(), Box<dyn Error>>
{
    if sub_matches.is_present("keep_entries_plaintext") {
        params.output_format = OutputFormat::TeiPlain;
    } else {
        params.output_format = OutputFormat::TeiFormatted;
    }

    if !sub_matches.is_present("source_path")
        && !sub_matches.is_present("source_paths_list")
    {
        let msg = "🔥 Either 'source_path' or 'source_paths_list' must be used.".to_string();
        return Err(Box::new(ToolError::Exit(msg)));
    }

    if sub_matches.is_present("source_path") {
        if let Ok(x) = sub_matches
            .value_of("source_path")
                .unwrap()
                .parse::<String>()
        {
            let path = PathBuf::from(&x);
            if path.exists() {
                params.source_paths = Some(vec![path]);
            } else {
                let msg = format!("🔥 Path does not exist: {:?}", &path);
                return Err(Box::new(ToolError::Exit(msg)));
            }
        }
    }

    if sub_matches.is_present("title") {
        if let Ok(x) = sub_matches.value_of("title").unwrap().parse::<String>() {
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

    if sub_matches.is_present("source_paths_list") {
        if let Ok(x) = sub_matches
            .value_of("source_paths_list")
                .unwrap()
                .parse::<String>()
        {
            let list_path = PathBuf::from(&x);
            let s = match fs::read_to_string(&list_path) {
                Ok(s) => s,
                Err(e) => {
                    let msg = format!("🔥 Can't read path. {:?}", e);
                    return Err(Box::new(ToolError::Exit(msg)));
                }
            };
            let s = s.trim();

            let paths: Vec<PathBuf> = s.split('\n').map(PathBuf::from).collect();
            for path in paths.iter() {
                if !path.exists() {
                    let msg = format!("🔥 Path does not exist: {:?}", &path);
                    return Err(Box::new(ToolError::Exit(msg)));
                }
            }

            params.source_paths = Some(paths);
        }
    }

    let path = match sub_matches.value_of("output_path") {
        Some(x) => ensure_parent(&PathBuf::from(&x)),

        None => {
            let s = params.source_paths.as_ref().unwrap();
            let a = s.get(0).ok_or("can't use source_paths")?;
            let p = ensure_parent(a);
            let filename = p.file_name().unwrap();
            let dir = p.parent().unwrap();

            let p = dir.join(PathBuf::from(filename).with_extension("xml"));
            ensure_parent(&p)
        }
    };

    // Create the output filename with no spaces to avoid quoting problems when other tools pass on
    // the filename.

    let filename = path.file_name().unwrap().to_str().unwrap().replace(' ', "-");
    params.output_path = Some(path.with_file_name(filename));

    // Not accepting --allow_raw_html for TEI.

    params.run_command = run_command;

    Ok(())
}

fn process_suttacentral_json_to_markdown(
    params: &mut AppStartParams,
    sub_matches: &clap::ArgMatches<'_>,
    run_command: RunCommand)
    -> Result<(), Box<dyn Error>>
{
    if let Ok(x) = sub_matches.value_of("json_path").unwrap().parse::<String>() {
        let path = PathBuf::from(&x);
        if path.exists() {
            params.json_path = Some(path);
        } else {
            let msg = format!("🔥 Path does not exist: {:?}", &path);
            return Err(Box::new(ToolError::Exit(msg)));
        }
    }

    if let Ok(x) = sub_matches
        .value_of("output_path")
            .unwrap()
            .parse::<String>()
    {
        params.output_path = Some(PathBuf::from(&x));
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

    if let Ok(x) = sub_matches
        .value_of("dict_label")
            .unwrap()
            .parse::<String>()
    {
        params.dict_label = Some(x);
    }

    if sub_matches.is_present("reuse_metadata") {
        params.reuse_metadata = true;
    }

    if sub_matches.is_present("dont_process") {
        params.dont_process = true;
    }

    if sub_matches.is_present("dont_remove_see_also") {
        params.dont_remove_see_also = true;
    }

    params.run_command = run_command;

    Ok(())
}

fn process_nyanatiloka_to_markdown(
    params: &mut AppStartParams,
    sub_matches: &clap::ArgMatches<'_>,
    run_command: RunCommand)
    -> Result<(), Box<dyn Error>>
{
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
        .value_of("output_path")
            .unwrap()
            .parse::<String>()
    {
        params.output_path = Some(PathBuf::from(&x));
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

    if let Ok(x) = sub_matches
        .value_of("dict_label")
            .unwrap()
            .parse::<String>()
    {
        params.dict_label = Some(x);
    }

    if sub_matches.is_present("reuse_metadata") {
        params.reuse_metadata = true;
    }

    params.run_command = run_command;

    Ok(())
}

pub fn process_cli_args(matches: clap::ArgMatches<'_>) -> Result<AppStartParams, Box<dyn Error>> {
    info!("process_cli_args()");
    let mut params = AppStartParams::default();

    if let Some(sub_matches) = matches.subcommand_matches("suttacentral_json_to_markdown") {
        process_suttacentral_json_to_markdown(&mut params, sub_matches, RunCommand::SuttaCentralJsonToMarkdown)?;

    } else if let Some(sub_matches) = matches.subcommand_matches("nyanatiloka_to_markdown") {
        process_nyanatiloka_to_markdown(&mut params, sub_matches, RunCommand::NyanatilokaToMarkdown)?;

    } else if let Some(sub_matches) = matches.subcommand_matches("markdown_to_json") {
        process_markdown_to_json(&mut params, sub_matches, RunCommand::MarkdownToJson)?;

    } else if let Some(sub_matches) = matches.subcommand_matches("xlsx_to_json") {
        process_xlsx_to_json(&mut params, sub_matches, RunCommand::XlsxToJson)?;

    } else if let Some(sub_matches) = matches.subcommand_matches("json_to_xlsx") {
        process_json_to_xlsx(&mut params, sub_matches, RunCommand::JsonToXlsx)?;

    } else if let Some(sub_matches) = matches.subcommand_matches("markdown_to_ebook") {
        process_to_ebook(&mut params, sub_matches, RunCommand::MarkdownToEbook)?;

    } else if let Some(sub_matches) = matches.subcommand_matches("xlsx_to_ebook") {
        process_to_ebook(&mut params, sub_matches, RunCommand::XlsxToEbook)?;

    } else if let Some(sub_matches) = matches.subcommand_matches("markdown_to_babylon_gls") {
        process_to_babylon(&mut params, sub_matches, RunCommand::MarkdownToBabylon)?;

    } else if let Some(sub_matches) = matches.subcommand_matches("xlsx_to_babylon_gls") {
        process_to_babylon(&mut params, sub_matches, RunCommand::XlsxToBabylon)?;

    } else if let Some(sub_matches) = matches.subcommand_matches("markdown_to_stardict_xml") {
        process_to_stardict(&mut params, sub_matches, RunCommand::MarkdownToStardict)?;

    } else if let Some(sub_matches) = matches.subcommand_matches("xlsx_to_stardict_xml") {
        process_to_stardict(&mut params, sub_matches, RunCommand::XlsxToStardict)?;

    } else if let Some(sub_matches) = matches.subcommand_matches("markdown_to_c5") {
        process_to_c5(&mut params, sub_matches, RunCommand::MarkdownToC5)?;

    } else if let Some(sub_matches) = matches.subcommand_matches("xlsx_to_c5") {
        process_to_c5(&mut params, sub_matches, RunCommand::XlsxToC5)?;

    } else if let Some(sub_matches) = matches.subcommand_matches("markdown_to_tei") {
        process_to_tei(&mut params, sub_matches, RunCommand::MarkdownToTei)?;

    } else if let Some(sub_matches) = matches.subcommand_matches("xlsx_to_tei") {
        process_to_tei(&mut params, sub_matches, RunCommand::XlsxToTei)?;

    }

    if matches.is_present("show_logs") {
        params.show_logs = true;
    }

    params.source_paths = if let Some(paths) = params.source_paths {
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
        let new_word = DictWordMarkdown {
            word_header: DictWordHeader {
                word: e.word.to_lowercase(),
                meaning_order: 1,
                word_nom_sg: "".to_string(),
                is_root: false,
                dict_label: (*dict_label).to_string(),

                inflections: Vec::new(),
                phonetic: "".to_string(),
                transliteration: "".to_string(),

                summary: "".to_string(),

                synonyms: Vec::new(),
                antonyms: Vec::new(),
                homonyms: Vec::new(),
                also_written_as: Vec::new(),
                see_also: Vec::new(),
                comment: "".to_string(),

                grammar_roots: Vec::new(),
                grammar_prefix_and_root: "".to_string(),

                grammar_related_origin_word: "".to_string(),
                grammar_related_origin_roots: Vec::new(),

                grammar_construction: "".to_string(),
                grammar_base_construction: "".to_string(),
                grammar_compound_type: "".to_string(),
                grammar_compound_construction: "".to_string(),

                grammar_comment: "".to_string(),
                grammar_speech: "".to_string(),
                grammar_case: "".to_string(),
                grammar_num: "".to_string(),
                grammar_gender: "".to_string(),
                grammar_person: "".to_string(),
                grammar_voice: "".to_string(),
                grammar_object: "".to_string(),
                grammar_transitive: "".to_string(),
                grammar_negative: "".to_string(),
                grammar_verb: "".to_string(),

                examples: Vec::new(),

                root_language: "".to_string(),
                root_groups: Vec::new(),
                root_sign: "".to_string(),
                root_numbered_group: "".to_string(),

                // ebook.add_word will increment meaning_order if needed
                url_id: DictWordMarkdown::gen_url_id(&e.word, &dict_label, 1),
            },
            definition_md: html_to_markdown(&e.text).to_string(),
        };

        ebook.add_word(new_word)
    }
}

pub fn process_nyanatiloka_entries(
    nyanatiloka_root: &Option<PathBuf>,
    dict_label: &Option<String>,
    ebook: &mut Ebook,
) {
    let nyanatiloka_root = &nyanatiloka_root
        .as_ref()
        .expect("nyanatiloka_root is missing.");
    let dict_label = &dict_label.as_ref().expect("dict_label is missing.");

    info! {"=== Begin processing {:?} ===", nyanatiloka_root};

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

        entries.push(Entry { word, text });
    }

    for e in entries.iter() {
        let new_word = DictWordMarkdown {
            word_header: DictWordHeader {
                word: e.word.to_lowercase(),
                meaning_order: 1,
                word_nom_sg: "".to_string(),
                is_root: false,
                dict_label: (*dict_label).to_string(),

                inflections: Vec::new(),
                phonetic: "".to_string(),
                transliteration: "".to_string(),

                summary: "".to_string(),

                synonyms: Vec::new(),
                antonyms: Vec::new(),
                homonyms: Vec::new(),
                also_written_as: Vec::new(),
                see_also: Vec::new(),
                comment: "".to_string(),

                grammar_roots: Vec::new(),
                grammar_prefix_and_root: "".to_string(),

                grammar_related_origin_word: "".to_string(),
                grammar_related_origin_roots: Vec::new(),

                grammar_construction: "".to_string(),
                grammar_base_construction: "".to_string(),
                grammar_compound_type: "".to_string(),
                grammar_compound_construction: "".to_string(),

                grammar_comment: "".to_string(),
                grammar_speech: "".to_string(),
                grammar_case: "".to_string(),
                grammar_num: "".to_string(),
                grammar_gender: "".to_string(),
                grammar_person: "".to_string(),
                grammar_voice: "".to_string(),
                grammar_object: "".to_string(),
                grammar_transitive: "".to_string(),
                grammar_negative: "".to_string(),
                grammar_verb: "".to_string(),

                examples: Vec::new(),

                root_language: "".to_string(),
                root_groups: Vec::new(),
                root_sign: "".to_string(),
                root_numbered_group: "".to_string(),

                // ebook.add_word will increment meaning_order if needed
                url_id: DictWordMarkdown::gen_url_id(&e.word, &dict_label, 1),
            },
            definition_md: html_to_markdown(&e.text),
        };

        ebook.add_word(new_word)
    }
}

pub fn process_markdown_list(
    source_paths: Vec<PathBuf>,
    ebook: &mut Ebook,
) -> Result<(), Box<dyn Error>> {
    for p in source_paths.iter() {
        process_markdown(p, ebook)?;
    }

    Ok(())
}

pub fn split_metadata_and_entries(path: &PathBuf) -> Result<(String, String), Box<dyn Error>> {
    let s = fs::read_to_string(path).unwrap();

    // Split the Dictionary header and the DictWordMarkdown entries.
    let parts: Vec<&str> = s.split(DICTIONARY_WORD_ENTRIES_SEP).collect();

    if parts.len() != 2 {
        let msg = "Bad Markdown input. Can't separate the Dictionary header and DictWordMarkdown entries."
            .to_string();
        return Err(Box::new(ToolError::Exit(msg)));
    }

    let meta_txt = (*parts
        .get(0)
        .unwrap())
        .to_string()
        .replace(DICTIONARY_METADATA_SEP, "")
        .replace("``` toml", "")
        .replace("```", "");

    let entries_txt = (*parts.get(1).unwrap()).to_string();

    Ok((meta_txt, entries_txt))
}

pub fn parse_str_to_metadata(s: &str) -> Result<EbookMetadata, Box<dyn Error>> {
    let mut meta: EbookMetadata = match toml::from_str(s) {
        Ok(x) => x,
        Err(e) => {
            let msg = format!(
                "🔥 Can't serialize from TOML String: {:?}\nError: {:?}",
                &s, e
            );
            return Err(Box::new(ToolError::Exit(msg)));
        }
    };
    meta.created_date_human = Utc::now().to_rfc2822(); // Fri, 28 Nov 2014 12:00:09 +0000
    meta.created_date_opf = Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true);

    Ok(meta)
}

pub fn process_markdown(source_path: &PathBuf, ebook: &mut Ebook) -> Result<(), Box<dyn Error>> {
    info! {"=== Begin processing {:?} ===", source_path};

    let (meta_txt, entries_txt) = split_metadata_and_entries(&source_path)?;

    ebook.meta = parse_str_to_metadata(&meta_txt)?;

    let entries: Vec<Result<DictWordMarkdown, Box<dyn Error>>> = entries_txt
        .split("``` toml")
        .filter_map(|s| {
            let a = s.trim();
            if !a.is_empty() {
                Some(DictWordMarkdown::from_markdown(a))
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
            }
        }
    }

    Ok(())
}

fn html_to_markdown(html: &str) -> String {
    html2md::parse_html(html)
}

pub fn process_xlsx_list(
    source_paths: Vec<PathBuf>,
    ebook: &mut Ebook,
) -> Result<(), Box<dyn Error>> {
    for p in source_paths.iter() {
        process_xlsx(p, ebook)?;
    }

    Ok(())
}

pub fn process_xlsx(source_path: &PathBuf, ebook: &mut Ebook) -> Result<(), Box<dyn Error>> {
    info! {"=== Begin processing XLSX {:?} ===", source_path};

    let mut workbook: Xlsx<_> = open_workbook(source_path)?;

    let sheet_names = workbook.sheet_names();

    let metadata_name = if sheet_names.contains(&"Metadata".to_string()) {
        "Metadata"
    } else if sheet_names.contains(&"metadata".to_string()) {
        "metadata"
    } else {
        let msg = "Can't find sheet: 'Metadata'".to_string();
        return Err(Box::new(ToolError::Exit(msg)));
    };

    let entries_name = if sheet_names.contains(&"Words".to_string()) {
        "Words"
     } else if sheet_names.contains(&"words".to_string()) {
        "words"
     } else if sheet_names.contains(&"Word entries".to_string()) {
        "Word entries"
    } else if sheet_names.contains(&"Word Entries".to_string()) {
        "Word Entries"
    } else {
        let msg = "Can't find sheet: 'Words'".to_string();
        return Err(Box::new(ToolError::Exit(msg)));
    };

    let roots_name = if sheet_names.contains(&"Roots".to_string()) {
        "Roots"
    } else if sheet_names.contains(&"roots".to_string()) {
        "roots"
    } else {
        let msg = "Can't find sheet: 'Roots'".to_string();
        return Err(Box::new(ToolError::Exit(msg)));
    };

    let metadata_range = workbook.worksheet_range(metadata_name)
        .ok_or_else(|| format!("Can't find sheet: '{}'", &metadata_name))??;
    let entries_range = workbook.worksheet_range(entries_name)
        .ok_or_else(|| format!("Can't find sheet: '{}'", &entries_name))??;
    let roots_range = workbook.worksheet_range(roots_name)
        .ok_or_else(|| format!("Can't find sheet: '{}'", &roots_name))??;

    // Parse Metadata sheet

    {
        let mut iter = RangeDeserializerBuilder::new()
            .has_headers(true)
            .from_range(&metadata_range)?;

        match iter.next() {
            Some(x) => {
                let mut meta: EbookMetadata = x?;
                meta.created_date_human = Utc::now().to_rfc2822(); // Fri, 28 Nov 2014 12:00:09 +0000
                meta.created_date_opf = Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true);

                ebook.meta = meta;
            },
            None => {
                let msg = "Expected at least one row in the Metadata sheet.".to_string();
                return Err(Box::new(ToolError::Exit(msg)));
            }
        }
    }

    // Parse Words sheet

    {
        let iter = RangeDeserializerBuilder::new()
            .has_headers(true)
            .from_range(&entries_range)?;

        let entries: Vec<Result<DictWordXlsx, String>> = iter
            .map(|e| {
                match e {
                    Ok(x) => Ok(x),
                    Err(e) => {
                        Err(format!("Can't parse: {:?}", e))
                    }
                }
            })
        .collect();

        for i in entries.iter() {
            match i {
                Ok(x) => ebook.add_word(DictWordMarkdown::from_xlsx(x)),
                Err(msg) => {
                    println!("{}", msg);
                    // FIXME this segfaults
                    return Err(Box::new(ToolError::Exit(msg.clone())));
                }
            }
        }
    }

    // Parse Roots sheet
    // Same as 'Words' but we set 'is_root' to true.

    {
        let iter = RangeDeserializerBuilder::new()
            .has_headers(true)
            .from_range(&roots_range)?;

        let entries: Vec<Result<DictWordXlsx, String>> = iter
            .map(|e| {
                match e {
                    Ok(x) => Ok(x),
                    Err(e) => {
                        Err(format!("Can't parse: {:?}", e))
                    }
                }
            })
        .collect();

        for i in entries.iter() {
            match i {
                Ok(x) => {
                    let mut a: DictWordXlsx = x.clone();
                    a.is_root = true;
                    ebook.add_word(DictWordMarkdown::from_xlsx(&a))
                },
                Err(msg) => {
                    return Err(Box::new(ToolError::Exit(msg.clone())));
                }
            }
        }
    }

    Ok(())
}

pub fn process_json_list(
    source_paths: Vec<PathBuf>,
    metadata_path: PathBuf,
    ebook: &mut Ebook,
) -> Result<(), Box<dyn Error>> {
    for p in source_paths.iter() {
        process_json_entries(p, ebook)?;
    }
    process_json_metadata(&metadata_path, ebook)?;

    Ok(())
}

pub fn process_json_entries(source_path: &PathBuf, ebook: &mut Ebook) -> Result<(), Box<dyn Error>> {
    info! {"=== Begin processing {:?} ===", source_path};

    let s = fs::read_to_string(source_path).unwrap();
    let entries: Vec<DictWordXlsx> = serde_json::from_str(&s).unwrap();

    for i in entries.iter() {
        ebook.add_word(DictWordMarkdown::from_xlsx(i));
    }

    Ok(())
}

pub fn process_json_metadata(metadata_path: &PathBuf, ebook: &mut Ebook) -> Result<(), Box<dyn Error>> {
    info! {"=== Processing Metadata {:?} ===", metadata_path};

    let s = fs::read_to_string(metadata_path).unwrap();
    let meta: EbookMetadata = serde_json::from_str(&s).unwrap();
    ebook.meta = meta;

    Ok(())
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
