#!/bin/bash

SC_ROOT="$1/dictionaries/en"
OUT_DIR="$2"

# === Dhammika ===

cargo run -- suttacentral_json_to_markdown \
	--reuse_metadata \
	--title "Dhammika Pali - English Dictionary" \
	--dict_label Dhammika \
	--source_path "$SC_ROOT/dhammika.json" \
	--output_path "$OUT_DIR/dhammika.md"

cargo run -- suttacentral_json_to_markdown \
	--reuse_metadata \
	--title "Dhammika Pali - English Dictionary" \
	--dict_label Dhammika \
	--dont_process \
	--source_path "$SC_ROOT/dhammika.json" \
	--output_path "$OUT_DIR/dhammika_unprocessed.md"

# === DPPN ===

cargo run -- suttacentral_json_to_markdown \
	--reuse_metadata \
	--title "Dictionary of Pali Proper Names (DPPN)" \
	--dict_label DPPN \
	--source_path "$SC_ROOT/dppn.json" \
	--output_path "$OUT_DIR/dppn.md"

cargo run -- suttacentral_json_to_markdown \
	--reuse_metadata \
	--title "Dictionary of Pali Proper Names (DPPN)" \
	--dict_label DPPN \
	--dont_process \
	--source_path "$SC_ROOT/dppn.json" \
	--output_path "$OUT_DIR/dppn_unprocessed.md"

# === NCPED ===

cargo run -- suttacentral_json_to_markdown \
	--reuse_metadata \
	--title "New Concise Pali - English Dictionary (NCPED)" \
	--dict_label NCPED \
	--source_path "$SC_ROOT/ncped.json" \
	--output_path "$OUT_DIR/ncped.md"

cargo run -- suttacentral_json_to_markdown \
	--reuse_metadata \
	--title "New Concise Pali - English Dictionary (NCPED)" \
	--dict_label NCPED \
	--dont_process \
	--source_path "$SC_ROOT/ncped.json" \
	--output_path "$OUT_DIR/ncped_unprocessed.md"

# === PTS ===

cargo run -- suttacentral_json_to_markdown \
	--reuse_metadata \
	--title "Pali Text Society Pali - English Dictionary (PTS)" \
	--dict_label PTS \
	--dont_remove_see_also \
	--source_path "$SC_ROOT/pts.json" \
	--output_path "$OUT_DIR/pts.md"

cargo run -- suttacentral_json_to_markdown \
	--reuse_metadata \
	--title "Pali Text Society Pali - English Dictionary (PTS)" \
	--dict_label PTS \
	--dont_process \
	--dont_remove_see_also \
	--source_path "$SC_ROOT/pts.json" \
	--output_path "$OUT_DIR/pts_unprocessed.md"
