CREATE TABLE `authors` (
	`id`         INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
	`uid`        VARCHAR NOT NULL UNIQUE, --  sujato
	`blurb`      TEXT,                    --  Translated for SuttaCentral by Sujato Bhikkhu
	`long_name`  VARCHAR,                 --  Sujato Bhikkhu
	`short_name` VARCHAR                  --  Sujato
);

CREATE TABLE `root_texts` (
	`id`               INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
	`author_id`        INTEGER REFERENCES `authors` (`id`) ON DELETE CASCADE ON UPDATE CASCADE, -- ms
	`uid`              VARCHAR NOT NULL UNIQUE, --  dn1/pli/ms
	`acronym`          VARCHAR,                 --  DN 1
	`volpage`          VARCHAR,                 --  DN i 1
	`title`            VARCHAR,                 --  Brahmajāla
	`content_language` VARCHAR,                 --  pli
	`content_plain`    TEXT,                    --  content in plain text
	`content_html`     TEXT                     --  content in HTML
);

CREATE TABLE `translated_texts` (
	`id`               INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
	`author_id`        INTEGER REFERENCES `authors` (`id`) ON DELETE CASCADE ON UPDATE CASCADE, -- ms
	`uid`              VARCHAR NOT NULL UNIQUE,  --  dn1/en/bodhi
	`acronym`          VARCHAR,                  --  DN 1
	`volpage`          VARCHAR,                  --  DN i 1
	`title`            VARCHAR,                  --  The Root of All Things
	`root_title`       VARCHAR,                  --  Brahmajāla
	`content_language` VARCHAR,                  --  en
	`content_plain`    TEXT,                     --  content in plain text
	`content_html`     TEXT                      --  content in HTML
);

CREATE TABLE `dictionaries` (
	`id`   INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
	`label` VARCHAR NOT NULL UNIQUE,
	`title` VARCHAR NOT NULL
);

CREATE TABLE `dict_words` (
	`id`              INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
	`dictionary_id`   INTEGER NOT NULL REFERENCES `dictionaries` (`id`) ON DELETE CASCADE ON UPDATE CASCADE,
	`word`            VARCHAR NOT NULL,
	`word_nom_sg`     VARCHAR,
	`inflections`     VARCHAR,
	`phonetic`        VARCHAR,
	`transliteration` VARCHAR,
	`url_id`          VARCHAR NOT NULL UNIQUE
);

CREATE TABLE `meanings` (
	`id`                  INTEGER PRIMARY KEY AUTOINCREMENT,
	`dict_word_id`        INTEGER NOT NULL REFERENCES `dict_words` (`id`) ON DELETE CASCADE ON UPDATE CASCADE,
	`meaning_order`       INTEGER,
	`definition_md`       VARCHAR,
	`summary`             VARCHAR,
	`synonyms`            VARCHAR,
	`antonyms`            VARCHAR,
	`homonyms`            VARCHAR,
	`also_written_as`     VARCHAR,
	`see_also`            VARCHAR,
	`comment`             VARCHAR,
	`is_root`             TINYINT(1),
	`root_language`       VARCHAR,
	`root_groups`         VARCHAR,
	`root_sign`           VARCHAR,
	`root_numbered_group` VARCHAR
);

CREATE TABLE `grammars` (
	`id`                    INTEGER PRIMARY KEY AUTOINCREMENT,
	`meaning_id`            INTEGER NOT NULL REFERENCES `meanings` (`id`) ON DELETE CASCADE ON UPDATE CASCADE,
	`roots`                 VARCHAR,
	`prefix_and_root`       VARCHAR,
	`construction`          VARCHAR,
	`base_construction`     VARCHAR,
	`compound_type`         VARCHAR,
	`compound_construction` VARCHAR,
	`comment`               VARCHAR,
	`speech`                VARCHAR,
	`case`                  VARCHAR,
	`num`                   VARCHAR,
	`gender`                VARCHAR,
	`person`                VARCHAR,
	`voice`                 VARCHAR,
	`object`                VARCHAR,
	`transitive`            VARCHAR,
	`negative`              VARCHAR,
	`verb`                  VARCHAR
);

CREATE TABLE `examples` (
	`id`             INTEGER PRIMARY KEY AUTOINCREMENT,
	`meaning_id`     INTEGER NOT NULL REFERENCES `meanings` (`id`) ON DELETE CASCADE ON UPDATE CASCADE,
	`source_ref`     VARCHAR,
	`source_title`   VARCHAR,
	`text_md`        VARCHAR,
	`translation_md` VARCHAR
);

