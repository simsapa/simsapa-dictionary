#!/bin/bash

cargo run -- markdown_to_ebook \
  --source_path ../simsapa-dictionary-data/ncped.md \
  --dict_label "" \
  --ebook_format epub \
  --output_path ../simsapa-dictionary_releases/ncped.epub


