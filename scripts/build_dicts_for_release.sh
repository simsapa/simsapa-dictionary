#!/bin/bash

SRC_DIR=../simsapa-dictionary-data
OUT_DIR=../simsapa-dictionary_releases/new-release

if [ -z "$OUT_DIR" ]; then
    rm -r "$OUT_DIR"
fi

mkdir -p "$OUT_DIR"

PROJ_ROOT=$(pwd)

KINDLEGEN_PATH="$HOME/lib/kindlegen/kindlegen"

STARDICT_TEXT2BIN="/usr/lib/stardict-tools/stardict-text2bin"

# === Individual ===

for i in dhammika dppn ncped nyana pts; do
    cd "$PROJ_ROOT"

    cargo run -- markdown_to_ebook \
        --source_path "$SRC_DIR/$i.md" \
        --dict_label "" \
        --output_format mobi \
        --output_path "$OUT_DIR/$i.mobi" \
        --mobi_compression 0 \
        --kindlegen_path "$KINDLEGEN_PATH"

    cargo run -- markdown_to_ebook \
        --source_path "$SRC_DIR/$i.md" \
        --dict_label "" \
        --word_prefix "*" \
        --output_format epub \
        --output_path "$OUT_DIR/$i.epub"

    # TODO epubcheck

    stardict_out="$OUT_DIR/$i-stardict"
    mkdir -p "$stardict_out"

    cargo run -- markdown_to_stardict_xml \
        --source_path "$SRC_DIR/$i.md" \
        --output_path "$stardict_out/$i.xml"

    cd "$stardict_out"
    $STARDICT_TEXT2BIN "$i.xml" "$i.ifo"
    rm "$i.xml"
    cd ..
    zip -r "$i-stardict.zip" "$i-stardict"
    rm "$i-stardict" -r

done

# === Combined ===

cd "$PROJ_ROOT"

name="combined-dictionary"

cargo run -- markdown_to_ebook \
    --title "Combined Pali - English Dictionary" \
    --source_paths_list ./scripts/combined_dict_md_paths.txt \
    --output_format mobi \
    --output_path "$OUT_DIR/$name.mobi" \
    --mobi_compression 0 \
    --kindlegen_path "$KINDLEGEN_PATH"

cargo run -- markdown_to_ebook \
    --title "Combined Pali - English Dictionary" \
    --source_paths_list ./scripts/combined_dict_md_paths.txt \
    --word_prefix "*" \
    --output_format epub \
    --output_path "$OUT_DIR/$name.epub"

# TODO epubcheck

stardict_out="$OUT_DIR/$name-stardict"
mkdir -p "$stardict_out"

cargo run -- markdown_to_stardict_xml \
    --title "Combined Pali - English Dictionary" \
    --source_paths_list ./scripts/combined_dict_md_paths.txt \
    --output_path "$stardict_out/$name.xml"

cd "$stardict_out"
$STARDICT_TEXT2BIN "$name.xml" "$name.ifo"
rm "$name.xml"
cd ..
zip -r "$name-stardict.zip" "$name-stardict"
rm "$name-stardict" -r

