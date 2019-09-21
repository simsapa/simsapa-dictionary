#!/bin/bash

cargo run -- markdown_to_ebook \
	--markdown_path ../simsapa-dictionary-data/ncped.md \
	--ebook_format epub \
	--output_path ../simsapa-dictionary-data/ncped.epub \
	--dont_remove_generated_files

