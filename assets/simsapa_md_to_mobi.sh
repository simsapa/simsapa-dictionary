#!/bin/bash

SRC_NAME=$(basename $1 .md)
DEST_FILE="$SRC_NAME.mobi"

# If kindlegen is not in PATH, specify it with "--kindlegen_path path/to/kindlegen".

# Convert
simsapa_dictionary markdown_to_ebook \
    --source_path "$1" \
    --dict_label "" \
    --ebook_format mobi \
    --output_path "$DEST_FILE" \
    --mobi_compression 0

