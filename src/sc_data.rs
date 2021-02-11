use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};
use regex::Regex;

use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;

use crate::db_schema;
use crate::db_models::{DbAuthor, NewAuthor, DbTranslatedText, NewTranslatedText, DbRootText, NewRootText};

use walkdir::{DirEntry, WalkDir};

use scraper::{Html, Selector};

// NOTE Run 'make po2json' at least once to produce the JSON from the PO files.

pub fn process_suttacentral_root_po_texts_to_sqlite(
    sc_data_path: &Path,
    po_text_json_path: &Path,
    sqlite_db_path: &Path
    ) -> Result<(), Box<dyn Error>>
{
    info!("process_suttacentral_root_texts()");

    // === Database connection ===

    let conn = SqliteConnection::establish(sqlite_db_path.to_str().unwrap())
        .expect("Error connecting to database.");

    let mut authors: Vec<DbAuthor> = vec![];
    let mut root_texts: Vec<DbRootText> = vec![];
    let mut translated_texts: Vec<DbTranslatedText> = vec![];

    info!{"\n=== Begin processing JSON data. ===\n"};

    // Deserialize data from JSON

    let structure_suttas: Vec<StructureSutta>;
    {
        let p = sc_data_path.join(PathBuf::from("structure/sutta.json"));
        let mut s = fs::read_to_string(p).unwrap();

        // In sutta.json, numerals are sometimes int type, sometimes quoted as
        // string. Fixing this here, every numeral will quoted as a string.
        s = Regex::new("(\"number_in_vagga\": )([0-9]+),").unwrap().replace_all(&s, "$1\"$2\",").to_string();
        s = Regex::new("(\"vagga_number\": )([0-9]+),").unwrap().replace_all(&s, "$1\"$2\",").to_string();

        structure_suttas = serde_json::from_str(&s).unwrap();
    }

    let authors_editions: Vec<AuthorEdition>;
    {
        let p = sc_data_path.join(PathBuf::from("additional-info/author_edition.json"));
        let s = fs::read_to_string(p).unwrap();
        authors_editions = serde_json::from_str(&s).unwrap();
    }

    // folder is pli-en, one half of the PO pairs is Pali, the other is the translated language
    let translated_lang = String::from("en");

    let pli_en_path = po_text_json_path.join(PathBuf::from("pli-en"));

    let po_divisions: Vec<String> = vec![
        String::from("an"),// an/an01 -- an/an11
        String::from("dn"),
        String::from("kn"),// kn/thag, kn/thig
        String::from("mn"),
        //String::from("pli-tv"),// FIXME complex sub-foldes
        String::from("sn"),// sn/sn01 -- sn/sn56
    ];

    info!{"\n=== Parsing the PO texts (as JSON) into division books. ===\n"};

    for division in po_divisions.iter() {

        // Construct a DivisionBook to hold info for each PO file, and find the
        // info.json which has metadata about the division.
        //
        // Walk the folders recurively. info.json is at the root of the division
        // folder.
        //
        // Derive sub_section from the folder:
        //
        // an/an01 -> an01
        //
        // Derive vagga_name from the file name:
        //
        // an/an01/an1.001-10.json -> an1.001-10

        #[allow(dead_code)]
        struct DivisionBook {
            translated_lang: String,
            division: String,
            sub_section: Option<String>,
            vagga_name: String,
            po_msgs: Vec<PoMessage>,
            json_content: String,
            file_path: PathBuf,
            file_name: String,
        }

        let mut division_info: Vec<PoAuthorInfo> = vec![];
        let mut division_books: Vec<DivisionBook> = vec![];

        let folder = pli_en_path.join(PathBuf::from(division));

        info!("Walking '{:?}'", folder);

        let walker = WalkDir::new(&folder).into_iter();
        for entry in walker.filter_entry(|e| !is_hidden(e)) {
            let entry = entry.unwrap();
            if entry.path().is_dir() {
                info!("Skipping dir entry '{}'", entry.path().to_str().unwrap());
                continue;
            }

            info!("{}", entry.file_name().to_str().unwrap());

            let json_content = fs::read_to_string(entry.path().to_str().unwrap()).unwrap();

            // If the current file is info.json, deserialize it to PoAuthorInfo.

            if entry.file_name().to_str().unwrap() == "info.json" {
                division_info = serde_json::from_str(&json_content).unwrap();
            } else {

                // Else:
                // - deserialize to Gettext PO messages
                // - obtain sub_section
                // - obtain vagga_name

                // Fixing a null value in the JSON.
                // if entry.path().ends_with("pli-tv-bu-vb/en/pli-tv-bu-vb-pc/pli-tv-bu-vb-pc46.json") {
                //     json_content = json_content.replace("\"msgctxt\": null,",
                //                                         "\"msgctxt\": \"pli-tv-bu-vb-pc46:pts-cs2.2.1\",");
                // }

                // NOTE The `comment` field is sometimes string-typed with a 'HTML:' prefix. We
                // strip that in the result.

                let po_msgs = match serde_json::from_str::<Vec<PoMessage>>(&json_content) {
                    Ok(x) => {
                        let msgs: Vec<PoMessage> = x.iter().map(|i| {
                            let mut a = i.clone();
                            a.comment = a.comment.trim_start_matches("HTML: ").to_string();
                            a
                        }).collect();
                        msgs
                    },

                    Err(_) => {
                        panic!{"Can't read PoMessages."};
                    }
                };

                let sub_section: Option<String>;
                if entry.path()        // path: an/an01/an1.001-10.json
                    .parent().unwrap() // parent: an/an01
                    .ends_with(Path::new(division))
                {

                    // For DN and MN texts there are no subsections, unlike in
                    // the AN and SN.
                    sub_section = None;

                } else {

                    // Parse out the subsection for AN, SN, and Vinaya texts.

                    // an
                    let above_sec = entry.path() // an/an01/an1.001-10.json
                        .parent().unwrap()       // an/an01
                        .parent().unwrap();      // an

                    // an01
                    let sub_sec = entry.path()   // an/an01/an1.001-10.json
                        .parent().unwrap()       // an/an01
                        .strip_prefix(&above_sec).unwrap(); // an01

                    sub_section = Some(sub_sec.to_str().unwrap().to_string());

                }

                let book = DivisionBook {
                    // en
                    translated_lang: translated_lang.clone(),
                    // an
                    division: division.to_string(),
                    // an01
                    sub_section,
                    // an1.001-10
                    vagga_name: entry.path().file_stem().unwrap().to_str().unwrap().to_string(),
                    po_msgs,
                    json_content,
                    file_path: entry.path().to_path_buf(),
                    file_name: entry.file_name().to_str().unwrap().to_string(),
                };

                division_books.push(book);
            }

        }

        info!("\nProcessing division authors.\n");

        // Root author

        let root_author: DbAuthor;
        {
            let uid = &division_info.iter()
                .find(|i| i.msgid == "root_author_uid").unwrap()
                .msgstr.trim();

            let blurb = &division_info.iter()
                .find(|i| i.msgid == "root_author_blurb").unwrap()
                .msgstr.trim();

            let a = match authors_editions.iter().find(|i| i.uid == *uid) {
                Some(x) => x,
                None => {
                    panic!{"Can't find author uid: '{}'", uid};
                }
            };

            let new_author = NewAuthor {
                uid:         &uid,
                blurb:       &blurb,
                long_name:   &a.long_name,
                short_name:  &a.short_name,
            };

            root_author = match get_author(&conn, uid) {
                Some(author) => author,
                None => {
                    {
                        info!("Inserting author: {}", new_author.uid);
                        let author: DbAuthor = create_new_author(&conn, &new_author);
                        authors.push(author);
                    }
                    get_author(&conn, uid).unwrap()
                }
            };
        }

        // Translation author

        let translated_author: DbAuthor;
        {
            let uid = &division_info.iter()
                .find(|i| i.msgid == "translation_author_uid").unwrap()
                .msgstr.trim();

            let blurb = &division_info.iter()
                .find(|i| i.msgid == "translation_author_blurb").unwrap()
                .msgstr.trim();

            let a = match authors_editions.iter().find(|i| i.uid == *uid) {
                Some(x) => x,
                None => {
                    panic!{"Can't find author uid: '{}'", uid};
                }
            };

            let new_author = NewAuthor {
                uid:         &uid,
                blurb:       &blurb,
                long_name:   &a.long_name,
                short_name:  &a.short_name,
            };

            translated_author = match get_author(&conn, uid) {
                Some(author) => author,
                None => {
                    {
                        info!("Inserting author: {}", new_author.uid);
                        let author: DbAuthor = create_new_author(&conn, &new_author);
                        authors.push(author);
                    }
                    get_author(&conn, uid).unwrap()
                }
            };
        }

        info!("\nProcessing division books into RootText and TranslationText.\n");

        for book in division_books.iter() {

            // Obtain the necessary information for a text.
            //
            // uid will be division/div_number/lang/author such as:
            // - dn/1/en/bodhi
            // - an/1.001-10/en/bodhi
            //
            // Page titles can be identified by finding the PO messages which
            // start with the comment '</p><h1>'.
            //
            // We will split the PO messages to RootText and TranslationText using
            // msgid and msgstr fields respectively.
            //
            // PO messages with msgctxt field are the main sutta content.
            //
            // The comment field has to be prepended to the message text to
            // build the HTML content.
            //
            // Then we strip the HTML to plain text for better fulltext search.

            // --- Obtain uid ---

            // an
            let division = Path::new(&book.division).parent().unwrap().to_str().unwrap();
            // 1.1-10 from an1.001-10.json
            // np10     from pli-tv-bu-vb-np10.json
            let with_dash = format!{"{}-", division};
            let mut div_number: String = if book.vagga_name.contains(&with_dash) {
                book.vagga_name.replace(&with_dash, "").to_string()
            } else {
                book.vagga_name.replace(division, "").to_string()
            };

            // remove leading zeros!
            div_number = div_number.trim_start_matches('0').to_string();
            div_number = Regex::new(r"([\.-])0+").unwrap().replace_all(&div_number, "$1").to_string();

            // pli
            let root_lang = "pli";

            // dn/1/pli/ms
            let root_text_uid = format!{"{}/{}/{}/{}",
                                        division,
                                        div_number,
                                        root_lang,
                                        root_author.uid.clone()};

            // dn/1/en/bodhi
            let translated_text_uid = format!{"{}/{}/{}/{}",
                                              division,
                                              div_number,
                                              translated_lang.clone(),
                                              translated_author.uid.clone()};

            // --- Obtain acronym and volpage ---

            let mut acronym = "".to_string();
            let mut volpage = "".to_string();

            // NOTE Ignoring SN ranges. sutta.json has AN suttas with uid in a
            // range (an4.123-124) but SN suttas broken down to individuals
            // within a range (sn45.98, sn45.99, ...)
            //
            // The chapter will be read and stored as one entry, such as
            // sn45.098-102.json

            // NOTE Ignoring pli-tv... ids, because sutta.json uses the pali
            // names as uid.

            // mn007
            // an2.087-97
            // an4.058
            let re = Regex::new("^[a-z]+[0-9]+(?:\\.[0-9]+)?\\.").unwrap();
            if let Some(m) = re.find(&book.file_name) {
                // dn1, sn22.1, mn077
                let a = &book.file_name[m.start()..m.end()-1];
                // remove leading zeros, leaving:
                // mn77
                // an2.87-97
                let vol_uid = Regex::new("([a-z\\.]+)0*([1-9][0-9]*)").unwrap().replace_all(a, "$1$2");
                if let Some(s) = structure_suttas.iter().find(|i| i.uid == vol_uid) {
                    acronym = s.acronym.clone();
                    volpage = s.volpage.clone();
                }
            }

            // --- Obtain titles ---

            // Also strip prefixed number from titles.

            let title_re = Regex::new("^[0-9 \\.]+").unwrap();

            let title_pali = {
                let a = match book.po_msgs.iter().find(|i| i.comment.starts_with("</p><h1>")) {
                    Some(x) => x,
                    None => {
                        panic!{"Can't find title in:\n{:#?}", &book.po_msgs};
                    }
                };
                title_re.replace(&a.msgid, "").to_string()
            };

            let title_translated = {
                let s = &book.po_msgs.iter()
                    .find(|i| i.comment.starts_with("</p><h1>")).unwrap()
                    .msgstr;
                title_re.replace(&s, "").to_string()
            };

            // --- Build page content: root and translated, html and plain text ---

            let mut root_html = String::new();
            let mut translated_html = String::new();

            for i in book.po_msgs.iter().filter(|i| !i.msgctxt.is_empty()) {
                // some comments contain variation info

                /*
                #. </p><p>
                #. <a class="sc" id="sc2"></a>
                #. VAR: Na hi nūna → nahanūna (bj, s1, s2, km, pts1) | naha nūna (s3)
                msgctxt "an4.67:2.1"
                msgid ""
                "“Na hi nūna so, bhikkhave, bhikkhu cattāri ahirājakulāni mettena cittena "
                "phari."
                msgstr ""
                "“Mendicants, that monk mustn’t have spread a mind of love to the four royal "
                "snake families."
                 */

                // rg 'VAR[^:]' shows there is a typo as VARL in
                // pli-tv-bu-vb/pli-tv-bu-vb-pc/pli-tv-bu-vb-pc48.po

                if i.comment.contains("VAR") {

                    let re = Regex::new("VAR[:L].*").unwrap();
                    let mut variation = String::new();
                    for cap in re.captures_iter(&i.comment) {
                        variation.push_str(&cap[0]);
                    }

                    // only adding variation info to the root text.
                    root_html.push_str(&format!{"</p><p class=\"variation\">{}</p><p>", variation});

                    let s = re.replace_all(&i.comment, "").to_string();

                    root_html.push_str(&s);
                    translated_html.push_str(&s);

                } else {

                    root_html.push_str(&i.comment);
                    translated_html.push_str(&i.comment);

                }

                // append space to separate sentences
                root_html.push_str(&format!{"{} ", &i.msgid});
                translated_html.push_str(&format!{"{} ", &i.msgstr});

            }

            // Final closing tags seem to be missing.
            root_html.push_str("</p></article></section>");
            translated_html.push_str("</p></article></section>");

            let mut root_plain = html2text::from_read(root_html.as_bytes(), 100);
            let mut translated_plain = html2text::from_read(translated_html.as_bytes(), 100);

            // strip markdown # and > from plain text content
            {
                // at the beginning of the text
                let re = Regex::new("^[#> ]+").unwrap();
                root_plain = re.replace_all(&root_plain, "").to_string();
                translated_plain = re.replace_all(&translated_plain, "").to_string();
            }
            {
                // in the middle of the text
                let re = Regex::new("\n[#> ]+").unwrap();
                root_plain = re.replace_all(&root_plain, "\n").to_string();
                translated_plain = re.replace_all(&translated_plain, "\n").to_string();
            }

            // --- Construct records for database ---

            let new_root_text = NewRootText {
                author_id:        &root_author.id,
                uid:              &root_text_uid,
                acronym:          &acronym,
                volpage:          &volpage,
                title:            &title_pali,
                content_language: &root_lang,
                content_html:     &root_html,
                content_plain:    &root_plain,
            };

            let new_translated_text = NewTranslatedText {
                author_id:        &translated_author.id,
                uid:              &translated_text_uid,
                acronym:          &acronym,
                volpage:          &volpage,
                title:            &title_translated,
                root_title:       &title_pali,
                content_language: &translated_lang,
                content_html:     &translated_html,
                content_plain:    &translated_plain,
            };

            match get_root_text(&conn, &root_text_uid) {
                Some(_text) => warn!("Already exists: {}", translated_text_uid),
                None => {
                    info!("Inserting root text: {}", new_root_text.uid);
                    let text = create_new_root_text(&conn, &new_root_text);
                    root_texts.push(text);
                }
            }

            match get_translated_text(&conn, &translated_text_uid) {
                Some(_text) => warn!("Already exists: {}", translated_text_uid),
                None => {
                    info!("Inserting translated text: {}", new_translated_text.uid);
                    let text = create_new_translated_text(&conn, &new_translated_text);
                    translated_texts.push(text);
                }
            }
        }
    }

    info!{"\n=== End of processing PO texts data. ===\n"};

    info!{"Created Authors: {}", authors.len()};

    info!{"Created RootTexts: {}", root_texts.len()};

    info!{"Created TranslatedTexts: {}", translated_texts.len()};

    Ok(())
}

