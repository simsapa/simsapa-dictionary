use std::default::Default;
use std::error::Error;
use std::collections::BTreeMap;

use regex::Regex;

use crate::error::ToolError;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DictWord {
    /// The dictionary word lookup entry.
    pub word: String,

    /// The nominative singular form (if applies).
    #[serde(default)]
    pub word_nom_sg: String,

    /// A label to distinguish dictionary sources or authors.
    #[serde(default)]
    pub dict_label: String,

    /// Inflected or conjugated forms such as plurals, which should return this word entry.
    #[serde(default)]
    pub inflections: Vec<String>,

    /// Phonetic spelling, such as IPA.
    #[serde(default)]
    pub phonetic: String,

    /// Transliteration to Latin from other alphabets such as Thai or Chinese.
    #[serde(default)]
    pub transliteration: String,

    /// Used to create cross-link id attributes, auto-generated in this crate.
    #[serde(default)]
    pub url_id: String,

    #[serde(default)]
    pub meanings: Vec<Meaning>,

    /// (Used internally)
    #[serde(default)]
    pub meanings_count: usize,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Meaning {
    /// Used for sorting.
    #[serde(default)]
    pub meaning_order: usize,

    /// Translation and explanation in English, Markdown format.
    #[serde(default)]
    pub definition_md: String,

    /// Short translation in English, plain text.
    #[serde(default)]
    pub summary: String,

    /// Different words with similar meaning.
    #[serde(default)]
    pub synonyms: Vec<String>,
    /// Opposite meanings.
    #[serde(default)]
    pub antonyms: Vec<String>,
    /// Similar form or construction but different meaning.
    #[serde(default)]
    pub homonyms: Vec<String>,
    /// Spelling variations of the same word.
    #[serde(default)]
    pub also_written_as: Vec<String>,
    /// Related terms.
    #[serde(default)]
    pub see_also: Vec<String>,

    #[serde(default)]
    pub comment: String,

    // Root meaning specific

    /// Marking root entries to separate them from words.
    #[serde(default)]
    pub is_root: bool,

    #[serde(default)]
    pub root_language: String,

    /// ["upa", "gam"]
    #[serde(default)]
    pub root_groups: Vec<String>,

    /// "a"
    #[serde(default)]
    pub root_sign: String,

    /// "1.1"
    #[serde(default)]
    pub root_numbered_group: String,

    #[serde(default)]
    pub grammar: Grammar,

    #[serde(default)]
    pub examples: Vec<Example>,

    /// (Used internally)
    #[serde(default)]
    pub example_count: usize,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug)]
pub struct Grammar {
    /// ["upa", "gam"]
    #[serde(default)]
    pub roots: Vec<String>,

    /// "ā bhuj", for ābhujati
    #[serde(default)]
    pub prefix_and_root: String,

    /// "upa + gaccha + ti"
    #[serde(default)]
    pub construction: String,

    /// "gam + a = gaccha", Root and conjugation sign
    #[serde(default)]
    pub base_construction: String,

    /// kammadhāraya / etc.
    #[serde(default)]
    pub compound_type: String,
    /// abahula + kata
    #[serde(default)]
    pub compound_construction: String,

    /// "pp. of upagata", General grammar comment
    #[serde(default)]
    pub comment: String,

    /// "verb", Part of speech.
    #[serde(default)]
    pub speech: String,

    /// "acc."
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

    /// trans. / intrans. / ditrans. / empty
    #[serde(default)]
    pub transitive: String,

    /// true / false / empty
    #[serde(default)]
    pub negative: String,

    /// causative / passive / denominate / intensive / empty
    #[serde(default)]
    pub verb: String,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug)]
pub struct Example {
    /// "AN 5.11"
    #[serde(default)]
    pub source_ref: String,

    /// "paṭhama dārukkhandhopamasuttaṃ"
    #[serde(default)]
    pub source_title: String,

    /// "evam'eva kho, bhikkhave, sace tumhe'pi na orimaṃ tīraṃ upagacchatha, na pārimaṃ tīraṃ upagacchatha..."
    #[serde(default)]
    pub text_md: String,

