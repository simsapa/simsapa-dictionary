extern crate regex;
extern crate walkdir;

extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate toml;
extern crate calamine;

extern crate html2md;
extern crate zip;

#[macro_use]
extern crate log;
extern crate env_logger;
extern crate kankyo;
#[macro_use]
extern crate clap;
extern crate chrono;

extern crate comrak;
extern crate handlebars;
extern crate deunicode;

use clap::App;

pub mod app;
pub mod dict_word;
pub mod ebook;
pub mod error;
pub mod helpers;
pub mod letter_groups;
pub mod pali;

use std::process::exit;
use regex::Regex;

use app::RunCommand;
use ebook::Ebook;
use helpers::ok_or_exit;

#[allow(clippy::cognitive_complexity)]
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
        match app::process_cli_args(matches) {
            Ok(x) => x,
            Err(e) => {
                error!("{:?}", e);
                exit(2);
            }
        }
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
            let p = &app_params
                .output_path
                .expect("output_path is missing.");

            let mut ebook = Ebook::new(app_params.output_format, &p);

            app::process_suttacentral_json(
                &app_params.json_path,
                &app_params.dict_label,
                &mut ebook,
            );

            info!("Added words: {}", ebook.len());

            ebook.process_also_written_as();
            ebook.process_strip_repeat_word_title();
            ebook.process_grammar_note();
            ebook.process_see_also_from_definition();
            ok_or_exit(app_params.used_first_arg, ebook.process_summary());

            ok_or_exit(app_params.used_first_arg, ebook.write_markdown());
        }

        RunCommand::NyanatilokaToMarkdown => {
            let p = &app_params
                .output_path
                .expect("output_path is missing.");

            let mut ebook = Ebook::new(app_params.output_format, &p);

            app::process_nyanatiloka_entries(
                &app_params.nyanatiloka_root,
                &app_params.dict_label,
                &mut ebook,
            );

            ok_or_exit(app_params.used_first_arg, ebook.process_summary());

            info!("Added words: {}", ebook.len());

            ok_or_exit(app_params.used_first_arg, ebook.write_markdown());
        }

        RunCommand::MarkdownToEbook | RunCommand::XlsxToEbook => {
            let o = app_params.output_path.clone();
            let output_path = o.expect("output_path is missing.");
            let mut ebook = Ebook::new(app_params.output_format, &output_path);

            let paths = app_params.source_paths.clone();
            let p = paths.expect("source_paths is missing.");
            let source_paths = p.to_vec();

            match app_params.run_command {
                RunCommand::MarkdownToEbook => {
                    ok_or_exit(
                        app_params.used_first_arg,
                        app::process_markdown_list(source_paths, &mut ebook),
                    );
                }

                RunCommand::XlsxToEbook => {
                    ok_or_exit(
                        app_params.used_first_arg,
                        app::process_xlsx_list(source_paths, &mut ebook),
                    );
                }

                _ => {},
            }

            info!("Added words: {}", ebook.len());

            if let Some(ref title) = app_params.title {
                ebook.meta.title = title.clone();
            }

            if let Some(ref dict_label) = app_params.dict_label {
                for (_key, word) in ebook.dict_words.iter_mut() {
                    word.word_header.dict_label = dict_label.clone();
                }
            }

            ok_or_exit(app_params.used_first_arg, ebook.create_ebook(&app_params));

            if !app_params.dont_remove_generated_files {
                ok_or_exit(app_params.used_first_arg, ebook.remove_generated_files());
            }
        }

        RunCommand::MarkdownToBabylon | RunCommand::XlsxToBabylon => {
            let o = app_params.output_path.clone();
            let output_path = o.expect("output_path is missing.");
            let mut ebook = Ebook::new(app_params.output_format, &output_path);

            let paths = app_params.source_paths.clone();
            let p = paths.expect("source_paths is missing.");
            let source_paths = p.to_vec();

            match app_params.run_command {
                RunCommand::MarkdownToBabylon => {
                    ok_or_exit(
                        app_params.used_first_arg,
                        app::process_markdown_list(source_paths, &mut ebook),
                    );
                }

                RunCommand::XlsxToBabylon => {
                    ok_or_exit(
                        app_params.used_first_arg,
                        app::process_xlsx_list(source_paths, &mut ebook),
                    );
                }

                _ => {}
            }

            info!("Added words: {}", ebook.len());

            // Convert /define/word links with bword://word, as recognized by Stardict.
            for (_, w) in ebook.dict_words.iter_mut() {
                let re_define = Regex::new(r"\[([^\]]+)\]\(/define/([^\(\)]+)\)").unwrap();
                w.definition_md = re_define.replace_all(&w.definition_md, "[$1](bword://$2)").to_string();
            }

            if let Some(ref title) = app_params.title {
                ebook.meta.title = title.clone();
            }

            if let Some(ref dict_label) = app_params.dict_label {
                for (_key, word) in ebook.dict_words.iter_mut() {
                    word.word_header.dict_label = dict_label.clone();
                }
            }

            ok_or_exit(app_params.used_first_arg, ebook.create_babylon());
        }

        RunCommand::MarkdownToStardict | RunCommand::XlsxToStardict => {
            let o = app_params.output_path.clone();
            let output_path = o.expect("output_path is missing.");
            let mut ebook = Ebook::new(app_params.output_format, &output_path);

            let paths = app_params.source_paths.clone();
            let p = paths.expect("source_paths is missing.");
            let source_paths = p.to_vec();

            match app_params.run_command {
                RunCommand::MarkdownToStardict => {
                    ok_or_exit(
                        app_params.used_first_arg,
                        app::process_markdown_list(source_paths, &mut ebook),
                    );
                }

                RunCommand::XlsxToStardict => {
                    ok_or_exit(
                        app_params.used_first_arg,
                        app::process_xlsx_list(source_paths, &mut ebook),
                    );
                }

                _ => {}
            }

            info!("Added words: {}", ebook.len());

            if let Some(ref title) = app_params.title {
                ebook.meta.title = title.clone();
            }

            if let Some(ref dict_label) = app_params.dict_label {
                for (_key, word) in ebook.dict_words.iter_mut() {
                    word.word_header.dict_label = dict_label.clone();
                }
            }

            ok_or_exit(app_params.used_first_arg, ebook.create_stardict());
        }
    }

    info!("Finished.");
}
