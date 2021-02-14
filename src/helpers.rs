use std::error::Error;
use std::path::PathBuf;
use std::process::exit;
use std::thread::sleep;
use std::time::Duration;

use handlebars::{Context, Handlebars, Helper, HelperResult, JsonRender, Output, RenderContext};
use walkdir::DirEntry;

use comrak::{markdown_to_html, ComrakOptions};

use pali_dict_core::pali;

pub fn markdown_helper(
    h: &Helper<'_, '_>,
    _: &Handlebars,
    _: &Context,
    _rc: &mut RenderContext<'_>,
    out: &mut dyn Output,
) -> HelperResult {
    let param = h.param(0).unwrap();
    let allow_raw_html: bool = h.param(1).unwrap().value().as_bool().unwrap();
    let html = md2html(param.value().render().as_ref(), allow_raw_html);
    out.write(&html)?;
    Ok(())
}

pub fn countitems(
    h: &Helper<'_, '_>,
    _: &Handlebars,
    _: &Context,
    _rc: &mut RenderContext<'_>,
    out: &mut dyn Output,
) -> HelperResult {
    let count = if let Some(items) = h.param(0).unwrap().value().as_array() {
        items.len()
    } else if let Some(words) = h.param(0).unwrap().value().as_object() {
        words.len()
    } else {
        0
    };
    out.write(&format!("{}", count))?;
    Ok(())
}

pub fn to_velthuis(
    h: &Helper<'_, '_>,
    _: &Handlebars,
    _: &Context,
    _rc: &mut RenderContext<'_>,
    out: &mut dyn Output,
) -> HelperResult {
    let param = h.param(0).unwrap();
    let word = pali::to_velthuis(param.value().render().as_ref());
    out.write(&word)?;
    Ok(())
}

pub fn word_title(
    h: &Helper<'_, '_>,
    _: &Handlebars,
    _: &Context,
    _rc: &mut RenderContext<'_>,
    out: &mut dyn Output,
) -> HelperResult {
    let word: String = h.param(0).unwrap().value().as_str().unwrap().to_string();
    let meta = h.param(1).unwrap().value().as_object().unwrap();

    let word_prefix: String = meta.get("word_prefix").unwrap().as_str().unwrap().to_string();
    let word_prefix_velthuis: bool = meta.get("word_prefix_velthuis").unwrap().as_bool().unwrap();

    let word_velthuis = pali::to_velthuis(&word);

    let mut text = String::new();

    if !word_prefix.is_empty() {
        text.push_str(&format!("{} ", word_prefix));
    }

    if word_prefix_velthuis && word != word_velthuis {
        text.push_str(&format!("{} - ", word_velthuis));
    }

    text.push_str(&word);

    out.write(&text)?;
    Ok(())
}

pub fn headword_plain(
    h: &Helper<'_, '_>,
    _: &Handlebars,
    _: &Context,
    _rc: &mut RenderContext<'_>,
    out: &mut dyn Output,
) -> HelperResult {
    let word: String = h.param(0).unwrap().value().as_str().unwrap().to_string();
    let inflections = h.param(1).unwrap().value();

    let inflections_content = if let Some(items) = inflections.as_array() {
        if !items.is_empty() {
            items.iter()
                .map(|i| i.render())
                .collect::<Vec<String>>()
                .join("; ")
        } else {
            "".to_string()
        }
    } else {
        "".to_string()
    };

    let mut text = word;

    if !inflections_content.is_empty() {
        text.push_str("; ");
        text.push_str(&inflections_content);
    }

    out.write(&text)?;
    Ok(())
}

pub fn cover_media_type(
    h: &Helper<'_, '_>,
    _: &Handlebars,
    _: &Context,
    _rc: &mut RenderContext<'_>,
    out: &mut dyn Output,
) -> HelperResult {
    let path = h.param(0).unwrap().value().render();
    let media_type = if path.ends_with(".png") {
        "image/png"
    } else {
        "image/jpeg"
    };
    out.write(media_type)?;
    Ok(())
}

