#!/bin/bash

#RUST_LOG=sources_to_markdown=info cargo run --bin sources_to_markdown 2>&1 | tee sources_to_markdown.log

NYANA_ROOT="$HOME/src/dict-nyanatiloka"
OUT_DIR="../simsapa-dictionary-data"

cargo run -- nyanatiloka_to_markdown \
	--nyanatiloka_root "$NYANA_ROOT" \
	--markdown_path "$OUT_DIR/nyana.md" \
	--dict_label Nyana

