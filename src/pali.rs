/*
Pali alphabetical order:

a ā i ī u ū e o ṃ / ṁ k kh g gh ṅ c ch j jh ñ ṭ ṭh ḍ ḍh ṇ t th d dh n p ph b bh m y r l ḷ v s h

A Ā I Ī U Ū E O Ṃ / Ṁ K KH G GH Ṅ C CH J JH Ñ Ṭ ṬH Ḍ ḌH Ṇ T TH D DH N P PH B BH M Y R L Ḷ V S H

Romanized, alphabetical order, Pali only:

a ā b bh c ch d dh ḍ ḍh e g gh h i ī j jh k kh l ḷ m ṃ / ṁ n ṅ ṇ ñ o p ph r s t th ṭ ṭh u ū v y

Romanized alphabetical order, including ISO Basic Latin:

a ā b bh c ch d dh ḍ ḍh e f g gh h i ī j jh k kh l ḷ m ṃ / ṁ n ṅ ṇ ñ o p ph q r s t th ṭ ṭh u ū v w x y z

*/

// Not including ṁ.
//
// Including ISO Basic Latin, to allow for English entries from Nyanatiloka

//                                                             10                    20                   30                     40
//                                     0 1 2 3  4 5  6 7  8 9  0 1 2 3  4 5 6 7 8  9 0  1 2 3 4 5 6 7 8 9 0 1  2 3 4 5 6  7 8  9 0 1 2 3 4 5
const ROMANIZED_PALI_ALPHABET: &str = "a ā b bh c ch d dh ḍ ḍh e f g gh h i ī j jh k kh l ḷ m ṃ n ṅ ṇ ñ o p ph q r s t th ṭ ṭh u ū v w x y z -";

const RPA_DOUBLES_FIRST: &str = "bh ch dh ḍh gh jh kh ph th ṭh a ā b c d ḍ e f g h i ī j k l ḷ m ṃ n ṅ ṇ ñ o p q r s t ṭ u ū v w x y z -";

pub fn romanized_pali_letter_index(word: &str) -> usize {
    let alphabet: Vec<String> = ROMANIZED_PALI_ALPHABET
        .split(' ')
        .map(String::from)
        .collect();
    let word_clean = word.trim().to_lowercase();

    match first_letter(&word_clean) {
        Some(letter) => {
            for (idx, l) in alphabet.iter().enumerate() {
                if letter == *l {
                    return idx;
                }
            }
            alphabet.len()
        },
        None => alphabet.len(),
    }
}

pub fn first_letter(word: &str) -> Option<String> {
    let doubles_first: Vec<&str> = RPA_DOUBLES_FIRST.split(' ').collect();
    let word_clean = word.trim().to_lowercase();

    for i in doubles_first.iter() {
        if word_clean.starts_with(*i) {
            return Some(String::from(*i));
        }
    }

    None
}

pub fn to_velthuis(word: &str) -> String {
    // https://en.wikipedia.org/wiki/Pali#Text_in_ASCII
    word
        .replace('ā', "aa")
        .replace('ī', "ii")
        .replace('ū', "uu")
        .replace('ṃ', ".m")
        .replace('ṁ', ".m")
        .replace('ṇ', ".n")
        .replace('ñ', "~n")
        .replace('ṭ', ".t")
        .replace('ḍ', ".d")
        .replace('ṅ', "\"n")
        .replace('ḷ', ".l")
        .replace('Ā', "AA")
        .replace('Ī', "II")
        .replace('Ū', "UU")
        .replace('Ṃ', ".M")
        .replace('Ṁ', ".M")
        .replace('Ṇ', ".N")
        .replace('Ñ', "~N")
        .replace('Ṭ', ".T")
        .replace('Ḍ', ".D")
        .replace('Ṅ', "\"N")
        .replace('Ḷ', ".L")
}

#[cfg(test)]
mod tests {
    use super::*;

    struct WordIndex {
        word: String,
        index: usize,
    }

    struct WordLetter {
        word: String,
        letter: String,
    }

    #[test]
    fn test_first_letter() {

        let words = vec![
            WordLetter { word: "anicca".to_string(), letter: "a".to_string() },
            WordLetter { word: "āloka".to_string(), letter: "ā".to_string() }
        ];

        for w in words.iter() {
            assert_eq!(first_letter(&w.word).unwrap(), w.letter);
        }
    }

    #[test]
    fn pali_index() {

        let words = vec![
            WordIndex { word: "anicca".to_string(), index: 0 },
            WordIndex { word: "āloka".to_string(), index: 1 },
            WordIndex { word: "bhūta".to_string(), index: 3 },
            WordIndex { word: "khādaka".to_string(), index: 20 },
            WordIndex { word: "nivaraṇa".to_string(), index: 25 },
            WordIndex { word: "thālipāka".to_string(), index: 36 },
        ];

        for w in words.iter() {
            assert_eq!(romanized_pali_letter_index(&w.word), w.index);
        }
    }
}
