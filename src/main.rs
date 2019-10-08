extern crate regex;
extern crate walkdir;

extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate toml;

extern crate zip;
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

use clap::App;

pub mod app;
pub mod ebook;
pub mod dict_word;
pub mod letter_groups;
pub mod pali;
pub mod helpers;

use app::RunCommand;
use ebook::{Ebook, EbookFormat};

fn main() {
    std::env::set_var("RUST_LOG", "error");
    match kankyo::init() {
        Ok(_) => {}
        Err(e) => info!("Couldn't find a .env file: {:?}", e),
    }

    env_logger::init();
    info!("🚀 Launched");

    // --- CLI options ---

    let app_params = if let Some(params) = app::process_first_arg() {
        params
    } else {
        let cli_yaml = load_yaml!("cli.yml");
        let matches = App::from_yaml(cli_yaml).get_matches();
        app::process_cli_args(matches).unwrap()
    };

    if app_params.show_logs {
        std::env::set_var("RUST_LOG", "simsapa_dictionary=info");
    }

    info!("Subcommand given: {:?}", app_params.run_command);

    match app_params.run_command {
        RunCommand::NoOp => {
            println!("No subcommand given. Run with --help for more info.");
        }

        RunCommand::SuttaCentralJsonToMarkdown => {
            let p = &app_params.markdown_paths.expect("markdown_path is missing.");
            let output_markdown_path = p.get(0).unwrap().to_path_buf();

            let mut ebook = Ebook::new(app_params.ebook_format, &output_markdown_path);

            app::process_suttacentral_json(&app_params.json_path, &app_params.dict_label, &mut ebook);

            info!("Added words: {}", ebook.len());

            ebook.write_markdown();
        }

        RunCommand::NyanatilokaToMarkdown => {
            let p = &app_params.markdown_paths.expect("markdown_path is missing.");
            let output_markdown_path = p.get(0).unwrap().to_path_buf();

            let mut ebook = Ebook::new(app_params.ebook_format, &output_markdown_path);

            app::process_nyanatiloka_entries(&app_params.nyanatiloka_root, &app_params.dict_label, &mut ebook);

            info!("Added words: {}", ebook.len());

            ebook.write_markdown();
        }

        RunCommand::MarkdownToEbook => {
            let output_path = &app_params.output_path.expect("output_path is missing.");
            let mut ebook = Ebook::new(app_params.ebook_format, &output_path);

            let p = &app_params.markdown_paths.expect("markdown_paths is missing.");
            let markdown_paths = p.to_vec();

            let kindlegen_path = &app_params.kindlegen_path.expect("kindlegen_path is missing.");

            app::process_markdown_list(markdown_paths, &mut ebook);

            info!("Added words: {}", ebook.len());

            ebook.create_ebook_build_folders();

            match ebook.ebook_format {
                EbookFormat::Epub => {
                    ebook.write_mimetype();
                    ebook.write_meta_inf_files();
                    ebook.write_oebps_files();
                    ebook.zip_files_as_epub(app_params.zip_with);
                }

                EbookFormat::Mobi => {
                    ebook.write_oebps_files();

                    if !app_params.dont_run_kindlegen {
                        ebook.run_kindlegen(&kindlegen_path, app_params.mobi_compression);
                    }
                }
            }

            if !app_params.dont_remove_generated_files {
                ebook.remove_generated_files();
            }
        }
    }

    info!("Finished.");
}
