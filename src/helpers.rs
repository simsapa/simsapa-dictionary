use std::error::Error;
use std::path::PathBuf;
use std::process::exit;
use std::thread::sleep;
use std::time::Duration;

use handlebars::{Context, Handlebars, Helper, HelperResult, JsonRender, Output, RenderContext};
use regex::Regex;
use walkdir::DirEntry;

use comrak::{markdown_to_html, ComrakOptions};

use crate::pali;

pub fn markdown_helper(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _rc: &mut RenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    let param = h.param(0).unwrap();
    let html = md2html(param.value().render().as_ref());
    out.write(&html)?;
    Ok(())
}

pub fn to_velthuis(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _rc: &mut RenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    let param = h.param(0).unwrap();
    let word = pali::to_velthuis(param.value().render().as_ref());
    out.write(&word)?;
    Ok(())
}

pub fn word_list(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _rc: &mut RenderContext,
    out: &mut dyn Output,
) -> HelperResult {

    let prefix = h.param(0).unwrap().value().render();
    let items = h.param(1).unwrap().value();

    let items_content = if let Some(items) = items.as_array() {
        if !items.is_empty() {
            items.iter()
                .map(|i| format!("<a href=\"bword://{}\">{}</a>", i.render(), i.render()))
                .collect::<Vec<String>>()
                .join(", ")
        } else {
            return Ok(());
        }
    } else {
        return Ok(());
    };

    let content = format!("<p>{} {}</p>", &prefix, &items_content);
    out.write(&content)?;
    Ok(())
}

pub fn grammar_and_phonetic(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _rc: &mut RenderContext,
    out: &mut dyn Output,
) -> HelperResult {

    let grammar = h.param(0).unwrap().value().render();
    let phonetic = h.param(1).unwrap().value().render();

    if grammar.is_empty() && phonetic.is_empty() {
        return Ok(());
    }

    let g = if grammar.is_empty() {
        "".to_string()
    } else {
        format!("<i style=\"color: green;\">{}</i>", grammar)
    };

    let ph = if phonetic.is_empty() {
        "".to_string()
    } else {
        format!(" <span>[{}]</span>", phonetic)
    };

    out.write(&format!("<p>{}{}</p>", g, ph))?;
    Ok(())
}

pub fn md2html(markdown: &str) -> String {
    let mut opts = ComrakOptions::default();
    opts.smart = true;

    // Remove links until we can resolve them to entry file locations.
    let re = Regex::new(r"\[([^\]]*)\]\([^\)]*\)").unwrap();
    let res = re.replace_all(markdown, "$1").to_string();

    markdown_to_html(&res, &opts).trim().to_string()
}

pub fn is_hidden(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| s.starts_with('.'))
        .unwrap_or(false)
}

pub fn ensure_parent(p: &PathBuf) -> PathBuf {
    match p.parent() {
        Some(_) => PathBuf::from(p),
        None => PathBuf::from(".").join(p),
    }
}

/// If the markdown path was given as "ncped.md" (no parent), prefix with "." so that .parent()
/// calls work.
pub fn ensure_parent_all(paths: &[PathBuf]) -> Vec<PathBuf> {
    paths.iter().map(|p| ensure_parent(p)).collect()
}

pub fn ok_or_exit<T>(wait: bool, res: Result<T, Box<dyn Error>>) -> T {
    match res {
        Ok(x) => x,
        Err(e) => {
            error!("{:?}", e);
            if wait {
                println!("Exiting. Waiting 10s, or press Ctrl-C to close ...");
                sleep(Duration::from_secs(10));
            }
            exit(2);
        }
    }
}
