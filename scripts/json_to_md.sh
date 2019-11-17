#!/bin/bash

#RUST_LOG=sources_to_markdown=info cargo run --bin sources_to_markdown 2>&1 | tee sources_to_markdown.log

SC_ROOT="$HOME/src/suttacentral-2019-11-15/sc-data/dictionaries/en"
OUT_DIR="../simsapa-dictionary-data"

# === Dhammika ===

cargo run -- suttacentral_json_to_markdown \
	--title "Dhammika Pali - English Dictionary" \
	--dict_label Dhammika \
	--json_path "$SC_ROOT/dhammika.json" \
	--output_path "$OUT_DIR/dhammika.md"

cargo run -- suttacentral_json_to_markdown \
	--title "Dhammika Pali - English Dictionary" \
	--dict_label Dhammika \
	--dont_process \
	--json_path "$SC_ROOT/dhammika.json" \
	--output_path "$OUT_DIR/dhammika_unprocessed.md"

# === DPPN ===

cargo run -- suttacentral_json_to_markdown \
	--title "Dictionary of Pali Proper Names (DPPN)" \
	--dict_label DPPN \
	--json_path "$SC_ROOT/dppn.json" \
	--output_path "$OUT_DIR/dppn.md"

cargo run -- suttacentral_json_to_markdown \
	--title "Dictionary of Pali Proper Names (DPPN)" \
	--dict_label DPPN \
	--dont_process \
	--json_path "$SC_ROOT/dppn.json" \
	--output_path "$OUT_DIR/dppn_unprocessed.md"

# === NCPED ===

cargo run -- suttacentral_json_to_markdown \
	--title "New Concise Pali - English Dictionary (NCPED)" \
	--dict_label NCPED \
	--json_path "$SC_ROOT/ncped.json" \
	--output_path "$OUT_DIR/ncped.md"

cargo run -- suttacentral_json_to_markdown \
	--title "New Concise Pali - English Dictionary (NCPED)" \
	--dict_label NCPED \
	--dont_process \
	--json_path "$SC_ROOT/ncped.json" \
	--output_path "$OUT_DIR/ncped_unprocessed.md"

# === PTS ===

cargo run -- suttacentral_json_to_markdown \
	--title "Pali Text Society Pali - English Dictionary (PTS)" \
	--dict_label PTS \
	--dont_remove_see_also \
	--json_path "$SC_ROOT/pts.json" \
	--output_path "$OUT_DIR/pts.md"

cargo run -- suttacentral_json_to_markdown \
	--title "Pali Text Society Pali - English Dictionary (PTS)" \
	--dict_label PTS \
	--dont_process \
	--dont_remove_see_also \
	--json_path "$SC_ROOT/pts.json" \
	--output_path "$OUT_DIR/pts_unprocessed.md"
