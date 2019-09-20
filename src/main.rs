extern crate regex;
extern crate walkdir;

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
extern crate chrono;

extern crate handlebars;
extern crate comrak;

use std::fs;

use clap::App;

pub mod app;
pub mod ebook;
pub mod dict_word;
pub mod letter_groups;
pub mod pali;
pub mod helpers;

use app::RunCommand;
use ebook::Ebook;

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
            let mut ebook = Ebook::new();

            let p = &app_params.markdown_paths.expect("markdown_path is missing.");
            let markdown_path = p.get(0).unwrap().to_path_buf();

            app::process_suttacentral_json(&app_params.json_path, &app_params.dict_label, &mut ebook);

            info!("Added words: {}", ebook.len());

            ebook.write_markdown(&markdown_path);
        }

        RunCommand::NyanatilokaToMarkdown => {
            let mut ebook = Ebook::new();

            let p = &app_params.markdown_paths.expect("markdown_path is missing.");
            let markdown_path = p.get(0).unwrap().to_path_buf();

            app::process_nyanatiloka_entries(&app_params.nyanatiloka_root, &app_params.dict_label, &mut ebook);

            info!("Added words: {}", ebook.len());

            ebook.write_markdown(&markdown_path);
        }

        RunCommand::MarkdownToMobi => {
            let mut ebook = Ebook::new();

            let p = &app_params.markdown_paths.expect("markdown_paths is missing.");
            let markdown_paths = p.to_vec();

            let mobi_path = &app_params.mobi_path.expect("mobi_path is missing.");
            let kindlegen_path = &app_params.kindlegen_path.expect("kindlegen_path is missing.");

            app::process_markdown_list(markdown_paths, &mut ebook);

            info!("Added words: {}", ebook.len());

            let oepbs_dir = mobi_path.parent().unwrap().join("OEPBS");

            if !oepbs_dir.exists() {
                fs::create_dir(&oepbs_dir).unwrap();
            }

            ebook.write_oepbs_files(&oepbs_dir);

            if !app_params.dont_run_kindlegen {
                app::run_kindlegen(
                    &kindlegen_path,
                    &mobi_path,
                    app_params.mobi_compression,
                    &oepbs_dir);
            }

            if !app_params.dont_remove_generated_files {
                fs::remove_dir_all(&oepbs_dir).unwrap();
            }
        }
    }

    info!("Finished.");
}
