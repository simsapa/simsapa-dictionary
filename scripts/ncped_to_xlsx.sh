#!/bin/bash

SRC_DIR="../simsapa-dictionary-data"
OUT_DIR="../simsapa-dictionary_releases/ncped"

i=ncped

cargo run -- markdown_to_json \
  --source_path "$SRC_DIR/$i.md" \
  --output_path "$OUT_DIR/$i.json"

cat ./scripts/ncped-tables.txt | \
  sed 's/^load "ncped.json";/load "'"$i"'.json";/' | \
  sed 's/^load "ncped-metadata.json";/load "'"$i"'-metadata.json";/' | \
  sed 's/^write "ncped.xlsx";/write "'$i'.xlsx";/' | \
  cat -s > "$OUT_DIR/$i-tables.txt"

cd "$OUT_DIR"
python2 ../../json2xlsx_simsapa/json2xlsx/utilities/json2xlsx.py "$i-tables.txt"

rm "$i-tables.txt" "$i.json" "$i-metadata.json"