pub fn word_list(
    h: &Helper<'_, '_>,
    _: &Handlebars,
    _: &Context,
    _rc: &mut RenderContext<'_>,
    out: &mut dyn Output,
) -> HelperResult {

    let prefix = h.param(0).unwrap().value().render();
    let items = h.param(1).unwrap().value();

    let items_content = if let Some(items) = items.as_array() {
        if !items.is_empty() {
            items.iter()
                .map(|i| i.render())
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

pub fn word_list_plain(
    h: &Helper<'_, '_>,
    _: &Handlebars,
    _: &Context,
    _rc: &mut RenderContext<'_>,
    out: &mut dyn Output,
) -> HelperResult {

    let prefix = h.param(0).unwrap().value().render();
    let items = h.param(1).unwrap().value();

    let items_content = if let Some(items) = items.as_array() {
        if !items.is_empty() {
            items.iter()
                .map(|i| i.render())
                .collect::<Vec<String>>()
                .join(", ")
        } else {
            return Ok(());
        }
    } else {
        return Ok(());
    };

    let content = format!("{} {}", &prefix, &items_content);
    out.write(&content)?;
    Ok(())
}

pub fn word_list_tei(
    h: &Helper<'_, '_>,
    _: &Handlebars,
    _: &Context,
    _rc: &mut RenderContext<'_>,
    out: &mut dyn Output,
) -> HelperResult {

    let prefix = h.param(0).unwrap().value().render();
    let items = h.param(1).unwrap().value();
    let type_attr = h.param(2).unwrap().value().render();

    // <xr type="syn"><ref target="#pynrit">pynrit</ref></xr>

    // The items are already converted to <ref> links at this point
    let items_content = if let Some(items) = items.as_array() {
        if !items.is_empty() {
            items.iter()
                .map(|i| i.render())
                .collect::<Vec<String>>()
                .join(", ")
        } else {
            return Ok(());
        }
    } else {
        return Ok(());
    };

    let content = format!("<xr type=\"{}\">{} {}</xr>", &type_attr, &prefix, &items_content);
    out.write(&content)?;
    Ok(())
}

fn get_grammar_text(grammar: &serde_json::Value) -> String {
    let speech  = grammar.get("speech").unwrap().render();
    let case    = grammar.get("case").unwrap().render();
    let num     = grammar.get("num").unwrap().render();
    let gender  = grammar.get("gender").unwrap().render();
    let person  = grammar.get("person").unwrap().render();
    let voice   = grammar.get("voice").unwrap().render();
    let object  = grammar.get("object").unwrap().render();

    let transitive = grammar.get("transitive").unwrap().render();
    let negative = grammar.get("negative").unwrap().render();
    let verb = grammar.get("verb").unwrap().render();

    let grammar_text = vec![
        speech,
        case,
        num,
        gender,
        person,
        voice,
        object,
        transitive,
        negative,
        verb,
    ].iter().filter_map(|i| {
        let s = i.trim();
        if s.is_empty() {
            None
        } else {
            Some(s)
        }
    }).collect::<Vec<&str>>().join(", ");

    grammar_text.trim().to_string()
}

pub fn format_grammar_text_html(grammar: &serde_json::Value) -> String {
    let grammar_text = get_grammar_text(grammar);

    if grammar_text.is_empty() {
        "".to_string()
    } else {
        // grass green
        format!("<i style=\"color: #448C19;\">{}</i>", grammar_text)
    }
}

pub fn format_grammar_text_plain(grammar: &serde_json::Value) -> String {
    let grammar_text = get_grammar_text(grammar);

    if grammar_text.is_empty() {
        "".to_string()
    } else {
        format!("/{}/", grammar_text)
    }
}

pub fn grammar_text(
    h: &Helper<'_, '_>,
    _: &Handlebars,
    _: &Context,
    _rc: &mut RenderContext<'_>,
    out: &mut dyn Output,
) -> HelperResult {

    let grammar = h.param(0).unwrap().value();

    out.write(&format_grammar_text_html(grammar))?;
    Ok(())
}

pub fn grammar_text_plain(
    h: &Helper<'_, '_>,
    _: &Handlebars,
    _: &Context,
    _rc: &mut RenderContext<'_>,
    out: &mut dyn Output,
) -> HelperResult {

    let grammar = h.param(0).unwrap().value();

    out.write(&format_grammar_text_plain(grammar))?;
    Ok(())
}

pub fn format_phonetic_transliteration_html(dict_word_render: &serde_json::Value, add_velthuis: bool) -> String {

    let word = dict_word_render.get("word").unwrap().render();
    let phonetic = dict_word_render.get("phonetic").unwrap().render();
    let transliteration = dict_word_render.get("transliteration").unwrap().render();

    let mut parts: Vec<String> = Vec::new();

    if !phonetic.is_empty() {
        // dark ocean blue
        parts.push(format!("<span style=\"color: #0B4A72;\">{}</span>", phonetic));
    }

    if transliteration.is_empty() {
        if add_velthuis {
            let velthuis = pali::to_velthuis(&word);
            if word != velthuis {
                // dark ocean blue
                parts.push(format!("<span style=\"color: #0B4A72;\">{}</span>", velthuis));
            }
        }
    } else {
        // dark ocean blue
        parts.push(format!("<span style=\"color: #0B4A72;\">{}</span>", transliteration));
    };

    if parts.is_empty() {
        "".to_string()
    } else {
        format!("<p>{}</p>", parts.join(" | "))
    }
}

pub fn phonetic_transliteration(
    h: &Helper<'_, '_>,
    _: &Handlebars,
    _: &Context,
    _rc: &mut RenderContext<'_>,
    out: &mut dyn Output,
) -> HelperResult {

    let dict_word_render = h.param(0).unwrap().value();
    let add_velthuis = h.param(1).unwrap().value().as_bool().unwrap();

    out.write(&format_phonetic_transliteration_html(dict_word_render, add_velthuis))?;
    Ok(())
}

pub fn format_phonetic_transliteration_plain(dict_word_render: &serde_json::Value, add_velthuis: bool) -> String {

    let word = dict_word_render.get("word").unwrap().render();
    let phonetic = dict_word_render.get("phonetic").unwrap().render();
    let transliteration = dict_word_render.get("transliteration").unwrap().render();

    let mut parts: Vec<String> = Vec::new();

    if !phonetic.is_empty() {
        parts.push(phonetic);
    }

    if transliteration.is_empty() {
        if add_velthuis {
            let velthuis = pali::to_velthuis(&word);
            if word != velthuis {
                parts.push(velthuis);
            }
        }
    } else {
        parts.push(transliteration);
    };

    if parts.is_empty() {
        "".to_string()
    } else {
        parts.join(" | ")
    }
}

pub fn phonetic_transliteration_plain(
    h: &Helper<'_, '_>,
    _: &Handlebars,
    _: &Context,
    _rc: &mut RenderContext<'_>,
    out: &mut dyn Output,
) -> HelperResult {

    let word_header = h.param(0).unwrap().value();
    let add_velthuis = h.param(1).unwrap().value().as_bool().unwrap();

    out.write(&format_phonetic_transliteration_plain(word_header, add_velthuis))?;
    Ok(())
}

pub fn md2html(markdown: &str, allow_raw_html: bool) -> String {
    let mut opts = ComrakOptions::default();
    opts.smart = true;
    opts.unsafe_ = allow_raw_html;
    markdown_to_html(markdown, &opts).trim().to_string()
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

/// https://stackoverflow.com/questions/38406793/why-is-capitalizing-the-first-letter-of-a-string-so-convoluted-in-rust
pub fn uppercase_first_letter(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}

pub fn light_html_escape(data: &str) -> String {
    //lazy_static! {
    //    static ref RE_AMP: Regex = Regex::new(r"&\b").unwrap();
    //}
    //RE_AMP.replace_all(&data, "&amp;").to_string()

    data.replace('&', "&amp;")
}

