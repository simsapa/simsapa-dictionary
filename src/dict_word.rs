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

    #[serde(default)]
    pub phonetic: String,

    #[serde(default)]
    pub transliteration: String,

    /// comma-seperated list
    #[serde(default)]
    pub inflections: String,

    /// comma-seperated list
    #[serde(default)]
    pub synonyms: String,

    /// comma-seperated list
    #[serde(default)]
    pub antonyms: String,

    /// comma-seperated list
    #[serde(default)]
    pub see_also: String,

    /// comma-seperated list
    #[serde(default)]
    pub also_written_as: String,

    pub definition_md: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DictWordHeader {
    #[serde(default)]
    pub dict_label: String,

    pub word: String,

    #[serde(default)]
    pub url_id: String,
    #[serde(default)]
    pub summary: String,
    #[serde(default)]
    pub grammar: String,
    #[serde(default)]
    pub phonetic: String,
    #[serde(default)]
    pub transliteration: String,
    #[serde(default)]
    pub inflections: Vec<String>,
    #[serde(default)]
    pub synonyms: Vec<String>,
    #[serde(default)]
    pub antonyms: Vec<String>,
    #[serde(default)]
    pub see_also: Vec<String>,
    #[serde(default)]
    pub also_written_as: Vec<String>,
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
            definition_md: (*parts.get(1).unwrap()).to_string(),
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

    pub fn gen_url_id(word: &str, dict_label: &str) -> String {
        let a = if word.is_empty() {
            "untitled".to_string()
        } else {
            word.to_string()
        };

        let b = if dict_label.is_empty() {
            "unlabeled".to_string()
        } else {
            dict_label.to_string()
        };

        let id = format!("{}-{}", a, b);
        id.to_lowercase().replace('.', "-").replace(' ', "-")
    }

    pub fn set_url_id(&mut self) {
        self.word_header.url_id = DictWord::gen_url_id(
            &self.word_header.word,
            &self.word_header.dict_label);
    }

    pub fn from_xlsx(w: &DictWordXlsx) -> DictWord {
        DictWord {
            word_header: DictWordHeader {
                dict_label: w.dict_label.clone(),
                word: w.word.clone(),
                url_id: DictWord::gen_url_id(&w.word, &w.dict_label),
                summary: w.summary.clone(),
                grammar: w.grammar.clone(),
                phonetic: w.phonetic.clone(),
                transliteration: w.transliteration.clone(),
                inflections: DictWord::parse_csv_list(&w.inflections),
                synonyms: DictWord::parse_csv_list(&w.synonyms),
                antonyms: DictWord::parse_csv_list(&w.antonyms),
                see_also: DictWord::parse_csv_list(&w.see_also),
                also_written_as: DictWord::parse_csv_list(&w.also_written_as),
            },
            definition_md: w.definition_md.clone(),
        }
    }
}

impl DictWordXlsx {
    pub fn from_dict_word(w: &DictWord) -> DictWordXlsx {
        let h = w.word_header.clone();
        DictWordXlsx {
            dict_label: h.dict_label.clone(),
            word: h.word.clone(),
            summary: h.summary.clone(),
            grammar: h.grammar.clone(),
            phonetic: h.phonetic.clone(),
            transliteration: h.transliteration.clone(),
            inflections: h.inflections.join(", "),
            synonyms: h.synonyms.join(", "),
            antonyms: h.antonyms.join(", "),
            see_also: h.see_also.join(", "),
            also_written_as: h.also_written_as.join(", "),
            definition_md: w.definition_md.clone().trim().to_string(),
        }
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
            dict_label: "".to_string(),
            word: "word".to_string(),
            url_id: "word".to_string(),
            summary: "".to_string(),
            grammar: "".to_string(),
            phonetic: "".to_string(),
            transliteration: "".to_string(),
            inflections: Vec::new(),
            synonyms: Vec::new(),
            antonyms: Vec::new(),
            see_also: Vec::new(),
            also_written_as: Vec::new(),
        }
    }
}
