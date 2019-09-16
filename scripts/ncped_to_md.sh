#!/bin/bash

#RUST_LOG=sources_to_markdown=info cargo run --bin sources_to_markdown 2>&1 | tee sources_to_markdown.log

cargo run -- suttacentral_json_to_markdown --json_path ~/src/suttacentral-2018-09-03/sc-data/dictionaries/en/ncped.json --markdown_path ../simsapa-dictionary-data/ncped.md --dict_label NCPED

