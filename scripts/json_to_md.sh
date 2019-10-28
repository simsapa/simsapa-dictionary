#!/bin/bash

#RUST_LOG=sources_to_markdown=info cargo run --bin sources_to_markdown 2>&1 | tee sources_to_markdown.log

SC_ROOT="$HOME/src/suttacentral-2018-09-03/sc-data/dictionaries/en"
OUT_DIR="../simsapa-dictionary-data"

cargo run -- suttacentral_json_to_markdown \
	--json_path "$SC_ROOT/dhammika.json" \
	--output_path "$OUT_DIR/dhammika.md" \
	--dict_label Dhammika

cargo run -- suttacentral_json_to_markdown \
	--json_path "$SC_ROOT/dppn.json" \
	--output_path "$OUT_DIR/dppn.md" \
	--dict_label DPPN

cargo run -- suttacentral_json_to_markdown \
	--json_path "$SC_ROOT/ncped.json" \
	--output_path "$OUT_DIR/ncped.md" \
	--dict_label NCPED

cargo run -- suttacentral_json_to_markdown \
	--json_path "$SC_ROOT/pts.json" \
	--output_path "$OUT_DIR/pts.md" \
	--dict_label PTS