pub fn process_suttacentral_html_texts_to_sqlite(
    sc_data_path: &Path,
    sqlite_db_path: &Path
    ) -> Result<(), Box<dyn Error>>
{
    info!("process_suttacentral_html_texts()");

    // === Database connection ===

    let conn = SqliteConnection::establish(sqlite_db_path.to_str().unwrap())
        .expect("Error connecting to database.");

    let mut authors: Vec<DbAuthor> = vec![];
    let mut translated_texts: Vec<DbTranslatedText> = vec![];

    let authors_editions: Vec<AuthorEdition>;
    {
        let p = sc_data_path.join(PathBuf::from("additional-info/author_edition.json"));
        info!{"{:#?}", p};
        let s = fs::read_to_string(p).unwrap();
        authors_editions = serde_json::from_str(&s).unwrap();
    }

    info!{"\n=== Being processing HTML texts data. ===\n"};

    let html_root = sc_data_path.join(Path::new("html_text"));
    let folder = html_root.join(Path::new("en/pli/sutta/an"));

    info!("Walking '{:?}'", folder);

    let walker = WalkDir::new(&folder).into_iter();
    for entry in walker.filter_entry(|e| !is_hidden(e)) {
        let entry = entry.unwrap();
        let entry_path = entry.path().to_str().unwrap();
        let entry_file_name = entry.file_name().to_str().unwrap();

        if entry.path().is_dir() {
            info!("Skipping dir entry '{}'", entry.path().to_str().unwrap());
            continue;
        }
        if !entry_file_name.ends_with(".html") {
            continue;
        }

        info!("Processing: {}", entry_path);

        let content = fs::read_to_string(entry_path).unwrap();
        let doc = Html::parse_document(&content);

        let body_html;
        {
            let a = content.find("<body>").unwrap() + 6;
            let b = content.find("</body>").unwrap();
            body_html = content[a..b].trim().to_string();
        }

        let mut body_plain = html2text::from_read(body_html.as_bytes(), 100);

        // strip markdown # and > from plain text content

        // at the beginning of the text
        body_plain = Regex::new("^[#> ]+").unwrap().replace_all(&body_plain, "").to_string();

        // in the middle of the text
        body_plain = Regex::new("\n[#> ]+").unwrap().replace_all(&body_plain, "\n").to_string();

        // name of the the first folder after html_root
        // sc-data/html_text/en/pli/sutta/kn
        // en
        let translated_lang;
        {
            // No slash at the beginning:
            // en/pli/sutta/mn
            let folder = entry_path
                .trim_start_matches(html_root.to_str().unwrap())
                .trim_start_matches('/');
            let a = folder.find('/').unwrap();
            translated_lang = folder[0..a].to_string();
        }

        let translated_author: DbAuthor;
        {

            // The only <meta> info stored in the HTML is the author. Present as
            // 'author' or 'content' attribute:
            //
            // <meta author="Bhikkhu Bodhi">
            // <meta name="author" content="Bhikkhu Bodhi">

            /*
            $ cd sc-data/html_text
            $ rg '<meta' . | sed 's/<meta /\n&/g' | rg -ve '(author|charset)=' | rg '<meta'
            <meta name="author" content="Bhikkhu Bodhi">
            <meta name="author" content="Bhikkhu Bodhi">
            <meta name="author" content="T.W. Rhys Davids"></head><body><div id="text" lang="en">
            ...
            <meta>

            $ rg 'name="author" content="'
            en/pli/sutta/dn/dn2.html
            5:<meta name="author" content="Bhikkhu Bodhi">
            en/pli/sutta/dn/dn1.html
            5:<meta name="author" content="Bhikkhu Bodhi">
            en/pli/sutta/dn/dn3.html
            1:<!doctype html><html><head><title></title><meta charset="UTF-8"><meta name="author" content="T.W. Rhys Davids"></head><body><div id="text" lang="en">
             */

            let mut long_name = String::new();
            {
                let selector = Selector::parse("meta").unwrap();

                for element in doc.select(&selector) {
                    if let Some(name) = element.value().attr("author") {
                        long_name.push_str(name.trim());
                    }
                }
            }

            if long_name.is_empty() {
                let selector = Selector::parse(r#"meta[name="author"]"#).unwrap();

                for element in doc.select(&selector) {
                    if let Some(name) = element.value().attr("content") {
                        long_name.push_str(name.trim());
                    }
                }
            }

            if long_name.is_empty() {
                warn!{"Can't obtain author's name from: {}", entry_path};
                long_name.push_str("n/a");
            }

            // Differences in author name b/w the <meta> tag and the JSON data

            if long_name == "Amaravati Sangha" {
                long_name = "The Amaravati Sangha".to_string();
            } else if long_name == "IB Horner" {
                long_name = "I.B. Horner".to_string();
            }

            translated_author = match get_author_by_long_name(&conn, &long_name) {
                Some(author) => author,
                None => {
                    {
                        let a = match authors_editions.iter()
                            .find(|i| i.long_name == *long_name)
                        {
                            Some(author) => author,
                            None => panic!{"Can't find author in the JSON: '{}'", long_name},
                        };

                        let new_author = NewAuthor {
                            uid:         &a.uid,
                            blurb:       "",
                            long_name:   &a.long_name,
                            short_name:  &a.short_name,
                        };

                        info!("Inserting author: {}", new_author.uid);
                        let author: DbAuthor = create_new_author(&conn, &new_author);
                        authors.push(author);
                    }
                    get_author_by_long_name(&conn, &long_name).unwrap()
                }
            };
        }

        // an
        let mut division = String::new();
        if entry_path.contains("/pli/sutta/") || entry_path.contains("/pli/abhidhamma/")
        {
            // $ ls html_text/en/pli/sutta
            // an  dn  kn  mn  sn
            // $ ls html_text/en/pli/abhidhamma
            // ds  kv  patthana  vb

            let re = Regex::new("/pli/(sutta|abhidhamma)/([^/]+)/").unwrap();
            for cap in re.captures_iter(entry_path) {
                division = cap[2].to_string();
            }

        } else if entry_path.contains("/pli/vinaya/") {

            division = entry.path().file_stem().unwrap().to_str().unwrap().to_string();

        }

        if division.is_empty() {
            panic!{"Can't obtain division from: {}", entry_path};
        }

        let vagga_name = entry.path().file_stem().unwrap().to_str().unwrap().to_string();
        // 1.001-10 from an1.001-10.json
        // np10     from pli-tv-bu-vb-np10.json
        let with_dash = format!{"{}-", division};
        let mut div_number: String = if vagga_name.contains(&with_dash) {
            vagga_name.replace(&with_dash, "").to_string()
        } else {
            vagga_name.replace(&division, "").to_string()
        };

        // remove leading zeros!
        div_number = div_number.trim_start_matches('0').to_string();
        div_number = Regex::new(r"([\.-])0+").unwrap().replace_all(&div_number, "$1").to_string();

        // dn/1/en/bodhi
        let translated_text_uid = format!{"{}/{}/{}/{}",
                                          division,
                                          div_number,
                                          translated_lang,
                                          translated_author.uid.clone()};

        let mut pali_title = String::new();
        {
            // Not safe to assume p.division
            //let selector = Selector::parse("section.sutta div.hgroup p").unwrap();
            let selector = Selector::parse("header ul li.division[lang='pli']").unwrap();

            for element in doc.select(&selector) {
                for i in element.text().collect::<Vec<_>>().iter() {
                    if !i.trim().is_empty() {
                        if !pali_title.is_empty() {
                            pali_title.push_str(" -- ");
                        }
                        pali_title.push_str(i.trim())
                    }
                }
            }
        }

        if pali_title.is_empty() {
            warn!{"Can't obtain Pali title for: {}", entry_path};
        }

        let mut translated_title = String::new();
        {
            //let selector = Selector::parse("section.sutta div.hgroup h1").unwrap();
            let selector = Selector::parse("header ul li:last-child").unwrap();

            for element in doc.select(&selector) {
                for i in element.text().collect::<Vec<_>>().iter() {
                    if !i.trim().is_empty() {
                        if !translated_title.is_empty() {
                            translated_title.push_str(" -- ");
                        }
                        translated_title.push_str(i.trim())
                    }
                }
            }
        }

        // strip prefixed number from titles
        {
            let re = Regex::new("^[0-9 \\.]+").unwrap();
            pali_title = re.replace(&pali_title, "").to_string();
            translated_title = re.replace(&translated_title, "").to_string();
        }

        if translated_title.is_empty() {
            warn!{"Can't obtain translated title for: {}", entry_path};
        }

        let mut acronym = String::new();
        let mut volpage = String::new();

        // Obtain acronym and volpage from a root text corresponding to the
        // translated text.

        let root_text_uid = format!{"{}/{}/{}/{}",
                                    division,
                                    div_number,
                                    "pli",
                                    "ms"};

        match get_root_text(&conn, &root_text_uid) {
            Some(text) => {
                acronym = text.acronym;
                volpage = text.volpage;
            },
            None => {
                warn!("Can't obtain acronym and volpage text with uid '{}', for path: {}",
                      root_text_uid,
                      entry_path);
            },
        }

        let new_translated_text = NewTranslatedText {
            author_id:        &translated_author.id,
            uid:              &translated_text_uid,
            acronym:          &acronym,
            volpage:          &volpage,
            title:            &translated_title,
            root_title:       &pali_title,
            content_language: &translated_lang,
            content_html:     &body_html,
            content_plain:    &body_plain,
        };

        match get_translated_text(&conn, &translated_text_uid) {
            Some(_text) => warn!("Already exists: {}", translated_text_uid),
            None => {
                info!("Inserting translated text: {}", new_translated_text.uid);
                let text = create_new_translated_text(&conn, &new_translated_text);
                translated_texts.push(text);
            }
        }
    }

    info!{"\n=== End of processing HTML texts data. ===\n"};

    info!{"Created Authors: {}", authors.len()};

    info!{"Created TranslatedTexts: {}", translated_texts.len()};

    Ok(())
}

fn is_hidden(entry: &DirEntry) -> bool {
    entry.file_name()
        .to_str()
        .map(|s| s.starts_with('.'))
        .unwrap_or(false)
}

fn get_author(conn: &SqliteConnection,
                  author_uid: &str)
                  -> Option<DbAuthor>
{
    use db_schema::authors::dsl::*;
    //use schema::authors;

    let mut items = authors
        .filter(uid.eq(author_uid))
        .load::<DbAuthor>(conn)
        .expect("Error loading the author.");

    items.pop()
}

fn get_author_by_long_name(conn: &SqliteConnection,
                               author_long_name: &str)
                               -> Option<DbAuthor>
{
    use db_schema::authors::dsl::*;

    let mut items = authors
        .filter(long_name.eq(author_long_name))
        .load::<DbAuthor>(conn)
        .expect("Error loading the author.");

    items.pop()
}

fn create_new_author<'a>(conn: &SqliteConnection,
                         new_author: &'a NewAuthor)
                         -> DbAuthor
{
    use db_schema::authors::dsl::*;
    use db_schema::authors;

    diesel::insert_into(authors::table)
        .values(new_author)
        .execute(conn)
        .expect("Error inserting the author.");

    let mut items = authors
        .filter(uid.eq(new_author.uid))
        .load::<DbAuthor>(conn)
        .expect("Error loading the inserted author.");

    items.pop().unwrap()
}

fn get_root_text(conn: &SqliteConnection,
                     text_uid: &str) -> Option<DbRootText>
{
    use db_schema::root_texts::dsl::*;
    //use db_schema::root_texts;

    let mut items = root_texts
        .filter(uid.eq(text_uid))
        .load::<DbRootText>(conn)
        .expect("Error loading the root text.");

    items.pop()
}

fn create_new_root_text<'a>(conn: &SqliteConnection,
                            new_root_text: &'a NewRootText)
                            -> DbRootText
{
    use db_schema::root_texts::dsl::*;
    use db_schema::root_texts;

    diesel::insert_into(root_texts::table)
        .values(new_root_text)
        .execute(conn)
        .expect("Error inserting the root text.");

    let mut items = root_texts
        .filter(uid.eq(new_root_text.uid))
        .load::<DbRootText>(conn)
        .expect("Error loading the inserted root text.");

    items.pop().unwrap()
}

fn get_translated_text(conn: &SqliteConnection,
                           text_uid: &str)
                           -> Option<DbTranslatedText>
{
    use db_schema::translated_texts::dsl::*;
    //use schema::translated_texts;

    let mut items = translated_texts
        .filter(uid.eq(text_uid))
        .load::<DbTranslatedText>(conn)
        .expect("Error loading the translated text.");

    items.pop()
}

fn create_new_translated_text<'a>(conn: &SqliteConnection,
                                  new_translated_text: &'a NewTranslatedText)
                                  -> DbTranslatedText
{
    use db_schema::translated_texts::dsl::*;
    use db_schema::translated_texts;

    diesel::insert_into(translated_texts::table)
        .values(new_translated_text)
        .execute(conn)
        .expect("Error inserting the translated text.");

    let mut items = translated_texts
        .filter(uid.eq(new_translated_text.uid))
        .load::<DbTranslatedText>(conn)
        .expect("Error loading the inserted translated text.");

    items.pop().unwrap()
}

/// sc-data/structure/sutta.json
///
/// Ignored fields:
///
/// - "subdivision_uid": "dn",
/// - "vagga_number": 1,
/// - "number_in_vagga": 1,
/// - "biblio_uid": ""
#[derive(Deserialize)]
pub struct StructureSutta {
    pub name: String,
    pub uid: String,
    pub language: String,
    pub acronym: String,
    pub volpage: String,
}

/// sc-data/additional-info/author_edition.json
#[derive(Deserialize, Debug)]
pub struct AuthorEdition {
    #[serde(rename = "type")]
    pub item_type: String,
    pub uid: String,
    pub short_name: String,
    pub long_name: String,
}

/// sc-data/po_text/pli-en/mn/mn001.json
///
/// Ignored fields:
///
/// - "msgid_plural": "",
/// - "msgstr_plural": {},
/// - "obsolete": 0,
/// - "tcomment": "",
/// - "occurrences": [],
/// - "flags": [],
/// - "previous_msgctxt": null,
/// - "previous_msgid": null,
/// - "previous_msgid_plural": null,
#[derive(Debug, Clone, Deserialize)]
pub struct PoMessage {
    pub msgid: String,
    pub msgstr: String,
    pub msgctxt: String,
    pub encoding: String,
    pub comment: String,
    pub linenum: i32,
}

/// sc-data/po_text/mn/info.json
///
/// Ignored fields:
///
/// - "msgid_plural": "",
/// - "msgstr_plural": {},
/// - "msgctxt": null,
/// - "obsolete": 0,
/// - "encoding": "utf-8",
/// - "tcomment": "",
/// - "occurrences": [],
/// - "flags": [],
/// - "previous_msgctxt": null,
/// - "previous_msgid": null,
/// - "previous_msgid_plural": null,
/// - "linenum": 17
#[derive(Deserialize)]
pub struct PoAuthorInfo {
    pub msgid: String,
    pub msgstr: String,
    pub comment: String,
}
