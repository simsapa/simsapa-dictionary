#!/bin/bash

cargo run -- markdown_to_ebook \
  --source_path ../simsapa-dictionary-data/pts.md \
  --dict_label "" \
  --word_prefix "*" \
  --output_format epub \
  --output_path ../simsapa-dictionary_releases/pts.epub