    #[serde(default)]
    pub translation_md: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DictWordMarkdown {
    pub word_header: DictWordHeader,
    #[serde(default)]
    pub definition_md: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DictWordXlsx {
    /// `word` is the only required field. Every other field has a default or empty.
    pub word: String,

    #[serde(default)]
    pub meaning_order: usize,

    /// Nominative singular form.
    #[serde(default)]
    pub word_nom_sg: String,

    #[serde(default)]
    pub dict_label: String,

    /// comma-seperated list
    #[serde(default)]
    pub inflections: String,

    #[serde(default)]
    pub phonetic: String,

    #[serde(default)]
    pub transliteration: String,

    /// A helper number to mark the number of examples collected for this meaning. This is a visual
    /// aid in the Spreadsheet.
    #[serde(default)]
    pub example_count: usize,

    // Meaning

    #[serde(default)]
    pub definition_md: String,

    #[serde(default)]
    pub summary: String,

    /// comma-seperated list
    #[serde(default)]
    pub synonyms: String,

    /// comma-seperated list
    #[serde(default)]
    pub antonyms: String,

    /// comma-seperated list
    #[serde(default)]
    pub homonyms: String,

    /// comma-seperated list
    #[serde(default)]
    pub also_written_as: String,

    /// comma-seperated list
    #[serde(default)]
    pub see_also: String,

    /// General comment or study notes.
    #[serde(default)]
    pub comment: String,

    // Root meaning specific

    /// Marking root meanings to separate them from words.
    #[serde(default)]
    pub is_root: bool,

    #[serde(default)]
    pub root_language: String,

    /// comma-seperated list
    #[serde(default)]
    pub root_groups: String,

    /// "a"
    #[serde(default)]
    pub root_sign: String,

    /// "1.1"
    #[serde(default)]
    pub root_numbered_group: String,

    // Grammar

    #[serde(default)]
    pub gr_roots: String,
    #[serde(default)]
    pub gr_prefix_and_root: String,

    #[serde(default)]
    pub gr_construction: String,
    #[serde(default)]
    pub gr_base_construction: String,
    #[serde(default)]
    pub gr_compound_type: String,
    #[serde(default)]
    pub gr_compound_construction: String,

    #[serde(default)]
    pub gr_comment: String,
    #[serde(default)]
    pub gr_speech: String,
    #[serde(default)]
    pub gr_case: String,
    #[serde(default)]
    pub gr_num: String,
    #[serde(default)]
    pub gr_gender: String,
    #[serde(default)]
    pub gr_person: String,
    #[serde(default)]
    pub gr_voice: String,
    #[serde(default)]
    pub gr_object: String,
    #[serde(default)]
    pub gr_transitive: String,
    #[serde(default)]
    pub gr_negative: String,
    #[serde(default)]
    pub gr_verb: String,

    // Examples

    #[serde(default)]
    pub ex_1_source_ref: String,
    #[serde(default)]
    pub ex_1_source_title: String,
    #[serde(default)]
    pub ex_1_text_md: String,
    #[serde(default)]
    pub ex_1_translation_md: String,

    #[serde(default)]
    pub ex_2_source_ref: String,
    #[serde(default)]
    pub ex_2_source_title: String,
    #[serde(default)]
    pub ex_2_text_md: String,
    #[serde(default)]
    pub ex_2_translation_md: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DictWordHeader {
    /// `word` is the only required field.
    pub word: String,

    #[serde(default)]
    pub meaning_order: usize,

    /// Nominative singular form.
    #[serde(default)]
    pub word_nom_sg: String,

    #[serde(default)]
    pub dict_label: String,

    #[serde(default)]
    pub inflections: Vec<String>,

    #[serde(default)]
    pub phonetic: String,

    #[serde(default)]
    pub transliteration: String,

    // Meaning

    #[serde(default)]
    pub summary: String,

    #[serde(default)]
    pub synonyms: Vec<String>,

    #[serde(default)]
    pub antonyms: Vec<String>,

    #[serde(default)]
    pub homonyms: Vec<String>,

    #[serde(default)]
    pub also_written_as: Vec<String>,

    #[serde(default)]
    pub see_also: Vec<String>,

    #[serde(default)]
    pub comment: String,

    // Root meaning specific

    #[serde(default)]
    pub is_root: bool,

    #[serde(default)]
    pub root_language: String,

    #[serde(default)]
    pub root_groups: Vec<String>,

    #[serde(default)]
    pub root_sign: String,

    #[serde(default)]
    pub root_numbered_group: String,

    // Grammar

    #[serde(default)]
    pub grammar_roots: Vec<String>,
    #[serde(default)]
    pub grammar_prefix_and_root: String,

    #[serde(default)]
    pub grammar_construction: String,
    #[serde(default)]
    pub grammar_base_construction: String,
    #[serde(default)]
    pub grammar_compound_type: String,
    #[serde(default)]
    pub grammar_compound_construction: String,

    #[serde(default)]
    pub grammar_comment: String,
    #[serde(default)]
    pub grammar_speech: String,
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
    #[serde(default)]
    pub grammar_transitive: String,
    #[serde(default)]
    pub grammar_negative: String,
    #[serde(default)]
    pub grammar_verb: String,

    #[serde(default)]
    pub examples: Vec<Example>,

    #[serde(default)]
    pub url_id: String,
}

impl DictWordMarkdown {
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

    pub fn from_markdown(s: &str) -> Result<DictWordMarkdown, Box<dyn Error>> {
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

        Ok(DictWordMarkdown {
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
        let id_part = DictWord::gen_url_id(word, dict_label);

        format!("{}-{}", id_part, meaning_order)
    }

    pub fn set_url_id(&mut self) {
        self.word_header.url_id = DictWordMarkdown::gen_url_id(
            &self.word_header.word,
            &self.word_header.dict_label,
            self.word_header.meaning_order,
        );
    }

    pub fn from_xlsx(w: &DictWordXlsx) -> DictWordMarkdown {
        let mut examples: Vec<Example> = Vec::new();

        {
            let ex = Example {
                source_ref: w.ex_1_source_ref.clone(),
                source_title: w.ex_1_source_title.clone(),
                text_md: w.ex_1_text_md.clone(),
                translation_md: w.ex_1_translation_md.clone(),
            };
            let n = ex.source_ref.len() + ex.source_title.len() + ex.text_md.len() + ex.translation_md.len();
            if n > 0 {
                examples.push(ex);
            }
        }

        {
            let ex = Example {
                source_ref: w.ex_2_source_ref.clone(),
                source_title: w.ex_2_source_title.clone(),
                text_md: w.ex_2_text_md.clone(),
                translation_md: w.ex_2_translation_md.clone(),
            };
            let n = ex.source_ref.len() + ex.source_title.len() + ex.text_md.len() + ex.translation_md.len();
            if n > 0 {
                examples.push(ex);
            }
        }

        DictWordMarkdown {
            word_header: DictWordHeader {
                word: w.word.clone(),
                meaning_order: w.meaning_order,
                word_nom_sg: w.word_nom_sg.clone(),
                is_root: w.is_root,
                dict_label: w.dict_label.clone(),

                inflections: DictWordMarkdown::parse_csv_list(&w.inflections),
                phonetic: w.phonetic.clone(),
                transliteration: w.transliteration.clone(),

                // Meaning

                summary: w.summary.clone(),

                synonyms: DictWordMarkdown::parse_csv_list(&w.synonyms),
                antonyms: DictWordMarkdown::parse_csv_list(&w.antonyms),
                homonyms: DictWordMarkdown::parse_csv_list(&w.homonyms),
                also_written_as: DictWordMarkdown::parse_csv_list(&w.also_written_as),
                see_also: DictWordMarkdown::parse_csv_list(&w.see_also),
                comment: w.comment.clone(),

                // Grammar

                grammar_roots: DictWordMarkdown::parse_csv_list(&w.gr_roots),
                grammar_prefix_and_root: w.gr_prefix_and_root.clone(),

                grammar_construction: w.gr_construction.clone(),
                grammar_base_construction: w.gr_base_construction.clone(),
                grammar_compound_type: w.gr_compound_type.clone(),
                grammar_compound_construction: w.gr_compound_construction.clone(),

                grammar_comment: w.gr_comment.clone(),
                grammar_speech: w.gr_speech.clone(),
                grammar_case: w.gr_case.clone(),
                grammar_num: w.gr_num.clone(),
                grammar_gender: w.gr_gender.clone(),
                grammar_person: w.gr_person.clone(),
                grammar_voice: w.gr_voice.clone(),
                grammar_object: w.gr_object.clone(),
                grammar_transitive: w.gr_transitive.clone(),
                grammar_negative: w.gr_negative.clone(),
                grammar_verb: w.gr_verb.clone(),

                examples,

                root_language: w.root_language.clone(),
                root_groups: DictWordMarkdown::parse_csv_list(&w.root_groups),
                root_sign: w.root_sign.clone(),
                root_numbered_group: w.root_numbered_group.clone(),

                url_id: DictWordMarkdown::gen_url_id(&w.word, &w.dict_label, w.meaning_order),
            },
            definition_md: w.definition_md.clone(),
        }
    }
}

impl DictWordXlsx {
    pub fn from_dict_word_markdown(w: &DictWordMarkdown) -> DictWordXlsx {
        let h = w.word_header.clone();
        let mut a = DictWordXlsx {
            word: h.word.clone(),
            meaning_order: h.meaning_order,
            word_nom_sg: h.word_nom_sg.clone(),
            is_root: h.is_root,
            dict_label: h.dict_label.clone(),

            inflections: h.inflections.join(", "),
            phonetic: h.phonetic.clone(),
            transliteration: h.transliteration.clone(),

            example_count: 0,

            definition_md: w.definition_md.clone().trim().to_string(),

            summary: h.summary.clone(),

            synonyms: h.synonyms.join(", "),
            antonyms: h.antonyms.join(", "),
            homonyms: h.homonyms.join(", "),

            also_written_as: h.also_written_as.join(", "),
            see_also: h.see_also.join(", "),
            comment: h.comment.clone(),

            gr_roots: h.grammar_roots.join(", "),
            gr_prefix_and_root: h.grammar_prefix_and_root.clone(),

            gr_construction: h.grammar_construction.clone(),
            gr_base_construction: h.grammar_base_construction.clone(),
            gr_compound_type: h.grammar_compound_type.clone(),
            gr_compound_construction: h.grammar_compound_construction.clone(),

            gr_comment: h.grammar_comment.clone(),
            gr_speech: h.grammar_speech.clone(),
            gr_case: h.grammar_case.clone(),
            gr_num: h.grammar_num.clone(),
            gr_gender: h.grammar_gender.clone(),
            gr_person: h.grammar_person.clone(),
            gr_voice: h.grammar_voice.clone(),
            gr_object: h.grammar_object.clone(),
            gr_transitive: h.grammar_transitive.clone(),
            gr_negative: h.grammar_negative.clone(),
            gr_verb: h.grammar_verb.clone(),

            ex_1_source_ref: "".to_string(),
            ex_1_source_title: "".to_string(),
            ex_1_text_md: "".to_string(),
            ex_1_translation_md: "".to_string(),

            ex_2_source_ref: "".to_string(),
            ex_2_source_title: "".to_string(),
            ex_2_text_md: "".to_string(),
            ex_2_translation_md: "".to_string(),

            root_language: h.root_language.clone(),
            root_groups: h.root_groups.join(", "),
            root_sign: h.root_sign.clone(),
            root_numbered_group: h.root_numbered_group.clone(),
        };

        if let Some(ex) = w.word_header.examples.get(0) {
            a.ex_1_source_ref = ex.source_ref.clone();
            a.ex_1_source_title = ex.source_title.clone();
            a.ex_1_text_md = ex.text_md.clone();
            a.ex_1_translation_md = ex.translation_md.clone();

            let n = ex.source_ref.len() + ex.source_title.len() + ex.text_md.len() + ex.translation_md.len();
            if n > 0 {
                a.example_count += 1;
            }
        }

        if let Some(ex) = w.word_header.examples.get(1) {
            a.ex_2_source_ref = ex.source_ref.clone();
            a.ex_2_source_title = ex.source_title.clone();
            a.ex_2_text_md = ex.text_md.clone();
            a.ex_2_translation_md = ex.translation_md.clone();

            let n = ex.source_ref.len() + ex.source_title.len() + ex.text_md.len() + ex.translation_md.len();
            if n > 0 {
                a.example_count += 1;
            }
        }

        a
    }
}

impl Default for DictWordMarkdown {
    fn default() -> Self {
        DictWordMarkdown {
            word_header: DictWordHeader::default(),
            definition_md: "definition".to_string(),
        }
    }
}

impl Default for DictWordHeader {
    fn default() -> Self {
        DictWordHeader {
            word: "word".to_string(),
            meaning_order: 1,
            word_nom_sg: "".to_string(),
            is_root: false,
            dict_label: "".to_string(),

            inflections: Vec::new(),
            phonetic: "".to_string(),
            transliteration: "".to_string(),

            // Meaning

            summary: "".to_string(),

            synonyms: Vec::new(),
            antonyms: Vec::new(),
            homonyms: Vec::new(),
            also_written_as: Vec::new(),
            see_also: Vec::new(),
            comment: "".to_string(),

            // Grammar

            grammar_roots: Vec::new(),
            grammar_prefix_and_root: "".to_string(),

            grammar_construction: "".to_string(),
            grammar_base_construction: "".to_string(),
            grammar_compound_type: "".to_string(),
            grammar_compound_construction: "".to_string(),

            grammar_comment: "".to_string(),
            grammar_speech: "".to_string(),
            grammar_case: "".to_string(),
            grammar_num: "".to_string(),
            grammar_gender: "".to_string(),
            grammar_person: "".to_string(),
            grammar_voice: "".to_string(),
            grammar_object: "".to_string(),
            grammar_transitive: "".to_string(),
            grammar_negative: "".to_string(),
            grammar_verb: "".to_string(),

            examples: Vec::new(),

            root_language: "".to_string(),
            root_groups: Vec::new(),
            root_sign: "".to_string(),
            root_numbered_group: "".to_string(),

            url_id: "word".to_string(),
        }
    }
}

impl DictWord {
    pub fn from_dict_word_markdown(w: &DictWordMarkdown) -> DictWord {
        let h = w.word_header.clone();

        let grammar = Grammar {
            roots: h.grammar_roots.clone(),
            prefix_and_root: h.grammar_prefix_and_root,

            construction: h.grammar_construction,
            base_construction: h.grammar_base_construction,
            compound_type: h.grammar_compound_type,
            compound_construction: h.grammar_compound_construction,

            comment: h.grammar_comment,
            speech: h.grammar_speech,
            case: h.grammar_case,
            num: h.grammar_num,
            gender: h.grammar_gender,
            person: h.grammar_person,
            voice: h.grammar_voice,
            object: h.grammar_object,
            transitive: h.grammar_transitive,
            negative: h.grammar_negative,
            verb: h.grammar_verb,
        };

        let meaning = Meaning {
            meaning_order: h.meaning_order,
            definition_md: w.definition_md.clone(),
            summary: h.summary.clone(),
            synonyms: h.synonyms.clone(),
            antonyms: h.antonyms.clone(),
            homonyms: h.homonyms.clone(),
            also_written_as: h.also_written_as.clone(),
            see_also: h.see_also.clone(),
            comment: h.comment.clone(),
            is_root: h.is_root,
            root_language: h.root_language.clone(),
            root_groups: h.root_groups.clone(),
            root_sign: h.root_sign.clone(),
            root_numbered_group: h.root_numbered_group.clone(),
            grammar,
            example_count: h.examples.len(),
            examples: h.examples.clone(),
        };

        let meanings: Vec<Meaning> = vec![meaning];

        DictWord {
            word: h.word.clone(),
            word_nom_sg: h.word_nom_sg.clone(),
            dict_label: h.dict_label.clone(),
            inflections: h.inflections.clone(),
            phonetic: h.phonetic.clone(),
            transliteration: h.transliteration.clone(),
            meanings_count: 1,
            meanings,
            url_id: DictWord::gen_url_id(&h.word, &h.dict_label),
        }
    }

    pub fn gen_url_id(word: &str, dict_label: &str) -> String {
        let id_word = if word.trim().is_empty() {
            "untitled".to_string()
        } else {
            text_to_url_id_part(word.trim())
        };

        let id_label = if dict_label.trim().is_empty() {
            "unlabeled".to_string()
        } else {
            text_to_url_id_part(dict_label.trim())
        };

        format!("{}-{}", id_word, id_label)
    }

    pub fn set_url_id(&mut self) {
        self.url_id = DictWord::gen_url_id(
            &self.word,
            &self.dict_label,
        );
    }

}

fn text_to_url_id_part(text: &str) -> String {
    lazy_static! {
        static ref RE_INVALID_URL_ID: Regex = Regex::new(r"([^a-z0-9āīūṃṁṅñṭṭḍḍṇḷ-])").unwrap();
    }

    let mut id = text.to_lowercase().replace('.', "-").replace(' ', "-");

    let mut replace_list: BTreeMap<char, String> = BTreeMap::new();

    for caps in RE_INVALID_URL_ID.captures_iter(&id) {
        let from = caps.get(1).unwrap().as_str().to_string();
        let letter = from.chars().next().unwrap();

        replace_list
            .entry(letter)
            .or_insert_with(|| char_to_unicode_codepoint_text(letter));
    }

    for (from_letter, to_text) in replace_list.iter() {
        id = id.replace(&format!("{}", from_letter), to_text);
    }

    id
}

fn char_to_unicode_codepoint_text(letter: char) -> String {
    let text: String = letter.escape_unicode().collect();
    text
        .replace(r"\u", "u")
        .replace('{', "")
        .replace('}', "")
}

