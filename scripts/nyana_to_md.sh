#!/bin/bash

NYANA_ROOT="$HOME/src/dict-nyanatiloka"
OUT_DIR="../simsapa-dictionary-data"

cargo run -- nyanatiloka_to_markdown \
	--nyanatiloka_root "$NYANA_ROOT" \
	--markdown_path "$OUT_DIR/nyana.md" \
	--dict_label Nyana

