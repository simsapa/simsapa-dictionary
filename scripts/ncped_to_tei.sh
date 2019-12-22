#!/bin/bash

#  --keep_entries_plaintext \

cargo run -- markdown_to_tei \
  --source_path ../simsapa-dictionary-data/ncped.md \
  --output_path ../simsapa-dictionary_releases/ncped-tei.xml


