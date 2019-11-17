#!/bin/bash

cargo run -- markdown_to_ebook \
  --source_path ../simsapa-dictionary-data/ncped.md \
  --dict_label "" \
  --output_format mobi \
  --output_path ../simsapa-dictionary_releases/ncped.mobi \
  --mobi_compression 0 \
  --kindlegen_path $HOME/lib/kindlegen/kindlegen

