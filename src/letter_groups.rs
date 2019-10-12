use std::collections::BTreeMap;

use crate::dict_word::DictWord;
use crate::pali;

pub struct LetterGroups {
    pub groups: Groups,
}

type Groups = BTreeMap<usize, LetterGroup>;

#[derive(Serialize, Deserialize)]
pub struct LetterGroup {
    pub title: String,
    pub group_letter: String,
    pub letter_index: usize,
    pub dict_words: Vec<DictWord>,
}

impl LetterGroups {
    pub fn new_from_dict_words(dict_words: &[DictWord]) -> LetterGroups {
        let mut groups: Groups = BTreeMap::new();

        for i in dict_words.iter() {
            let w = i.word_header.word.clone();
            let key = pali::romanized_pali_letter_index(&w);
            let first_letter = match pali::first_letter(&w) {
                Some(x) => x,
                None => {
                    error!("Can't find first Pali letter of: {}", w);
                    "a".to_string()
                }
            };

            groups
                .entry(key)
                .or_insert(LetterGroup {
                    title: "".to_string(),
                    group_letter: first_letter,
                    letter_index: key,
                    dict_words: vec![i.clone()],
                })
                .dict_words
                .push(i.clone());
        }

        LetterGroups { groups }
    }

    pub fn len(&self) -> usize {
        self.groups.keys().len()
    }

    pub fn is_empty(&self) -> bool {
        let k: Vec<usize> = self.groups.keys().cloned().collect();
        k.is_empty()
    }
}
