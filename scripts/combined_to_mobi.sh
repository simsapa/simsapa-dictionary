#!/bin/bash

cargo run -- markdown_to_ebook \
	--markdown_paths_list ./scripts/combined_dict_md_paths.txt \
	--title "Combined Pali - English Dictionary" \
	--ebook_format mobi \
	--output_path ../simsapa-dictionary_releases/combined-dictionary.mobi \
	--mobi_compression 0 \
	--dont_remove_generated_files \
	--kindlegen_path $HOME/lib/kindlegen/kindlegen

