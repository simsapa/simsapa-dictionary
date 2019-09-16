extern crate regex;

extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate toml;

extern crate html2md;

#[macro_use]
extern crate log;
extern crate env_logger;
extern crate kankyo;
#[macro_use]
extern crate clap;

extern crate handlebars;
extern crate comrak;

use clap::App;
use std::fs;

pub mod app;
pub mod dictionary;
pub mod markdown;

use app::RunCommand;
use dictionary::Dictionary;

fn main() {
    std::env::set_var("RUST_LOG", "error");
    match kankyo::init() {
        Ok(_) => {}
        Err(e) => info!("Couldn't find a .env file: {:?}", e),
    }

    env_logger::init();
    info!("🚀 Launched");

    // --- CLI options ---

    let cli_yaml = load_yaml!("cli.yml");
    let matches = App::from_yaml(cli_yaml).get_matches();
    let app_params = app::process_cli_args(matches).unwrap();

    if app_params.show_logs {
        std::env::set_var("RUST_LOG", "simsapa_dictionary=info");
    }

    info!("Subcommand given: {:?}", app_params.run_command);

    match app_params.run_command {
        RunCommand::NoOp => {
            println!("No subcommand given. Run with --help for more info.");
        }

        RunCommand::SuttaCentralJsonToMarkdown => {
            let mut dictionary = Dictionary::new();

            let json_path = if let Some(p) = &app_params.json_path {
                p
            } else {
                panic!("json_path is missing.");
            };

            let markdown_path = if let Some(p) = &app_params.markdown_path {
                p
            } else {
                panic!("markdown_path is missing.");
            };

            let dict_label = if let Some(s) = &app_params.dict_label {
                s
            } else {
                panic!("dict_label is missing.");
            };

            dictionary.data.dict_header.dict_label = dict_label.to_string();
            app::process_suttacentral_json(&json_path, &mut dictionary);

            info!("Added words: {}", dictionary.len());

            dictionary.write_markdown(&markdown_path);
        }

        RunCommand::MarkdownToMobi => {
            let mut dictionary = Dictionary::new();

            let mobi_path = if let Some(s) = &app_params.mobi_path {
                s
            } else {
                panic!("mobi_path is missing.");
            };

            let markdown_path = if let Some(s) = &app_params.markdown_path {
                s
            } else {
                panic!("markdown_path is missing.");
            };

            app::process_markdown(&markdown_path, &mut dictionary);

            info!("Added words: {}", dictionary.len());

            let oepbs_dir = mobi_path.parent().unwrap().join("OEPBS");

            if !oepbs_dir.exists() {
                fs::create_dir(&oepbs_dir).unwrap();
            }

            dictionary.write_oepbs_files(&oepbs_dir);

            if !app_params.dont_run_kindlegen {
                let kindlegen_path = if let Some(s) = &app_params.kindlegen_path {
                    s
                } else {
                    panic!("kindlegen_path is missing.");
                };

                app::run_kindlegen(&kindlegen_path, &mobi_path, &oepbs_dir);
            }

            if !app_params.dont_remove_generated_files {
                fs::remove_dir_all(&oepbs_dir).unwrap();
            }
        }
    }

    info!("Finished.");
}
