use std::default::Default;
use std::error::Error;

use regex::Regex;

use crate::error::ToolError;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DictWord {
    pub word_header: DictWordHeader,
    #[serde(default)]
    pub definition_md: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DictWordXlsx {

    /// `word` is the only required field.
    pub word: String,

    #[serde(default)]
    pub dict_label: String,

    #[serde(default)]
    pub meaning_order: usize,

    #[serde(default)]
    pub summary: String,

    /// specific grammar properties
    #[serde(default)]
    pub grammar_case: String,
    #[serde(default)]
    pub grammar_num: String,
    #[serde(default)]
    pub grammar_gender: String,
    #[serde(default)]
    pub grammar_person: String,
    #[serde(default)]
    pub grammar_voice: String,
    #[serde(default)]
    pub grammar_object: String,
    /// general grammar comment
    #[serde(default)]
    pub grammar_comment: String,

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

    #[serde(default)]
    pub examples: String,

    #[serde(default)]
    pub definition_md: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DictWordHeader {
    /// `word` is the only required field.
    pub word: String,

    #[serde(default)]
    pub dict_label: String,
    #[serde(default)]
    pub meaning_order: usize,
    #[serde(default)]
    pub url_id: String,
    #[serde(default)]
    pub summary: String,

    /// specific grammar properties
    #[serde(default)]
    pub grammar_case: String,
    #[serde(default)]
    pub grammar_num: String,
    #[serde(default)]
    pub grammar_gender: String,
    #[serde(default)]
    pub grammar_person: String,
    #[serde(default)]
    pub grammar_voice: String,
    #[serde(default)]
    pub grammar_object: String,
    /// general grammar comment
    #[serde(default)]
    pub grammar_comment: String,

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
    #[serde(default)]
    pub examples: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DictWordRender {
    pub word: String,

    #[serde(default)]
    pub dict_label: String,

    #[serde(default)]
    pub url_id: String,

    #[serde(default)]
    pub meanings: Vec<DictWordMeaning>,

    #[serde(default)]
    pub meanings_count: usize,

    #[serde(default)]
    pub phonetic: String,
    #[serde(default)]
    pub transliteration: String,
    #[serde(default)]
    pub inflections: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DictWordMeaning {
    #[serde(default)]
    pub meaning_order: usize,

    #[serde(default)]
    pub grammar: DictWordGrammar,

    #[serde(default)]
    pub summary: String,

    #[serde(default)]
    pub definition_md: String,

    #[serde(default)]
    pub synonyms: Vec<String>,
    #[serde(default)]
    pub antonyms: Vec<String>,
    #[serde(default)]
    pub see_also: Vec<String>,
    #[serde(default)]
    pub also_written_as: Vec<String>,
    #[serde(default)]
    pub examples: String,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug)]
pub struct DictWordGrammar {
    /// specific grammar properties
    #[serde(default)]
    pub case: String,
    #[serde(default)]
    pub num: String,
    #[serde(default)]
    pub gender: String,
    #[serde(default)]
    pub person: String,
    #[serde(default)]
    pub voice: String,
    #[serde(default)]
    pub object: String,

    /// general grammar comment
    #[serde(default)]
    pub comment: String,
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

    pub fn gen_url_id(word: &str, dict_label: &str, meaning_order: usize) -> String {
        let id_word = if word.is_empty() {
            "untitled".to_string()
        } else {
            word.to_string()
        };

        let id_label = if dict_label.is_empty() {
            "unlabeled".to_string()
        } else {
            dict_label.to_string()
        };

        let id = format!("{}-{}-{}", id_word, id_label, meaning_order);
        clean_url_id(&id)
    }

    pub fn set_url_id(&mut self) {
        self.word_header.url_id = DictWord::gen_url_id(
            &self.word_header.word,
            &self.word_header.dict_label,
            self.word_header.meaning_order,
        );
    }

    pub fn from_xlsx(w: &DictWordXlsx) -> DictWord {
        DictWord {
            word_header: DictWordHeader {
                dict_label: w.dict_label.clone(),
                meaning_order: w.meaning_order,
                word: w.word.clone(),
                url_id: DictWord::gen_url_id(&w.word, &w.dict_label, w.meaning_order),
                summary: w.summary.clone(),

                grammar_case: w.grammar_case.clone(),
                grammar_num: w.grammar_num.clone(),
                grammar_gender: w.grammar_gender.clone(),
                grammar_person: w.grammar_person.clone(),
                grammar_voice: w.grammar_voice.clone(),
                grammar_object: w.grammar_object.clone(),
                grammar_comment: w.grammar_comment.clone(),

                phonetic: w.phonetic.clone(),
                transliteration: w.transliteration.clone(),
                inflections: DictWord::parse_csv_list(&w.inflections),
                synonyms: DictWord::parse_csv_list(&w.synonyms),
                antonyms: DictWord::parse_csv_list(&w.antonyms),
                see_also: DictWord::parse_csv_list(&w.see_also),
                also_written_as: DictWord::parse_csv_list(&w.also_written_as),
                examples: w.examples.clone(),
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
            meaning_order: h.meaning_order,
            word: h.word.clone(),
            summary: h.summary.clone(),

            grammar_case: h.grammar_case.clone(),
            grammar_num: h.grammar_num.clone(),
            grammar_gender: h.grammar_gender.clone(),
            grammar_person: h.grammar_person.clone(),
            grammar_voice: h.grammar_voice.clone(),
            grammar_object: h.grammar_object.clone(),
            grammar_comment: h.grammar_comment.clone(),

            phonetic: h.phonetic.clone(),
            transliteration: h.transliteration.clone(),
            inflections: h.inflections.join(", "),
            synonyms: h.synonyms.join(", "),
            antonyms: h.antonyms.join(", "),
            see_also: h.see_also.join(", "),
            also_written_as: h.also_written_as.join(", "),
            definition_md: w.definition_md.clone().trim().to_string(),
            examples: h.examples.clone(),
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
            meaning_order: 1,
            word: "word".to_string(),
            url_id: "word".to_string(),
            summary: "".to_string(),

            grammar_case: "".to_string(),
            grammar_num: "".to_string(),
            grammar_gender: "".to_string(),
            grammar_person: "".to_string(),
            grammar_voice: "".to_string(),
            grammar_object: "".to_string(),
            grammar_comment: "".to_string(),

            phonetic: "".to_string(),
            transliteration: "".to_string(),
            inflections: Vec::new(),
            synonyms: Vec::new(),
            antonyms: Vec::new(),
            see_also: Vec::new(),
            also_written_as: Vec::new(),
            examples: "".to_string(),
        }
    }
}

impl DictWordRender {
    pub fn from_dict_word(w: &DictWord) -> DictWordRender {
        let h = w.word_header.clone();

        let grammar = DictWordGrammar {
            case: h.grammar_case,
            num: h.grammar_num,
            gender: h.grammar_gender,
            person: h.grammar_person,
            voice: h.grammar_voice,
            object: h.grammar_object,
            comment: h.grammar_comment,
        };

        let meaning = DictWordMeaning {
            meaning_order: h.meaning_order,
            grammar,
            summary: h.summary.clone(),
            definition_md: w.definition_md.clone(),
            synonyms: h.synonyms.clone(),
            antonyms: h.antonyms.clone(),
            see_also: h.see_also.clone(),
            also_written_as: h.also_written_as.clone(),
            examples: h.examples.clone(),
        };

        let meanings: Vec<DictWordMeaning> = vec![meaning];

        DictWordRender {
            word: h.word.clone(),
            dict_label: h.dict_label.clone(),
            url_id: DictWordRender::gen_url_id(&h.word, &h.dict_label),
            meanings,
            meanings_count: 1,
            phonetic: h.phonetic.clone(),
            transliteration: h.transliteration.clone(),
            inflections: h.inflections.clone(),
        }
    }

    pub fn gen_url_id(word: &str, dict_label: &str) -> String {
        let id_word = if word.is_empty() {
            "untitled".to_string()
        } else {
            word.to_string()
        };

        let id_label = if dict_label.is_empty() {
            "unlabeled".to_string()
        } else {
            dict_label.to_string()
        };

        let id = format!("{}-{}", id_word, id_label);
        clean_url_id(&id)
    }

    pub fn set_url_id(&mut self) {
        self.url_id = DictWordRender::gen_url_id(
            &self.word,
            &self.dict_label,
        );
    }

}

fn clean_url_id(text: &str) -> String {
    lazy_static! {
        static ref RE_INVALID_URL_ID: Regex = Regex::new(r"[^a-z0-9āīūṃṁṅñṭṭḍḍṇḷ-]").unwrap();
    }

    let id = text.to_lowercase().replace('.', "-").replace(' ', "-");
    RE_INVALID_URL_ID.replace_all(&id, "-").to_string()
}

