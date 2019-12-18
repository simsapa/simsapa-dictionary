#!/bin/bash

cargo run -- markdown_to_stardict_xml \
  --allow_raw_html \
  --source_path ../simsapa-dictionary-data/ncped.md \
  --output_path ../simsapa-dictionary_releases/ncped.xml


