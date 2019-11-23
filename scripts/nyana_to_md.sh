#!/bin/bash

NYANA_ROOT="$HOME/src/dict-nyanatiloka"
OUT_DIR="../simsapa-dictionary-data"

cargo run -- nyanatiloka_to_markdown \
	--reuse_metadata \
	--title "Nyanatiloka Buddhist Dictionary" \
	--dict_label Nyana \
	--nyanatiloka_root "$NYANA_ROOT" \
	--output_path "$OUT_DIR/nyana.md"

