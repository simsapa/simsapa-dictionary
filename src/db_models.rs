use super::db_schema::*;

#[derive(Serialize, Queryable, QueryableByName)]
#[table_name="authors"]
pub struct DbAuthor {
    pub id:         i32,
    pub uid:        String,
    pub blurb:      String,
    pub long_name:  String,
    pub short_name: String,
}

#[derive(Insertable)]
#[table_name="authors"]
pub struct NewAuthor<'a> {
    pub uid:         &'a str,
    pub blurb:       &'a str,
    pub long_name:   &'a str,
    pub short_name:  &'a str,
}

#[derive(Serialize, Queryable, QueryableByName)]
#[table_name="root_texts"]
pub struct DbRootText {
    pub id:               i32,
    pub author_id:        i32,
    pub uid:              String,
    pub acronym:          String,
    pub volpage:          String,
    pub title:            String,
    pub content_language: String,
    pub content_plain:    String,
    pub content_html:     String,
}

#[derive(Insertable)]
#[table_name="root_texts"]
pub struct NewRootText<'a> {
    pub author_id:        &'a i32,
    pub uid:              &'a str,
    pub acronym:          &'a str,
    pub volpage:          &'a str,
    pub title:            &'a str,
    pub content_language: &'a str,
    pub content_plain:    &'a str,
    pub content_html:     &'a str,
}

#[derive(Serialize, Queryable, QueryableByName)]
#[table_name="fts_root_texts"]
pub struct FtsRootText {
    pub rowid:         i32,
    pub content_plain: String,
}

#[derive(Serialize, Queryable, QueryableByName)]
#[table_name="translated_texts"]
pub struct DbTranslatedText {
    pub id:               i32,
    pub author_id:        i32,
    pub uid:              String,
    pub acronym:          String,
    pub volpage:          String,
    pub title:            String,
    pub root_title:       String,
    pub content_language: String,
    pub content_plain:    String,
    pub content_html:     String,
}

#[derive(Insertable)]
#[table_name="translated_texts"]
pub struct NewTranslatedText<'a> {
    pub author_id:        &'a i32,
    pub uid:              &'a str,
    pub acronym:          &'a str,
    pub volpage:          &'a str,
    pub title:            &'a str,
    pub root_title:       &'a str,
    pub content_language: &'a str,
    pub content_plain:    &'a str,
    pub content_html:     &'a str,
}

#[derive(Serialize, Queryable, QueryableByName)]
#[table_name="fts_translated_texts"]
pub struct FtsTranslatedText {
    pub rowid:         i32,
    pub content_plain: String,
}

#[derive(Serialize, Queryable, QueryableByName)]
#[table_name = "dictionaries"]
pub struct DbDictionary {
    pub id:    i32,
    pub label: String,
    pub title: String,
}

#[derive(Insertable)]
#[table_name="dictionaries"]
pub struct NewDictionary<'a> {
    pub label: &'a str,
    pub title: &'a str,
}

#[derive(Serialize, Queryable, QueryableByName, Clone)]
#[table_name = "dict_words"]
pub struct DbDictWord {
    pub id:              i32,
    pub dictionary_id:   i32,
    pub word:            String,
    pub word_nom_sg:     String,
    pub inflections:     String,
    pub phonetic:        String,
    pub transliteration: String,
    pub url_id:          String,
}

#[derive(Insertable)]
#[table_name="dict_words"]
pub struct NewDictWord<'a> {
    pub dictionary_id:    &'a i32,
    pub word:             &'a str,
    pub word_nom_sg:      &'a str,
    pub inflections:      &'a str,
    pub phonetic:         &'a str,
    pub transliteration:  &'a str,
    pub url_id:           &'a str,
}

#[derive(Serialize, Queryable, QueryableByName)]
#[table_name = "meanings"]
pub struct DbMeaning {
    pub id: i32,
    pub dict_word_id: i32,
    pub meaning_order: i32,
    pub definition_md: String,
    pub summary: String,
    pub synonyms: String,
    pub antonyms: String,
    pub homonyms: String,
    pub also_written_as: String,
    pub see_also: String,
    pub comment: String,
    pub is_root: bool,
    pub root_language: String,
    pub root_groups: String,
    pub root_sign: String,
    pub root_numbered_group: String,
}

#[derive(Insertable)]
#[table_name="meanings"]
pub struct NewMeaning<'a> {
    pub dict_word_id: &'a i32,
    pub meaning_order: &'a i32,
    pub definition_md: &'a str,
    pub summary: &'a str,
    pub synonyms: &'a str,
    pub antonyms: &'a str,
    pub homonyms: &'a str,
    pub also_written_as: &'a str,
    pub see_also: &'a str,
    pub comment: &'a str,
    pub is_root: &'a bool,
    pub root_language: &'a str,
    pub root_groups: &'a str,
    pub root_sign: &'a str,
    pub root_numbered_group: &'a str,
}

#[derive(Serialize, Queryable, QueryableByName)]
#[table_name="fts_meanings"]
pub struct FtsMeaning {
    pub rowid:         i32,
    pub definition_md: String,
    pub summary:       String,
}

#[derive(Serialize, Queryable, QueryableByName)]
#[table_name = "grammars"]
pub struct DbGrammar {
    pub id: i32,
    pub meaning_id: i32,
    pub roots: String,
    pub prefix_and_root: String,
    pub construction: String,
    pub base_construction: String,
    pub compound_type: String,
    pub compound_construction: String,
    pub comment: String,
    pub speech: String,
    pub case: String,
    pub num: String,
    pub gender: String,
    pub person: String,
    pub voice: String,
    pub object: String,
    pub transitive: String,
    pub negative: String,
    pub verb: String,
}

#[derive(Insertable)]
#[table_name="grammars"]
pub struct NewGrammar<'a> {
    pub meaning_id: &'a i32,
    pub roots: &'a str,
    pub prefix_and_root: &'a str,
    pub construction: &'a str,
    pub base_construction: &'a str,
    pub compound_type: &'a str,
    pub compound_construction: &'a str,
    pub comment: &'a str,
    pub speech: &'a str,
    pub case: &'a str,
    pub num: &'a str,
    pub gender: &'a str,
    pub person: &'a str,
    pub voice: &'a str,
    pub object: &'a str,
    pub transitive: &'a str,
    pub negative: &'a str,
    pub verb: &'a str,
}

#[derive(Serialize, Queryable, QueryableByName)]
#[table_name = "examples"]
pub struct DbExample {
    pub id: i32,
    pub meaning_id: i32,
    pub source_ref: String,
    pub source_title: String,
    pub text_md: String,
    pub translation_md: String,
}

#[derive(Insertable)]
#[table_name="examples"]
pub struct NewExample<'a> {
    pub meaning_id: &'a i32,
    pub source_ref: &'a str,
    pub source_title: &'a str,
    pub text_md: &'a str,
    pub translation_md: &'a str,
}
