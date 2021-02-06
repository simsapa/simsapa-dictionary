table! {
    authors (id) {
        id -> Integer,
        uid -> Text,
        blurb -> Text,
        long_name -> Text,
        short_name -> Text,
    }
}

table! {
    root_texts (id) {
        id -> Integer,
        author_id -> Integer,
        uid -> Text,
        acronym -> Text,
        volpage -> Text,
        title -> Text,
        content_language -> Text,
        content_plain -> Text,
        content_html -> Text,
    }
}

table! {
    fts_root_texts (rowid) {
        rowid -> Integer,
        content_plain -> Text,
    }
}

table! {
    translated_texts (id) {
        id -> Integer,
        author_id -> Integer,
        uid -> Text,
        acronym -> Text,
        volpage -> Text,
        title -> Text,
        root_title -> Text,
        content_language -> Text,
        content_plain -> Text,
        content_html -> Text,
    }
}

table! {
    fts_translated_texts (rowid) {
        rowid -> Integer,
        content_plain -> Text,
    }
}

table! {
    dictionaries (id) {
        id -> Integer,
        label -> Text,
        title -> Text,
    }
}

table! {
    dict_words (id) {
        id -> Integer,
        dictionary_id -> Integer,
        word -> Text,
        word_nom_sg -> Text,
        inflections -> Text,
        phonetic -> Text,
        transliteration -> Text,
        url_id -> Text,
    }
}

table! {
    fts_meanings (rowid) {
        rowid -> Integer,
        definition_md -> Text,
        summary -> Text,
    }
}

table! {
    meanings (id) {
        id                  -> Integer,
        dict_word_id        -> Integer,
        meaning_order       -> Integer,
        definition_md       -> Text,
        summary             -> Text,
        synonyms            -> Text,
        antonyms            -> Text,
        homonyms            -> Text,
        also_written_as     -> Text,
        see_also            -> Text,
        comment             -> Text,
        is_root             -> Bool,
        root_language       -> Text,
        root_groups         -> Text,
        root_sign           -> Text,
        root_numbered_group -> Text,
    }
}

table! {
    grammars (id) {
        id                    -> Integer,
        meaning_id            -> Integer,
        roots                 -> Text,
        prefix_and_root       -> Text,
        construction          -> Text,
        base_construction     -> Text,
        compound_type         -> Text,
        compound_construction -> Text,
        comment               -> Text,
        speech                -> Text,
        case                  -> Text,
        num                   -> Text,
        gender                -> Text,
        person                -> Text,
        voice                 -> Text,
        object                -> Text,
        transitive            -> Text,
        negative              -> Text,
        verb                  -> Text,
    }
}

table! {
    examples (id) {
        id             -> Integer,
        meaning_id     -> Integer,
        source_ref     -> Text,
        source_title   -> Text,
        text_md        -> Text,
        translation_md -> Text,
    }
}

allow_tables_to_appear_in_same_query!(
    authors,
    root_texts,
    fts_root_texts,
    translated_texts,
    fts_translated_texts,
    dictionaries,
    dict_words,
    fts_meanings,
    meanings,
    grammars,
    examples,
);
