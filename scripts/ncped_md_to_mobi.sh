#!/bin/bash

cargo run -- markdown_to_mobi \
	--markdown_path ../simsapa-dictionary-data/ncped.md \
	--mobi_path ../simsapa-dictionary-data/ncped.mobi \
	--kindlegen_path $HOME/lib/kindlegen/kindlegen

