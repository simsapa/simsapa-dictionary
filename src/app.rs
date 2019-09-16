use std::default::Default;
use std::error::Error;
use std::fs;
use std::process::{exit, Command};
use std::path::PathBuf;

use crate::dictionary::{DictWord, DictWordHeader, Dictionary, DictHeader};

#[derive(Debug)]
pub struct AppStartParams {
    pub json_path: Option<PathBuf>,
    pub markdown_path: Option<PathBuf>,
    pub mobi_path: Option<PathBuf>,
    pub kindlegen_path: Option<PathBuf>,
    pub dict_label: Option<String>,
    pub dont_run_kindlegen: bool,
    pub dont_remove_generated_files: bool,
    pub run_command: RunCommand,
    pub show_logs: bool,
}

#[derive(Clone, Copy, Debug)]
pub enum RunCommand {
    NoOp,
    SuttaCentralJsonToMarkdown,
    MarkdownToMobi,
}

impl Default for AppStartParams {
    fn default() -> Self {
        AppStartParams {
            json_path: None,
            markdown_path: None,
            mobi_path: None,
            kindlegen_path: None,
            dict_label: None,
            dont_run_kindlegen: false,
            dont_remove_generated_files: false,
            run_command: RunCommand::NoOp,
            show_logs: false,
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
                error!("🔥 Path does not exist: {:?}", &path);
                exit(2);
            }
        }

        if let Ok(x) = sub_matches
            .value_of("markdown_path")
            .unwrap()
            .parse::<String>()
        {
            params.markdown_path = Some(PathBuf::from(&x));
        }

        if let Ok(x) = sub_matches
            .value_of("dict_label")
            .unwrap()
            .parse::<String>()
        {
            params.dict_label = Some(x);
        }

        params.run_command = RunCommand::SuttaCentralJsonToMarkdown;

    } else if let Some(sub_matches) = matches.subcommand_matches("markdown_to_mobi") {

        if let Ok(x) = sub_matches
            .value_of("markdown_path")
            .unwrap()
            .parse::<String>()
        {
            let path = PathBuf::from(&x);
            if path.exists() {
                params.markdown_path = Some(path);
            } else {
                error!("🔥 Path does not exist: {:?}", &path);
                exit(2);
            }
        }

        if let Ok(x) = sub_matches
            .value_of("mobi_path")
            .unwrap()
            .parse::<String>()
        {
            params.mobi_path = Some(PathBuf::from(&x));
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
                        error!("🔥 Path does not exist: {:?}", &path);
                        exit(2);
                    }
                }
            } else {
                // Look for the kindlegen executable. Either in the current working directory, or the system PATH.

                // Try if it is in the local folder.
                let path = if cfg!(target_os = "windows") {
                    PathBuf::from(".").join(PathBuf::from("kindlegen.exe"))
                } else {
                    PathBuf::from(".").join(PathBuf::from("kindlegen"))
                };

                if path.exists() {
                    params.kindlegen_path = Some(path);
                } else {
                    // Try if it is available from the system PATH.

                    let output = if cfg!(target_os = "windows") {
                        match Command::new("cmd").arg("/C").arg("where kindlegen.exe").output() {
                            Ok(o) => o,
                            Err(e) => {
                                error!("🔥 Failed to find KindleGen: {:?}", e);
                                exit(2);
                            }
                        }
                    } else {
                        match Command::new("sh").arg("-c").arg("which kindlegen").output() {
                            Ok(o) => o,
                            Err(e) => {
                                error!("🔥 Failed to find KindleGen: {:?}", e);
                                exit(2);
                            }
                        }
                    };

                    if output.status.success() {
                        let s = String::from_utf8(output.stdout).unwrap();
                        info!("🔎 Found KindleGen in: {}", s);
                        params.kindlegen_path = Some(PathBuf::from(s));
                    } else {
                        error!("🔥 Failed to find KindleGen.");
                        exit(2);
                    }
                }
            }
        }

        if sub_matches.is_present("dont_remove_generated_files") {
            params.dont_remove_generated_files = true;
        }

        params.run_command = RunCommand::MarkdownToMobi;
    }

    if matches.is_present("show_logs") {
        params.show_logs = true;
    }

    Ok(params)
}

pub fn process_suttacentral_json(
    json_path: &PathBuf,
    dictionary: &mut Dictionary,
) {
    info! {"\n=== Begin processing {:?} ===\n", json_path};

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
                word: e.word.clone(),
                summary: "".to_string(),
                grammar: "".to_string(),
            },
            definition_md: html_to_markdown(&e.text),
            definition_html: e.text.clone(),
        };

        dictionary.add(new_word)
    }
}

pub fn process_markdown(
    markdown_path: &PathBuf,
    dictionary: &mut Dictionary
) {
    info! {"\n=== Begin processing {:?} ===\n", markdown_path};

    let s = fs::read_to_string(markdown_path).unwrap();

    // Split the Dictionary header and the DictWord entries.
    let parts: Vec<&str> = s.split("--- DICTIONARY WORD ENTRIES ---").collect();

    if parts.len() != 2 {
        panic!("Something is wrong with the Markdown input. Can't separate the Dictionary header and DictWord entries.");
    }

    let a = parts.get(0).unwrap().to_string()
        .replace("--- DICTIONARY HEADER ---", "")
        .replace("``` toml", "")
        .replace("```", "");

    let header: DictHeader = toml::from_str(&a).unwrap();
    dictionary.data.dict_header = header;

    let a = parts.get(1).unwrap().to_string();
    let entries: Vec<DictWord> = a
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
        dictionary.add(i.clone());
    }
}

pub fn run_kindlegen(kindlegen_path: &PathBuf, mobi_path: &PathBuf, oepbs_dir: &PathBuf) {
    let opf_path = oepbs_dir.join(PathBuf::from("package.opf"));
    let mobi_name = mobi_path.file_name().unwrap().to_str().unwrap();
    // -c2 is quite slow
    let compr_opt = "-c0";

    let k = if cfg!(target_os = "windows") {
        clean_windows_str_path(kindlegen_path.to_str().unwrap())
    } else {
        kindlegen_path.to_str().unwrap()
    };

    let bin_cmd = format!("{} {} {} -dont_append_source -o {}",
        k,
        opf_path.to_str().unwrap(),
        compr_opt,
        mobi_name);

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
        exit(2);
    }

    // Move the generate MOBI to its path. KindleGen puts the MOBI in the same folder with package.opf.
    fs::rename(oepbs_dir.join(mobi_name), mobi_path).unwrap();
}

/*
fn is_hidden(entry: &DirEntry) -> bool {
    entry.file_name()
        .to_str()
        .map(|s| s.starts_with("."))
        .unwrap_or(false)
}
*/

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

pub fn clean_windows_str_path(p: &str) -> &str {
    p.trim_start_matches("\\\\?\\")
}
