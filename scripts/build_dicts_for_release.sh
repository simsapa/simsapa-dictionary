#!/bin/bash

SRC_DIR=../simsapa-dictionary-data
OUT_DIR=../simsapa-dictionary_releases/new-release

if [ -z "$OUT_DIR" ]; then
    rm -r "$OUT_DIR"
fi

mkdir -p "$OUT_DIR"

PROJ_ROOT=$(pwd)

KINDLEGEN_PATH="$HOME/lib/kindlegen/kindlegen"

EPUBCHECK_PATH="$HOME/bin/epubcheck"

STARDICT_TEXT2BIN="/usr/lib/stardict-tools/stardict-text2bin"

# === Individual ===

for i in dhammika dppn ncped nyana pts; do
    cd "$PROJ_ROOT"

    # Mobi

    cargo run -- markdown_to_ebook \
        --source_path "$SRC_DIR/$i.md" \
        --dict_label "" \
        --output_format mobi \
        --output_path "$OUT_DIR/$i.mobi" \
        --mobi_compression 0 \
        --kindlegen_path "$KINDLEGEN_PATH"

    # Epub

    cargo run -- markdown_to_ebook \
        --source_path "$SRC_DIR/$i.md" \
        --dict_label "" \
        --word_prefix "*" \
        --output_format epub \
        --output_path "$OUT_DIR/$i.epub"

    # Babylon

    cargo run -- markdown_to_babylon_gls \
        --source_path "$SRC_DIR/$i.md" \
        --output_path "$OUT_DIR/$i-babylon.gls"

    # Stardict

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

    # Xlsx

    cd "$PROJ_ROOT"

    cargo run -- markdown_to_json \
        --source_path "$SRC_DIR/$i.md" \
        --output_path "$OUT_DIR/$i.json"

    cat ./scripts/ncped-tables.txt | \
    sed 's/^load "ncped.json";/load "'"$i"'.json";/' | \
    sed 's/^load "ncped-metadata.json";/load "'"$i"'-metadata.json";/' | \
    sed 's/^write "ncped.xlsx";/write "'$i'.xlsx";/' | \
    cat -s > "$OUT_DIR/$i-tables.txt"

    cd "$OUT_DIR"
    python2 ../../json2xlsx_simsapa/json2xlsx/utilities/json2xlsx.py "$i-tables.txt"

    rm "$i-tables.txt" "$i.json" "$i-metadata.json"

done

# === Combined ===

cd "$PROJ_ROOT"

name="combined-dictionary"

# Mobi

cargo run -- markdown_to_ebook \
    --title "Combined Pali - English Dictionary" \
    --source_paths_list ./scripts/combined_dict_md_paths.txt \
    --output_format mobi \
    --output_path "$OUT_DIR/$name.mobi" \
    --mobi_compression 0 \
    --kindlegen_path "$KINDLEGEN_PATH"

# Epub

cargo run -- markdown_to_ebook \
    --title "Combined Pali - English Dictionary" \
    --source_paths_list ./scripts/combined_dict_md_paths.txt \
    --word_prefix "*" \
    --output_format epub \
    --output_path "$OUT_DIR/$name.epub"

# Babylon

cargo run -- markdown_to_babylon_gls \
    --title "Combined Pali - English Dictionary" \
    --source_paths_list ./scripts/combined_dict_md_paths.txt \
    --output_path "$OUT_DIR/$name-babylon.gls"

# Stardict

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

# Epubcheck

#cd "$PROJ_ROOT"
#cd "$OUT_DIR"
#
#for i in ./*.epub; do
#    echo "=== epubcheck $i ==="
#    $EPUBCHECK_PATH $i
#done

