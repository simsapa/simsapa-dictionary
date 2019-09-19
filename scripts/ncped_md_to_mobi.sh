#!/bin/bash

cargo run -- markdown_to_mobi \
	--markdown_paths_list ./scripts/two_ncped.txt \
	--mobi_path ../simsapa-dictionary-data/ncped.mobi \
	--mobi_compression 0 \
	--dont_remove_generated_files \
	--kindlegen_path $HOME/lib/kindlegen/kindlegen

