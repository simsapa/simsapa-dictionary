use std::path::PathBuf;
use std::error::Error;
use std::process::exit;
use std::thread::sleep;
use std::time::Duration;

use regex::Regex;
use walkdir::DirEntry;
use handlebars::{Handlebars, RenderContext, Helper, Context, JsonRender, HelperResult, Output};

use comrak::{markdown_to_html, ComrakOptions};

pub fn markdown_helper(h: &Helper, _: &Handlebars, _: &Context, _rc: &mut RenderContext, out: &mut dyn Output) -> HelperResult {
    let param = h.param(0).unwrap();
    let html = md2html(param.value().render().as_ref());
    out.write(&html)?;
    Ok(())
}

pub fn md2html(markdown: &str) -> String {
    let mut opts = ComrakOptions::default();
    opts.smart = true;

    // Remove links until we can resolve them to entry file locations.
    let re = Regex::new(r"\[([^\]]*)\]\([^\)]*\)").unwrap();
    let res = re.replace_all(markdown, "$1").to_string();

    markdown_to_html(&res, &opts)
}

pub fn is_hidden(entry: &DirEntry) -> bool {
    entry.file_name()
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
