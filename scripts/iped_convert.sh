#!/bin/bash

#cargo run -- markdown_to_stardict_xml \
#  --source_path ../simsapa-dictionary-data/iped.md \
#  --output_path ../simsapa-dictionary_releases/iped.xml

#cargo run -- markdown_to_ebook \
#  --source_path ../simsapa-dictionary_docs/mike/iped.md \
#  --dict_label "" \
#  --word_prefix_velthuis \
#  --output_format mobi \
#  --output_path ../simsapa-dictionary_docs/mike/iped.mobi \
#  --mobi_compression 0 \
#  --kindlegen_path $HOME/lib/kindlegen/kindlegen

cargo run -- markdown_to_ebook \
  --source_path ../simsapa-dictionary_docs/mike/iped.md \
  --dict_label "" \
  --word_prefix "*" \
  --output_format epub \
  --output_path ../simsapa-dictionary_docs/mike/iped.epub
