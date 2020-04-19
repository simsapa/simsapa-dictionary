use std::default::Default;
use std::error::Error;
use std::collections::BTreeMap;

use regex::Regex;

use crate::error::ToolError;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DictWordMarkdown {
    pub word_header: DictWordHeader,
    #[serde(default)]
    pub definition_md: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DictWordXlsx {
    /// `word` is the only required field.
    pub word: String,

    #[serde(default)]
    pub meaning_order: usize,

    /// Nominative singular form.
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
    pub variants: String,

    /// comma-seperated list
    #[serde(default)]
    pub also_written_as: String,

    /// comma-seperated list
    #[serde(default)]
    pub see_also: String,

    /// General comment or study notes.
    #[serde(default)]
    pub comment: String,

    // Grammar

    #[serde(default)]
    pub gr_roots: String,
    #[serde(default)]
    pub gr_prefix_and_root: String,

    #[serde(default)]
    pub gr_related_origin_word: String,
    #[serde(default)]
    pub gr_related_origin_roots: String,

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
    pub ex_1_text: String,
    #[serde(default)]
    pub ex_1_translation: String,

    #[serde(default)]
    pub ex_2_source_ref: String,
    #[serde(default)]
    pub ex_2_source_title: String,
    #[serde(default)]
    pub ex_2_text: String,
    #[serde(default)]
    pub ex_2_translation: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DictRootXlsx {
    #[serde(default)]
    pub root: String,

    #[serde(default)]
    pub meaning_order: usize,

    #[serde(default)]
    pub language: String,

    #[serde(default)]
    pub definition_md: String,

    #[serde(default)]
    pub groups: String,

    #[serde(default)]
    pub sign: String,

    #[serde(default)]
    pub numbered_group: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DictWordHeader {
    /// `word` is the only required field.
    pub word: String,

    #[serde(default)]
    pub meaning_order: usize,

    /// Nominative singular form.
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
    pub variants: Vec<String>,

    #[serde(default)]
    pub also_written_as: Vec<String>,

    #[serde(default)]
    pub see_also: Vec<String>,

    #[serde(default)]
    pub comment: String,

    // Grammar

    #[serde(default)]
    pub grammar_roots: Vec<String>,
    #[serde(default)]
    pub grammar_prefix_and_root: String,

    #[serde(default)]
    pub grammar_related_origin_word: String,
    #[serde(default)]
    pub grammar_related_origin_roots: Vec<String>,

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

    // FIXME examples should be Vec<DictWordExample>
    #[serde(default)]
    pub examples: String,

    #[serde(default)]
    pub url_id: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DictWordRender {
    /// The dictionary word lookup entry.
    pub word: String,

    /// The nominative singular form (if applies).
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

    #[serde(default)]
    pub meanings_count: usize,

    #[serde(default)]
    pub meanings: Vec<DictWordMeaning>,

    /// Used to create cross-link id attributes. Auto-generated internally.
    #[serde(default)]
    pub url_id: String,

}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DictWordMeaning {
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
    pub variants: Vec<String>,
    /// Spelling variations of the same word.
    #[serde(default)]
    pub also_written_as: Vec<String>,
    /// Related terms.
    #[serde(default)]
    pub see_also: Vec<String>,

    #[serde(default)]
    pub comment: String,

    #[serde(default)]
    pub grammar: DictWordGrammar,

    #[serde(default)]
    pub examples: Vec<DictWordExample>,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug)]
pub struct DictWordGrammar {
    /// ["upa", "gam"]
    #[serde(default)]
    roots: Vec<String>,

    /// "ā bhuj", for ābhujati
    #[serde(default)]
    prefix_and_root: String,

    /// Such as a Sanskrit word.
    #[serde(default)]
    related_origin_word: String,

    /// Such as Sanskrit roots.
    #[serde(default)]
    related_origin_roots: Vec<String>,

    /// "upa + gaccha + ti"
    #[serde(default)]
    construction: String,

    /// "gam + a = gaccha", Root and conjugation sign
    #[serde(default)]
    base_construction: String,

    /// kammadhāraya / etc.
    #[serde(default)]
    compound_type: String,
    /// abahula + kata
    #[serde(default)]
    compound_construction: String,

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
pub struct DictWordExample {
    /// "AN 5.11"
    #[serde(default)]
    source_ref: String,

    /// "paṭhama dārukkhandhopamasuttaṃ"
    #[serde(default)]
    source_title: String,

    /// "evam'eva kho, bhikkhave, sace tumhe'pi na orimaṃ tīraṃ upagacchatha, na pārimaṃ tīraṃ upagacchatha..."
    #[serde(default)]
    text: String,

    #[serde(default)]
    translation: String,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug)]
pub struct DictRoot {
    #[serde(default)]
    pub root: String,

    #[serde(default)]
    pub meaning_order: usize,

    #[serde(default)]
    pub language: String,

    /// Translation and explanation in English, Markdown format.
    #[serde(default)]
    pub definition_md: String,

    /// ["upa", "gam"]
    #[serde(default)]
    pub groups: Vec<String>,

    /// "a"
    #[serde(default)]
    pub sign: String,

    /// "1.1"
    #[serde(default)]
    pub numbered_group: String,
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
        let id_part = DictWordRender::gen_url_id(word, dict_label);

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
        // FIXME complete construction from xlsx to markdown
        DictWordMarkdown {
            word_header: DictWordHeader {
                word: w.word.clone(),
                meaning_order: w.meaning_order,
                word_nom_sg: w.word_nom_sg.clone(),
                dict_label: w.dict_label.clone(),

                inflections: DictWordMarkdown::parse_csv_list(&w.inflections),
                phonetic: w.phonetic.clone(),
                transliteration: w.transliteration.clone(),

                // Meaning

                summary: w.summary.clone(),

                synonyms: DictWordMarkdown::parse_csv_list(&w.synonyms),
                antonyms: DictWordMarkdown::parse_csv_list(&w.antonyms),
                variants: DictWordMarkdown::parse_csv_list(&w.variants),
                also_written_as: DictWordMarkdown::parse_csv_list(&w.also_written_as),
                see_also: DictWordMarkdown::parse_csv_list(&w.see_also),
                comment: w.comment.clone(),

                // Grammar

                grammar_roots: DictWordMarkdown::parse_csv_list(&w.gr_roots),
                grammar_prefix_and_root: w.gr_prefix_and_root.clone(),

                grammar_related_origin_word: w.gr_related_origin_word.clone(),
                grammar_related_origin_roots: DictWordMarkdown::parse_csv_list(&w.gr_related_origin_roots),

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

                // FIXME parse examples
                examples: "".to_string(),

                url_id: DictWordMarkdown::gen_url_id(&w.word, &w.dict_label, w.meaning_order),
            },
            definition_md: w.definition_md.clone(),
        }
    }
}

impl DictWordXlsx {
    // FIXME rename to from_dict_word_markdown
    pub fn from_dict_word(w: &DictWordMarkdown) -> DictWordXlsx {
        let h = w.word_header.clone();
        DictWordXlsx {
            word: h.word.clone(),
            meaning_order: h.meaning_order,
            word_nom_sg: h.word_nom_sg.clone(),
            dict_label: h.dict_label.clone(),

            inflections: h.inflections.join(", "),
            phonetic: h.phonetic.clone(),
            transliteration: h.transliteration.clone(),

            example_count: 0, // FIXME

            definition_md: w.definition_md.clone().trim().to_string(),

            summary: h.summary.clone(),

            synonyms: h.synonyms.join(", "),
            antonyms: h.antonyms.join(", "),
            variants: h.variants.join(", "),

            also_written_as: h.also_written_as.join(", "),
            see_also: h.see_also.join(", "),
            comment: h.comment.clone(),

            gr_roots: h.grammar_roots.join(", "),
            gr_prefix_and_root: h.grammar_prefix_and_root.clone(),

            gr_related_origin_word: h.grammar_related_origin_word.clone(),
            gr_related_origin_roots: h.grammar_related_origin_roots.join(", "),

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

            // FIXME parse examples
            // examples: h.examples.clone(),

            ex_1_source_ref: "".to_string(),
            ex_1_source_title: "".to_string(),
            ex_1_text: "".to_string(),
            ex_1_translation: "".to_string(),

            ex_2_source_ref: "".to_string(),
            ex_2_source_title: "".to_string(),
            ex_2_text: "".to_string(),
            ex_2_translation: "".to_string(),
        }
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
            dict_label: "".to_string(),

            inflections: Vec::new(),
            phonetic: "".to_string(),
            transliteration: "".to_string(),

            // Meaning

            summary: "".to_string(),

            synonyms: Vec::new(),
            antonyms: Vec::new(),
            variants: Vec::new(),
            also_written_as: Vec::new(),
            see_also: Vec::new(),
            comment: "".to_string(),

            // Grammar

            grammar_roots: Vec::new(),
            grammar_prefix_and_root: "".to_string(),

            grammar_related_origin_word: "".to_string(),
            grammar_related_origin_roots: Vec::new(),

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

            // FIXME examples should be Vec<DictWordExample>
            examples: "".to_string(),

            url_id: "word".to_string(),
        }
    }
}

impl DictWordRender {
    pub fn from_dict_word_markdown(w: &DictWordMarkdown) -> DictWordRender {
        let h = w.word_header.clone();

        let grammar = DictWordGrammar {
            roots: h.grammar_roots.clone(),
            prefix_and_root: h.grammar_prefix_and_root,
            related_origin_word: h.grammar_related_origin_word,
            related_origin_roots: h.grammar_related_origin_roots.clone(),

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

        let meaning = DictWordMeaning {
            meaning_order: h.meaning_order,
            definition_md: w.definition_md.clone(),
            summary: h.summary.clone(),
            synonyms: h.synonyms.clone(),
            antonyms: h.antonyms.clone(),
            variants: h.variants.clone(),
            also_written_as: h.also_written_as.clone(),
            see_also: h.see_also.clone(),
            comment: h.comment.clone(),
            grammar,

            // FIXME examples
            // examples: h.examples.clone(),
            examples: Vec::new(),
        };

        let meanings: Vec<DictWordMeaning> = vec![meaning];

        DictWordRender {
            word: h.word.clone(),
            word_nom_sg: h.word_nom_sg.clone(),
            dict_label: h.dict_label.clone(),
            inflections: h.inflections.clone(),
            phonetic: h.phonetic.clone(),
            transliteration: h.transliteration.clone(),
            meanings_count: 1,
            meanings,
            url_id: DictWordRender::gen_url_id(&h.word, &h.dict_label),
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
        self.url_id = DictWordRender::gen_url_id(
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

