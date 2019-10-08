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
