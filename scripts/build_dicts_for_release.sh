#!/bin/bash

SRC_DIR=../simsapa-dictionary-data
OUT_DIR=../simsapa-dictionary_releases/new-release

KINDLEGEN_PATH="$HOME/lib/kindlegen/kindlegen"

# === Combined ===

for i in dhammika dppn ncped nyana pts; do

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
        --output_format epub \
        --output_path "$OUT_DIR/$i.epub"

done

# === Combined ===

cargo run -- markdown_to_ebook \
    --source_paths_list ./scripts/combined_dict_md_paths.txt \
    --title "Combined Pali - English Dictionary" \
    --output_format mobi \
    --output_path "$OUT_DIR/combined-dictionary.mobi" \
    --mobi_compression 0 \
    --kindlegen_path "$KINDLEGEN_PATH"

cargo run -- markdown_to_ebook \
    --source_paths_list ./scripts/combined_dict_md_paths.txt \
    --title "Combined Pali - English Dictionary" \
    --output_format epub \
    --output_path "$OUT_DIR/combined-dictionary.epub"

