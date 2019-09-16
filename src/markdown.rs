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

    markdown_to_html(markdown, &opts)
}

