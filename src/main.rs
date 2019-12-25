#[macro_use]
extern crate lazy_static;
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
use std::path::PathBuf;
use regex::Regex;

use app::{AppStartParams, RunCommand};
use ebook::Ebook;
use helpers::ok_or_exit;

#[allow(clippy::cognitive_complexity)]
fn main() {
    std::env::set_var("RUST_LOG", "error");

    match kankyo::init() {
        Ok(_) => {}
        Err(e) => info!("Couldn't find a .env file: {:?}", e),
    }

    let mut args = std::env::args();
    let _bin_path = args.next();
    if let Some(a) = args.next() {
        if a == "--show_logs" {
            std::env::set_var("RUST_LOG", "info");
        }
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

    info!("Subcommand given: {:?}", app_params.run_command);

    match app_params.run_command {
        RunCommand::NoOp => {
            println!("No subcommand given. Run with --help for more info.");
        }

        RunCommand::SuttaCentralJsonToMarkdown => {
            let (i_p, o_p) = get_input_output(&app_params);
            let mut ebook = Ebook::new(app_params.output_format, app_params.allow_raw_html, &i_p, &o_p);

            if app_params.reuse_metadata {
                ok_or_exit(app_params.used_first_arg, ebook.reuse_metadata());
            }

            ebook.meta.created_date_human = "".to_string();
            ebook.meta.created_date_opf = "".to_string();

            app::process_suttacentral_json(
                &app_params.json_path,
                &app_params.dict_label,
                &mut ebook,
            );

            info!("Added words: {}", ebook.len());

            if !app_params.dont_process {
                ebook.process_tidy();
                ebook.process_also_written_as();
                ebook.process_strip_repeat_word_title();
                ebook.process_grammar_note();
                ebook.process_see_also_from_definition(app_params.dont_remove_see_also);
                ok_or_exit(app_params.used_first_arg, ebook.process_summary());
            }

            ebook.use_cli_overrides(&app_params.clone());

            ok_or_exit(app_params.used_first_arg, ebook.write_markdown());
        }

        RunCommand::NyanatilokaToMarkdown => {
            let (i_p, o_p) = get_input_output(&app_params);
            let mut ebook = Ebook::new(app_params.output_format, app_params.allow_raw_html, &i_p, &o_p);

            if app_params.reuse_metadata {
                ok_or_exit(app_params.used_first_arg, ebook.reuse_metadata());
            }

            ebook.meta.created_date_human = "".to_string();
            ebook.meta.created_date_opf = "".to_string();

            app::process_nyanatiloka_entries(
                &app_params.nyanatiloka_root,
                &app_params.dict_label,
                &mut ebook,
            );

            ebook.process_tidy();
            ok_or_exit(app_params.used_first_arg, ebook.process_summary());

            info!("Added words: {}", ebook.len());

            ebook.use_cli_overrides(&app_params.clone());

            ok_or_exit(app_params.used_first_arg, ebook.write_markdown());
        }

        RunCommand::MarkdownToEbook | RunCommand::XlsxToEbook => {
            let (i_p, o_p) = get_input_output(&app_params);
            let mut ebook = Ebook::new(app_params.output_format, app_params.allow_raw_html, &i_p, &o_p);

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

            ebook.process_text();

            ebook.use_cli_overrides(&app_params.clone());

            ok_or_exit(app_params.used_first_arg, ebook.create_ebook(&app_params));

            if !app_params.dont_remove_generated_files {
                ok_or_exit(app_params.used_first_arg, ebook.remove_generated_files());
            }
        }

        RunCommand::MarkdownToBabylon | RunCommand::XlsxToBabylon => {
            let (i_p, o_p) = get_input_output(&app_params);
            let mut ebook = Ebook::new(app_params.output_format, app_params.allow_raw_html, &i_p, &o_p);

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

            ebook.process_text();

            // Convert /define/word links with bword://word, as recognized by Stardict.
            for (_, w) in ebook.dict_words_input.iter_mut() {
                let re_define = Regex::new(r"\[([^\]]+)\]\(/define/([^\(\)]+)\)").unwrap();
                w.definition_md = re_define.replace_all(&w.definition_md, "[$1](bword://$2)").to_string();
            }

            ebook.use_cli_overrides(&app_params.clone());

            ok_or_exit(app_params.used_first_arg, ebook.create_babylon());
        }

        RunCommand::MarkdownToStardict | RunCommand::XlsxToStardict => {
            let (i_p, o_p) = get_input_output(&app_params);
            let mut ebook = Ebook::new(app_params.output_format, app_params.allow_raw_html, &i_p, &o_p);

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

            ebook.process_text();

            ebook.use_cli_overrides(&app_params.clone());

            ok_or_exit(app_params.used_first_arg, ebook.create_stardict());
        }

        RunCommand::MarkdownToC5 | RunCommand::XlsxToC5 => {
            let (i_p, o_p) = get_input_output(&app_params);
            let mut ebook = Ebook::new(app_params.output_format, app_params.allow_raw_html, &i_p, &o_p);

            let paths = app_params.source_paths.clone();
            let p = paths.expect("source_paths is missing.");
            let source_paths = p.to_vec();

            match app_params.run_command {
                RunCommand::MarkdownToC5 => {
                    ok_or_exit(
                        app_params.used_first_arg,
                        app::process_markdown_list(source_paths, &mut ebook),
                    );
                }

                RunCommand::XlsxToC5 => {
                    ok_or_exit(
                        app_params.used_first_arg,
                        app::process_xlsx_list(source_paths, &mut ebook),
                    );
                }

                _ => {}
            }

            info!("Added words: {}", ebook.len());

            ebook.process_text();

            ebook.use_cli_overrides(&app_params.clone());

            ok_or_exit(app_params.used_first_arg, ebook.create_c5());
        }

        RunCommand::MarkdownToTei | RunCommand::XlsxToTei => {
            let (i_p, o_p) = get_input_output(&app_params);
            let mut ebook = Ebook::new(app_params.output_format, app_params.allow_raw_html, &i_p, &o_p);

            let paths = app_params.source_paths.clone();
            let p = paths.expect("source_paths is missing.");
            let source_paths = p.to_vec();

            match app_params.run_command {
                RunCommand::MarkdownToTei => {
                    ok_or_exit(
                        app_params.used_first_arg,
                        app::process_markdown_list(source_paths, &mut ebook),
                    );
                }

                RunCommand::XlsxToTei => {
                    ok_or_exit(
                        app_params.used_first_arg,
                        app::process_xlsx_list(source_paths, &mut ebook),
                    );
                }

                _ => {}
            }

            info!("Added words: {}", ebook.len());

            ebook.process_text();

            ebook.use_cli_overrides(&app_params.clone());

            ok_or_exit(app_params.used_first_arg, ebook.create_tei());
        }

        RunCommand::MarkdownToJson => {
            let (i_p, o_p) = get_input_output(&app_params);
            let mut ebook = Ebook::new(app_params.output_format, app_params.allow_raw_html, &i_p, &o_p);

            let paths = app_params.source_paths.clone();
            let p = paths.expect("source_paths is missing.");
            let source_paths = p.to_vec();

            ok_or_exit(
                app_params.used_first_arg,
                app::process_markdown_list(source_paths, &mut ebook),
            );

            info!("Added words: {}", ebook.len());

            ok_or_exit(app_params.used_first_arg, ebook.create_json());
        }
    }

    info!("Finished.");
}

fn get_input_output(app_params: &AppStartParams) -> (PathBuf, PathBuf) {
    let i = app_params.clone().source_paths.expect("source_paths is missing");
    let i_p = PathBuf::from(i.get(0).unwrap().parent().unwrap());
    let o_p = app_params.clone().output_path.expect("output_path is missing.");
    (i_p, o_p)
}
