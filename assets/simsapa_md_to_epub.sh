#!/bin/bash

SRC_NAME=$(basename $1 .md)
DEST_FILE="$SRC_NAME.epub"

# Convert
simsapa_dictionary markdown_to_ebook \
    --markdown_path "$1" \
    --dict_label "" \
    --ebook_format epub \
    --output_path "$DEST_FILE"

