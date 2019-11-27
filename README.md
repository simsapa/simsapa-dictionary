# Simsapa Dictionary Tool

<!-- markdown-toc start - Don't edit this section. Run M-x markdown-toc-refresh-toc -->
**Table of Contents**

- [Simsapa Dictionary Tool](#simsapa-dictionary-tool)
    - [Applications](#applications)
        - [GoldenDict (Win, Mac OS/X, Linux desktop)](#goldendict-win-mac-osx-linux-desktop)
        - [Kindle Paperwhite](#kindle-paperwhite)
        - [Epub readers](#epub-readers)
        - [Android](#android)
    - [Feedback, corrections, bug reports](#feedback-corrections-bug-reports)
    - [Example dictionary](#example-dictionary)
    - [CLI Options](#cli-options)
    - [Sources](#sources)

<!-- markdown-toc end -->

This tool generates EPUB, MOBI, Stardict and Babylon dictionary files.

**Download Pali - English dictionaries**:  See the [Releases](https://github.com/simsapa/simsapa-dictionary/releases) page.

It includes a set of Pali - English dictionaries, and this tool for Linux, Mac and Windows.

The dictionary source texts are in the [simsapa-dictionary-data](https://github.com/simsapa/simsapa-dictionary-data) repo.

You can download the source text, edit and generate updated EPUB and MOBI files using this tool.

To generate MOBI files, also download [Kindlegen](https://www.amazon.com/gp/feature.html?docId=1000765211) from Amazon (free download).

## Applications

### GoldenDict (Win, Mac OS/X, Linux desktop)

![GoldenDict full text search](docs/goldendict_fulltext_chaya.jpg)

Use the `*-stardict.zip` files, extract them and add the folder to the dictionary list in GoldenDict.

Version 1.5 includes `Search menu > Full text search`, useful for English to Pali searches.

For Windows and OSX, download v1.5 from the [Early Access Builds](https://sourceforge.net/projects/goldendict/files/early%20access%20builds/).

Read mode on the [wiki pages](https://github.com/goldendict/goldendict/wiki).

On Linux, install `goldendict` from your package manager.

### Kindle Paperwhite

![Kindle Paperwhite](docs/kindle_paperwhite.jpg)

Use one of the `*.mobi` files and copy them to your Kindle. It will appear in the *Dictionaries* category.

### Epub readers

The `*.epub` files can be used with ebook readers which read the Epub format.

- iBooks on iOS
- [Calibre](https://calibre-ebook.com/) on desktop

### Android

Search for applications which can open or import `StarDict` format dictionaries.

You might have to copy-paste the link of a `*-stardict.zip` file from the
Releases page, or download it and extract it to a folder where the dictionary
application can find it.

Such apps include:

- [Dict Box - Universal Offline Dictionary](https://play.google.com/store/apps/details?id=com.grandsons.dictsharp)
- [GoldenDict (free)](https://play.google.com/store/apps/details?id=mobi.goldendict.android.free)

## Feedback, corrections, bug reports

Both the tool and the dictionary content has some rough edges.

The dictionary entries can be edited using the files at
[simsapa-dictionary-data](https://github.com/simsapa/simsapa-dictionary-data),
and the dictionary formats re-generated with this tool.

Dictionary corrections or bug reports about the tool are welcome. Open an Issue
here or see my email in the [Cargo.toml](Cargo.toml).

## Example dictionary

See an example dictionary content below. It starts with metadata describing the
dictionary, followed by the word entries. Each word entry starts with a
[TOML](https://github.com/toml-lang/toml) formatted block, followed by the
definition text in Markdown syntax.

Use a text editor such as Notepad++ and copy the example to a file, for example `ncped-example.md`.

The file extension must be `.md`.

Arrange the files in a folder:

```
dictionary/
  kindlegen.exe
  ncped-example.md
  simsapa_dictionary.exe
```

On Windows, drag-and-drop `ncped-example.md` on the `simsapa_dictionary.exe`.

On Linux and Mac, open a terminal in the folder and run `./simsapa_dictionary ./ncped-example.md`.

The default action is to generate a MOBI if `kindlegen.exe` is also present in the folder, otherwise to generate an EPUB.

More options are available, see them with `simsapa_dictionary.exe --help`. An overview is included below.

```
ndped-example.md
```

    --- DICTIONARY METADATA ---
    
    ``` toml
    title = "New Concise Pali - English Dictionary (NCPED)"
    description = "Pali - English"
    creator = "Simsapa Dhamma Reader"
    source = "https://simsapa.github.io"
    cover_path = "default_cover.jpg"
    book_id = "NcpedDictionarySimsapa"
    created_date_human = ""
    created_date_opf = ""
    ```
    
    --- DICTIONARY WORD ENTRIES ---
    
    ``` toml
    dict_label = "NCPED"
    word = "ababa"
    summary = "the name of a hell, or place in Avīci, where one s"
    grammar = ""
    inflections = []
    ```
    
    ababa
    
    masculine the name of a hell, or place in Avīci, where one suffers for an *ababa* of years.
    
    ``` toml
    dict_label = "NCPED"
    word = "abbhantara"
    summary = "interior, internal; being within, included in, amo"
    grammar = ""
    inflections = []
    ```
    
    abbhantara
    
    mfn. & neuter
    
    1. (mfn.) interior, internal; being within, included in, among; belonging to one ‘s house, personal, intimate.
    2. (n.)
       1. intermediate space, interval; the inside, interior.
       2. a measure of length (= 28 hatthas).
    
    ``` toml
    dict_label = "NCPED"
    word = "ajjhokāse"
    summary = "in the open air, in the open."
    grammar = ""
    inflections = []
    ```
    
    ajjhokāse
    
    ind. in the open air, in the open.

## CLI Options

Use the `help` command to discover the command line options, or see [src/cli.yml](src/cli.yml).

```
./simsapa_dictionary help
```

## Sources

The dictionary texts at [simsapa-dictionary-data](https://github.com/simsapa/simsapa-dictionary-data) were created using:

- JSON format dictionaries published at [suttacentral/sc-data](https://github.com/suttacentral/sc-data)
- [Nyanatiloka: Buddhist Dictionary](https://what-buddha-said.net/library/Buddhist.Dictionary/index_dict.n2.htm) published by [what-buddha-said.net](https://what-buddha-said.net/)

