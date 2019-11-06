use regex::Regex;
use std::default::Default;
use std::error::Error;

use crate::error::ToolError;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DictWord {
    pub word_header: DictWordHeader,
    pub definition_md: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DictWordXlsx {

    #[serde(default)]
    pub dict_label: String,

    pub word: String,

    #[serde(default)]
    pub summary: String,

    #[serde(default)]
    pub grammar: String,

    /// comma-seperated list
    #[serde(default)]
    pub inflections: String,

    /// comma-seperated list
    #[serde(default)]
    pub synonyms: String,

    /// comma-seperated list
    #[serde(default)]
    pub antonyms: String,

    pub definition_md: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DictWordHeader {
    #[serde(default)]
    pub dict_label: String,

    pub word: String,

    #[serde(default)]
    pub summary: String,
    #[serde(default)]
    pub grammar: String,
    #[serde(default)]
    pub inflections: Vec<String>,
    #[serde(default)]
    pub synonyms: Vec<String>,
    #[serde(default)]
    pub antonyms: Vec<String>,
}

impl DictWord {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn as_markdown_and_toml_string(&self) -> String {
        let header = toml::to_string(&self.word_header).unwrap();
        format!(
            "``` toml\n{}\n```\n\n{}",
            &header.trim(),
            &self.definition_md.trim()
        )
    }

    pub fn from_markdown(s: &str) -> Result<DictWord, Box<dyn Error>> {
        let a = s.replace("``` toml", "");
        let parts: Vec<&str> = a.split("```").collect();

        let toml = parts.get(0).unwrap();
        let word_header: DictWordHeader = match toml::from_str(toml) {
            Ok(x) => x,
            Err(e) => {
                let msg = format!(
                    "🔥 Can't serialize from TOML String: {:?}\nError: {:?}",
                    &toml, e
                );
                return Err(Box::new(ToolError::Exit(msg)));
            }
        };

        Ok(DictWord {
            word_header,
            definition_md: parts.get(1).unwrap().to_string(),
        })
    }

    fn parse_csv_list(s: &str) -> Vec<String> {
        let s = s.trim();
        if s.is_empty() {
            Vec::new()
        } else {
            s.split(',').map(|i| i.trim().to_string()).collect()
        }
    }

    pub fn from_xlsx(w: &DictWordXlsx) -> DictWord {
        DictWord {
            word_header: DictWordHeader {
                dict_label: w.dict_label.clone(),
                word: w.word.clone(),
                summary: w.summary.clone(),
                grammar: w.grammar.clone(),
                inflections: DictWord::parse_csv_list(&w.inflections),
                synonyms: DictWord::parse_csv_list(&w.synonyms),
                antonyms: DictWord::parse_csv_list(&w.antonyms),
            },
            definition_md: w.definition_md.clone(),
        }
    }

    pub fn clean_summary(&mut self) -> Result<(), Box<dyn Error>> {
        if !self.word_header.summary.is_empty() {
            self.word_header.summary = self.word_header.summary.trim().to_string();
        }

        if !self.word_header.summary.is_empty() {
            return Ok(());
        }

        let mut summary = self.definition_md.trim().to_string();

        // newlines to space
        summary = summary.replace("\n", " ");
        // contract multiple spaces
        let re_spaces = Regex::new("  +").unwrap();
        summary = re_spaces.replace_all(&summary, " ").trim().to_string();

        // remaining html tags
        summary = summary.replace("<sup>", "");
        summary = summary.replace("</sup>", "");
        summary = summary.replace("<i>", "");
        summary = summary.replace("</i>", "");
        summary = summary.replace("<b>", "");
        summary = summary.replace("</b>", "");

        let re_chars = Regex::new(r"[\n\t<>]").unwrap();
        summary = re_chars.replace_all(&summary, " ").trim().to_string();

        // slash escapes
        // un\-angered -> un-angered
        // un\\-angered -> un-angered
        summary = summary.replace(r"\\", "");
        summary = summary.replace(r"\", "");

        // See... with markdown link
        // (see *[abbha](/define/abbha)*) -> (see abbha)
        let re_see_markdown_links = Regex::new(r"\(see \*\[([^\]]+)\]\([^\)]+\)\**\)").unwrap();
        summary = re_see_markdown_links
            .replace_all(&summary, "(see $1)")
            .trim()
            .to_string();

        // markdown links
        // [abbha](/define/abbha) -> abbha
        let re_markdown_links = Regex::new(r"\[([^\]]+)\]\([^\)]+\)").unwrap();
        summary = re_markdown_links
            .replace_all(&summary, "$1")
            .trim()
            .to_string();

        // remaining markdown markup: *, []
        let re_markdown = Regex::new(r"[\*\[\]]").unwrap();
        summary = re_markdown.replace_all(&summary, "").trim().to_string();

        // Don't remove (see ...), so that one can look up the next word when noticing it in the
        // search hits.

        // (from|or|also ...)
        let re_from = Regex::new(r"^\((from|or|also) +[^\)]+\)").unwrap();

        // 1
        // 1.
        let re_num = Regex::new(r"^[0-9]\.*").unwrap();

        // grammar abbr., with- or without dot, with- or without parens
        let re_abbr_one = Regex::new(r"^\(*(d|f|m|ṃ|n|r|s|t)\.*\)*\.*\b").unwrap();
        let re_abbr_two = Regex::new(r"^\(*(ac|fn|id|mf|pl|pp|pr|sg|si)\.*\)*\.*\b").unwrap();
        let re_abbr_three = Regex::new(
            r"^\(*(abl|acc|act|adv|aor|dat|fpp|fut|gen|inc|ind|inf|loc|mfn|neg|opt)\.*\)*\.*\b",
        )
        .unwrap();
        let re_abbr_four = Regex::new(r"^\(*(caus|part|pass|pron)\.*\)*\.*\b").unwrap();
        let re_abbr_more = Regex::new(r"^\(*(absol|abstr|accus|compar|desid|feminine|impers|instr|masculine|neuter|plural|singular)\.*\)*\.*\b").unwrap();

        // (~ontī)
        // (-ikā)n.
        let re_suffix = Regex::new(r"^\([~-][^\)]+\)\w*\.*").unwrap();

        // agga-m-agga
        // abhi-uggantvā
        let re_hyphenated_twice = Regex::new(r"^\w+-\w+-\w+\b").unwrap();
        let re_hyphenated_once = Regex::new(r"^\w+-\w+\b").unwrap();

        let max_iter = 10;
        let mut n_iter = 0;

        loop {
            let word = self.word_header.word.clone();
            // the whole word
            //  abhijanat, abhikamin
            let mut s = summary.trim_start_matches(&word).trim().to_string();

            // part of the word
            // abhijana(t)
            // abhikami(n)
            let (char_idx, _char) = word.char_indices().last().unwrap();
            let w = word[..char_idx].to_string();
            s = s.trim_start_matches(&w).trim().to_string();

            s = re_hyphenated_twice.replace(&s, "").trim().to_string();
            s = re_hyphenated_once.replace(&s, "").trim().to_string();

            s = re_num.replace(&s, "").trim().to_string();
            s = re_from.replace(&s, "").trim().to_string();

            s = re_suffix.replace(&s, "").trim().to_string();

            s = s.trim_start_matches('.').trim().to_string();
            s = s.trim_start_matches(',').trim().to_string();
            s = s.trim_start_matches('-').trim().to_string();

            // (?)
            s = s.trim_start_matches("(?)").trim().to_string();
            s = s.trim_start_matches("?)").trim().to_string();

            // pp space
            s = s.trim_start_matches("pp ").trim().to_string();
            // abbr, start with longer patterns
            s = re_abbr_more.replace(&s, "").trim().to_string();
            s = re_abbr_four.replace(&s, "").trim().to_string();
            s = re_abbr_three.replace(&s, "").trim().to_string();
            s = re_abbr_two.replace(&s, "").trim().to_string();
            s = re_abbr_one.replace(&s, "").trim().to_string();

            // FIXME somehow the above sometimes leaves the closing paren and dot
            s = s.trim_start_matches(')').trim().to_string();
            s = s.trim_start_matches('.').trim().to_string();
            s = s.trim_start_matches(';').trim().to_string();

            // ~ā
            s = s.trim_start_matches(r"~ā,").trim().to_string();
            s = s.trim_start_matches(r"~ā").trim().to_string();
            // (& m.)
            s = s.trim_start_matches(r"(& m.)").trim().to_string();
            s = s.trim_start_matches(r"(& f.)").trim().to_string();
            s = s.trim_start_matches(r"(& n.)").trim().to_string();

            // m(fn).
            s = s.trim_start_matches("(& mfn.)").trim().to_string();
            s = s.trim_start_matches("m(fn)").trim().to_string();
            s = s.trim_start_matches('.').trim().to_string();

            // m.a
            s = s.trim_start_matches("m.a").trim().to_string();
            // &
            s = s.trim_start_matches('&').trim().to_string();
            // fpp[.]
            s = s.trim_start_matches("fpp[.]").trim().to_string();

            n_iter += 1;

            if s == summary {
                // stop if there was no change
                break;
            } else if n_iter == max_iter {
                // or we hit max_iter
                info!("max_iter reached: {}", s);
                summary = s;
                break;
            } else {
                // apply changes and loop again
                summary = s;
            }
        }

        // cap the length of the final summary

        if !summary.is_empty() {
            let sum_length = 50;
            if summary.char_indices().count() > sum_length {
                let (char_idx, _char) = summary
                    .char_indices()
                    .nth(sum_length)
                    .ok_or("Bad char index")?;
                summary = summary[..char_idx].trim().to_string();
            }

            // FIXME empty summary gets this too somehow
            // append ...
            //summary.push_str(" ...");
        }

        self.word_header.summary = summary;

        Ok(())
    }
}

impl Default for DictWord {
    fn default() -> Self {
        DictWord {
            word_header: DictWordHeader::default(),
            definition_md: "definition".to_string(),
        }
    }
}

impl Default for DictWordHeader {
    fn default() -> Self {
        DictWordHeader {
            dict_label: "ABCD".to_string(),
            word: "word".to_string(),
            summary: "summary".to_string(),
            grammar: "m.".to_string(),
            inflections: Vec::new(),
            synonyms: Vec::new(),
            antonyms: Vec::new(),
        }
    }
}
