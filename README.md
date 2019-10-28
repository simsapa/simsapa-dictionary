# Simsapa Dictionary Tool

<!-- markdown-toc start - Don't edit this section. Run M-x markdown-toc-refresh-toc -->
**Table of Contents**

- [Simsapa Dictionary Tool](#simsapa-dictionary-tool)
    - [Feedback, corrections, bug reports](#feedback-corrections-bug-reports)
    - [Example dictionary](#example-dictionary)
    - [CLI Options](#cli-options)
        - [help](#help)
        - [markdown_to_ebook](#markdowntoebook)
    - [Sources](#sources)

<!-- markdown-toc end -->

This tool generates EPUB and MOBI dictionary files.

See the [Releases](https://github.com/simsapa/simsapa-dictionary/releases) page for downloads.

It includes a set of Pali - English dictionaries, and this tool for Linux, Mac and Windows.

The dictionary source texts are in the [simsapa-dictionary-data](https://github.com/simsapa/simsapa-dictionary-data) repo.

You can download the source text, edit and generate updated EPUB and MOBI files using this tool.

To generate MOBI files, also download [Kindlegen](https://www.amazon.com/gp/feature.html?docId=1000765211) from Amazon (free download).

## Feedback, corrections, bug reports

Both the tool and the dictionary content has some rough edges.

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
    cover_path = "cover.jpg"
    book_id = "NcpedDictionarySimsapa"
    created_date_human = ""
    created_date_opf = ""
    is_epub = true
    is_mobi = false
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

### help

```
./simsapa_dictionary help
```

```
Simsapa Dictionary Tool 0.1.0
https://simsapa.github.io/
Generating Pali language dictionaries in MOBI and EPUB format.

USAGE:
    simsapa_dictionary [FLAGS] [SUBCOMMAND]

FLAGS:
    -h, --help         Prints help information
        --show_logs    Print log messages in the terminal.
    -V, --version      Prints version information

SUBCOMMANDS:
    help                             Prints this message or the help of the given subcommand(s)
    markdown_to_ebook                Process a Markdown file and generate an EPUB or MOBI dictionary.
    nyanatiloka_to_markdown          Process Ven. Nyanatiloka's Buddhist Dictionary and write a Markdown file with
                                     TOML headers.
    suttacentral_json_to_markdown    Process a dictionary JSON file from SuttaCentral and write a Markdown file with
                                     TOML headers.
```

### markdown_to_ebook

```
./simsapa_dictionary help markdown_to_ebook
```

```
Process a Markdown file and generate an EPUB or MOBI dictionary.

USAGE:
    simsapa_dictionary markdown_to_ebook [FLAGS] [OPTIONS] --ebook_format <FORMAT>

FLAGS:
        --dont_remove_generated_files    Turns off the removal of the generated OPF, HTML, etc. files used to create the
                                         MOBI. Useful for debugging.
        --dont_run_kindlegen             Turns off running KindleGen, and no MOBI file will be generated. Useful for
                                         debugging.
    -h, --help                           Prints help information
    -V, --version                        Prints version information
        --zip_with_cli                   Use the cli zip tool to create the Epub.
        --zip_with_lib                   Use the embedded zip library to create the Epub.

OPTIONS:
        --dict_label <LABEL>            Use this dict_label property, instead of the one defined in the Markdown file.
        --ebook_format <FORMAT>         Either EPUB or MOBI. [default: EPUB]  [possible values: EPUB, Epub, epub, MOBI,
                                        Mobi, mobi]
        --kindlegen_path <PATH>         The KindleGen tool must be available either (a) in the current folder with this
                                        tool, (b) in the system PATH, (c) declared with this option.
        --source_path <PATH>            A single Markdown file to read dictionary entries from. Either this or
                                        'source_paths_list' must be used.
        --source_paths_list <PATH>      A file with a list of Markdown file paths, one per line. Either this or
                                        'source_path' must be used.
        --mobi_compression <INT>        Compression level, 0-2, as used by KindleGen. 0: no compression, 1: standard DOC
                                        compression, 2: Kindle huffdic compression. [default: 0]  [possible values: 0,
                                        1, 2]
        --output_path <PATH>            The EPUB or MOBI file to write. Defaults to the same file name and folder as the
                                        first Markdown source.
        --title <TITLE>                 Use this title for the ebook, instead of the one defined in the Markdown file.
```

## Sources

The dictionary texts at [simsapa-dictionary-data](https://github.com/simsapa/simsapa-dictionary-data) were created using:

- JSON format dictionaries published at [suttacentral/sc-data](https://github.com/suttacentral/sc-data)
- [Nyanatiloka: Buddhist Dictionary](https://what-buddha-said.net/library/Buddhist.Dictionary/index_dict.n2.htm) published by [what-buddha-said.net](https://what-buddha-said.net/)

