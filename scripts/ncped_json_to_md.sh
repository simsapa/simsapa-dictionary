#!/bin/bash

#RUST_LOG=sources_to_markdown=info cargo run --bin sources_to_markdown 2>&1 | tee sources_to_markdown.log

SC_ROOT="$HOME/src/suttacentral-2019-11-15/sc-data/dictionaries/en"
OUT_DIR="../simsapa-dictionary-data"

cargo run -- suttacentral_json_to_markdown \
	--json_path "$SC_ROOT/ncped.json" \
	--output_path "$OUT_DIR/ncped.md" \
	--dict_label NCPED

