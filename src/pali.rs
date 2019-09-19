/*
Pali alphabetical order:

a ā i ī u ū e o ṃ / ṁ k kh g gh ṅ c ch j jh ñ ṭ ṭh ḍ ḍh ṇ t th d dh n p ph b bh m y r l ḷ v s h

A Ā I Ī U Ū E O Ṃ / Ṁ K KH G GH Ṅ C CH J JH Ñ Ṭ ṬH Ḍ ḌH Ṇ T TH D DH N P PH B BH M Y R L Ḷ V S H

Romanized, alphabetical order, Pali only:

a ā b bh c ch d dh ḍ ḍh e g gh h i ī j jh k kh l ḷ m ṃ / ṁ n ṅ ṇ ñ o p ph r s t th ṭ ṭh u ū v y

Romanized alphabetical order, including ISO Basic Latin:

a ā b bh c ch d dh ḍ ḍh e f g gh h i ī j jh k kh l ḷ m ṃ / ṁ n ṅ ṇ ñ o p ph q r s t th ṭ ṭh u ū v w x y z

*/

// not including ṁ
const ROMANIZED_PALI_ALPHABET: &str = "a ā b bh c ch d dh ḍ ḍh e g gh h i ī j jh k kh l ḷ m ṃ n ṅ ṇ ñ o p ph r s t th ṭ ṭh u ū v y";

const RPA_DOUBLES_FIRST: &str = "bh ch dh ḍh gh jh kh ph th ṭh a ā b c d ḍ e g h i ī j k l ḷ m ṃ n ṅ ṇ ñ o p r s t ṭ u ū v y";

pub fn romanized_pali_letter_index(word: &str) -> usize {
    let alphabet: Vec<String> = ROMANIZED_PALI_ALPHABET.split(' ').map(String::from).collect();
    let word_clean = word.trim().to_lowercase();

    match first_letter(&word_clean) {
        Some(letter) => match alphabet.binary_search(&letter) {
            Ok(x) => x,
            Err(_) => alphabet.len(),
        },
        None => alphabet.len()
    }
}

pub fn first_letter(word: &str) -> Option<String> {
    let doubles_first: Vec<&str> = RPA_DOUBLES_FIRST.split(' ').collect();
    let word_clean = word.trim().to_lowercase();

    for i in doubles_first.iter() {
        if word_clean.starts_with(*i) {
            return Some(String::from(*i))
        }
    }

    None
}

