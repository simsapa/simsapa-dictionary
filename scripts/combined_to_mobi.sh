#!/bin/bash

cargo run -- markdown_to_ebook \
	--source_paths_list ./scripts/combined_dict_md_paths.txt \
	--title "Combined Pali - English Dictionary" \
	--cover_path "covers/combined_cover.png" \
	--word_prefix_velthuis \
	--output_format mobi \
	--output_path ../simsapa-dictionary_releases/combined-dictionary.mobi \
	--mobi_compression 0 \
	--kindlegen_path $HOME/lib/kindlegen/kindlegen

