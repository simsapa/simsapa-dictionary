use std::collections::BTreeMap;

use crate::dict_word::DictWordMarkdown;
use crate::pali;

pub struct LetterGroups {
    pub groups: Groups,
    pub words_to_url: BTreeMap<String, String>,
}

type Groups = BTreeMap<usize, LetterGroup>;

#[derive(Serialize, Deserialize, Clone)]
pub struct LetterGroup {
    pub title: String,
    pub group_letter: String,
    pub letter_index: usize,
    pub dict_words: Vec<DictWordMarkdown>,
}

impl Default for LetterGroups {
    fn default() -> LetterGroups {
        LetterGroups {
            groups: BTreeMap::new(),
            words_to_url: BTreeMap::new(),
        }
    }
}

impl LetterGroups {
    pub fn new_from_dict_words(dict_words: &[DictWordMarkdown]) -> LetterGroups {
        let mut groups: Groups = BTreeMap::new();
        let mut words_to_url: BTreeMap<String, String> = BTreeMap::new();

        for i in dict_words.iter() {
            let w = i.word_header.word.clone();
            let letter_index = pali::romanized_pali_letter_index(&w);
            let first_letter = match pali::first_letter(&w) {
                Some(x) => x,
                None => {
                    //warn!("Can't find the first letter of '{}', using 'a' and letter index {}", w, letter_index);
                    "a".to_string()
                }
            };

            groups
                .entry(letter_index)
                .or_insert(LetterGroup {
                    title: "".to_string(),
                    group_letter: first_letter.clone(),
                    letter_index,
                    dict_words: Vec::new(),
                })
                .dict_words
                .push(i.clone());

            words_to_url
                .entry(w.clone())
                .or_insert_with(||
                    format!("entries-{:02}.xhtml#{}", letter_index, i.word_header.url_id));

        }

        LetterGroups {
            groups,
            words_to_url,
        }
    }

    pub fn len(&self) -> usize {
        self.groups.keys().len()
    }

    pub fn is_empty(&self) -> bool {
        let k: Vec<usize> = self.groups.keys().cloned().collect();
        k.is_empty()
    }
}
