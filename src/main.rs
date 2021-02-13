#![feature(map_first_last)]

#[macro_use]
extern crate lazy_static;
extern crate regex;
extern crate walkdir;

#[macro_use]
extern crate diesel;

extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate toml;
extern crate calamine;
extern crate xlsxwriter;

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
pub mod dictionary;
pub mod dict_word;
pub mod error;
pub mod helpers;
pub mod letter_groups;
pub mod pali;
pub mod sc_data;
pub mod db_models;
pub mod db_schema;

use std::process::exit;
use std::path::PathBuf;
use regex::Regex;

use app::{AppStartParams, RunCommand};
use dictionary::Dictionary;
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
            let mut dict = Dictionary::new(
                app_params.output_format,
                app_params.allow_raw_html,
                &i_p,
                &o_p,
                app_params.entries_template.clone());

            if app_params.reuse_metadata {
                ok_or_exit(app_params.used_first_arg, dict.reuse_metadata());
            }

            dict.meta.created_date_human = "".to_string();
            dict.meta.created_date_opf = "".to_string();

            let s = app_params.clone().source_paths.expect("source_paths is missing");
            let s_p = PathBuf::from(s.get(0).unwrap());

            app::process_suttacentral_json(
                &s_p,
                &app_params.dict_label,
                &mut dict,
            );

            info!("Added words: {}", dict.len());

            dict.use_cli_overrides(&app_params);

            if !app_params.dont_process {
                dict.process_tidy();
                dict.process_also_written_as();
                dict.process_strip_repeat_word_title();
                dict.process_grammar_note();
                dict.process_see_also_from_definition(app_params.dont_remove_see_also);
                ok_or_exit(app_params.used_first_arg, dict.process_summary());
            }

            ok_or_exit(app_params.used_first_arg, dict.write_markdown());
        }

        RunCommand::SuttaCentralPoTextsToSqlite => {
            let i = app_params.clone().source_paths.expect("source_paths is missing");
            let i_p = PathBuf::from(i.get(0).unwrap());
            let o_p = app_params.clone().output_path.expect("output_path is missing.");
            let json_path = app_params.clone().po_text_json_path.expect("po_text_json_path is missing.");

            ok_or_exit(
                app_params.used_first_arg,
                sc_data::process_suttacentral_root_po_texts_to_sqlite(
                    &i_p,
                    &json_path,
                    &o_p,
                ));
        }

        RunCommand::SuttaCentralHtmlTextsToSqlite => {
            let i = app_params.clone().source_paths.expect("source_paths is missing");
            let i_p = PathBuf::from(i.get(0).unwrap());
            let o_p = app_params.clone().output_path.expect("output_path is missing.");

            ok_or_exit(
                app_params.used_first_arg,
                sc_data::process_suttacentral_html_texts_to_sqlite(
                    &i_p,
                    &o_p,
                ));
        }

        RunCommand::NyanatilokaToMarkdown => {
            let (i_p, o_p) = get_input_output(&app_params);
            let mut dict = Dictionary::new(
                app_params.output_format,
                app_params.allow_raw_html,
                &i_p,
                &o_p,
                app_params.entries_template.clone());

            if app_params.reuse_metadata {
                ok_or_exit(app_params.used_first_arg, dict.reuse_metadata());
            }

            dict.meta.created_date_human = "".to_string();
            dict.meta.created_date_opf = "".to_string();

            app::process_nyanatiloka_entries(
                &app_params.nyanatiloka_root,
                &app_params.dict_label,
                &mut dict,
            );

            dict.use_cli_overrides(&app_params);

            dict.process_tidy();
            ok_or_exit(app_params.used_first_arg, dict.process_summary());

            info!("Added words: {}", dict.len());

            ok_or_exit(app_params.used_first_arg, dict.write_markdown());
        }

        RunCommand::MarkdownToEbook | RunCommand::MarkdownToSqlite | RunCommand::XlsxToEbook | RunCommand::XlsxToRenderJson | RunCommand::XlsxToSqlite => {
            let (i_p, o_p) = get_input_output(&app_params);
            let mut dict = Dictionary::new(
                app_params.output_format,
                app_params.allow_raw_html,
                &i_p,
                &o_p,
                app_params.entries_template.clone());

            let paths = app_params.source_paths.clone();
            let p = paths.expect("source_paths is missing.");
            let source_paths = p.to_vec();

            match app_params.run_command {
                RunCommand::MarkdownToEbook | RunCommand::MarkdownToSqlite => {
                    ok_or_exit(
                        app_params.used_first_arg,
                        app::process_markdown_list(source_paths, &mut dict),
                    );
                }

                RunCommand::XlsxToEbook | RunCommand::XlsxToRenderJson | RunCommand::XlsxToSqlite => {
                    ok_or_exit(
                        app_params.used_first_arg,
                        app::process_xlsx_list(source_paths, &mut dict),
                    );
                }

                _ => {},
            }

            info!("Added words: {}", dict.len());

            dict.use_cli_overrides(&app_params);

            dict.process_text();

            match app_params.run_command {
                RunCommand::MarkdownToEbook | RunCommand::XlsxToEbook => {
                    ok_or_exit(app_params.used_first_arg, dict.create_ebook(&app_params));
                }

                RunCommand::XlsxToRenderJson => {
                    ok_or_exit(app_params.used_first_arg, dict.create_render_json());
                }

                RunCommand::MarkdownToSqlite | RunCommand::XlsxToSqlite => {
                    ok_or_exit(app_params.used_first_arg, dict.insert_to_sqlite(&app_params));
                }

                _ => {},
            }

            if !app_params.dont_remove_generated_files {
                ok_or_exit(app_params.used_first_arg, dict.remove_generated_files());
            }
        }

        RunCommand::MarkdownToBabylon | RunCommand::XlsxToBabylon => {
            let (i_p, o_p) = get_input_output(&app_params);
            let mut dict = Dictionary::new(
                app_params.output_format,
                app_params.allow_raw_html,
                &i_p,
                &o_p,
                app_params.entries_template.clone());

            let paths = app_params.source_paths.clone();
            let p = paths.expect("source_paths is missing.");
            let source_paths = p.to_vec();

            match app_params.run_command {
                RunCommand::MarkdownToBabylon => {
                    ok_or_exit(
                        app_params.used_first_arg,
                        app::process_markdown_list(source_paths, &mut dict),
                    );
                }

                RunCommand::XlsxToBabylon => {
                    ok_or_exit(
                        app_params.used_first_arg,
                        app::process_xlsx_list(source_paths, &mut dict),
                    );
                }

                _ => {}
            }

            info!("Added words: {}", dict.len());

            dict.use_cli_overrides(&app_params);

            dict.process_text();

            // Convert /define/word links with bword://word, as recognized by Stardict.
            for (_, w) in dict.dict_words_input.iter_mut() {
                let re_define = Regex::new(r"\[([^\]]+)\]\(/define/([^\(\)]+)\)").unwrap();
                w.definition_md = re_define.replace_all(&w.definition_md, "[$1](bword://$2)").to_string();
            }

            ok_or_exit(app_params.used_first_arg, dict.create_babylon());
        }

        RunCommand::MarkdownToStardict | RunCommand::XlsxToStardict => {
            let (i_p, o_p) = get_input_output(&app_params);
            let mut dict = Dictionary::new(
                app_params.output_format,
                app_params.allow_raw_html,
                &i_p,
                &o_p,
                app_params.entries_template.clone());

            let paths = app_params.source_paths.clone();
            let p = paths.expect("source_paths is missing.");
            let source_paths = p.to_vec();

            match app_params.run_command {
                RunCommand::MarkdownToStardict => {
                    ok_or_exit(
                        app_params.used_first_arg,
                        app::process_markdown_list(source_paths, &mut dict),
                    );
                }

                RunCommand::XlsxToStardict => {
                    ok_or_exit(
                        app_params.used_first_arg,
                        app::process_xlsx_list(source_paths, &mut dict),
                    );
                }

                _ => {}
            }

            info!("Added words: {}", dict.len());

            dict.use_cli_overrides(&app_params);

            dict.process_text();

            ok_or_exit(app_params.used_first_arg, dict.create_stardict());
        }

        RunCommand::MarkdownToC5 | RunCommand::XlsxToC5 => {
            let (i_p, o_p) = get_input_output(&app_params);
            let mut dict = Dictionary::new(
                app_params.output_format,
                app_params.allow_raw_html,
                &i_p,
                &o_p,
                app_params.entries_template.clone());

            let paths = app_params.source_paths.clone();
            let p = paths.expect("source_paths is missing.");
            let source_paths = p.to_vec();

            match app_params.run_command {
                RunCommand::MarkdownToC5 => {
                    ok_or_exit(
                        app_params.used_first_arg,
                        app::process_markdown_list(source_paths, &mut dict),
                    );
                }

                RunCommand::XlsxToC5 => {
                    ok_or_exit(
                        app_params.used_first_arg,
                        app::process_xlsx_list(source_paths, &mut dict),
                    );
                }

                _ => {}
            }

            info!("Added words: {}", dict.len());

            dict.use_cli_overrides(&app_params);

            dict.process_text();

            ok_or_exit(app_params.used_first_arg, dict.create_c5());
        }

        RunCommand::MarkdownToTei | RunCommand::XlsxToTei => {
            let (i_p, o_p) = get_input_output(&app_params);
            let mut dict = Dictionary::new(
                app_params.output_format,
                app_params.allow_raw_html,
                &i_p,
                &o_p,
                app_params.entries_template.clone());

            let paths = app_params.source_paths.clone();
            let p = paths.expect("source_paths is missing.");
            let source_paths = p.to_vec();

            match app_params.run_command {
                RunCommand::MarkdownToTei => {
                    ok_or_exit(
                        app_params.used_first_arg,
                        app::process_markdown_list(source_paths, &mut dict),
                    );
                }

                RunCommand::XlsxToTei => {
                    ok_or_exit(
                        app_params.used_first_arg,
                        app::process_xlsx_list(source_paths, &mut dict),
                    );
                }

                _ => {}
            }

            info!("Added words: {}", dict.len());

            dict.use_cli_overrides(&app_params);

            dict.process_text();

            ok_or_exit(app_params.used_first_arg, dict.create_tei());
        }

        RunCommand::XlsxToLaTeX => {
            let (i_p, o_p) = get_input_output(&app_params);
            let mut dict = Dictionary::new(
                app_params.output_format,
                app_params.allow_raw_html,
                &i_p,
                &o_p,
                app_params.entries_template.clone());

            let paths = app_params.source_paths.clone();
            let p = paths.expect("source_paths is missing.");
            let source_paths = p.to_vec();

            ok_or_exit(
                app_params.used_first_arg,
                app::process_xlsx_list(source_paths, &mut dict),
            );

            info!("Added words: {}", dict.len());

            dict.use_cli_overrides(&app_params);

            dict.process_text();

            ok_or_exit(app_params.used_first_arg, dict.create_latex());
        }

        RunCommand::MarkdownToJson => {
            let (i_p, o_p) = get_input_output(&app_params);
            let mut dict = Dictionary::new(
                app_params.output_format,
                app_params.allow_raw_html,
                &i_p,
                &o_p,
                app_params.entries_template.clone());

            let paths = app_params.source_paths.clone();
            let p = paths.expect("source_paths is missing.");
            let source_paths = p.to_vec();

            ok_or_exit(
                app_params.used_first_arg,
                app::process_markdown_list(source_paths, &mut dict),
            );

            info!("Added words: {}", dict.len());

            dict.use_cli_overrides(&app_params);

            ok_or_exit(app_params.used_first_arg, dict.create_json());
        }

        RunCommand::XlsxToJson => {
            let (i_p, o_p) = get_input_output(&app_params);
            let mut dict = Dictionary::new(
                app_params.output_format,
                app_params.allow_raw_html,
                &i_p,
                &o_p,
                app_params.entries_template.clone());

            let paths = app_params.source_paths.clone();
            let p = paths.expect("source_paths is missing.");
            let source_paths = p.to_vec();

            ok_or_exit(
                app_params.used_first_arg,
                app::process_xlsx_list(source_paths, &mut dict),
            );

            info!("Added words: {}", dict.len());

            dict.use_cli_overrides(&app_params);

            ok_or_exit(app_params.used_first_arg, dict.create_json());
        }

        RunCommand::JsonToXlsx => {
            let (i_p, o_p) = get_input_output(&app_params);
            let mut dict = Dictionary::new(
                app_params.output_format,
                app_params.allow_raw_html,
                &i_p,
                &o_p,
                app_params.entries_template.clone());

            let paths = app_params.source_paths.clone();
            let p = paths.expect("source_paths is missing.");
            let source_paths = p.to_vec();

            let mp = app_params.metadata_path.clone();
            let metadata_path = mp.expect("metadata_path is missing");

            ok_or_exit(
                app_params.used_first_arg,
                app::process_json_list(source_paths, metadata_path, &mut dict),
            );

            info!("Added words: {}", dict.len());

            dict.use_cli_overrides(&app_params);

            ok_or_exit(app_params.used_first_arg, dict.create_xlsx());
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
