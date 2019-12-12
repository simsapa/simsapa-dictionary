#!/bin/bash

SRC_DIR=../simsapa-dictionary-data
OUT_DIR=../simsapa-dictionary_releases/dict

if [ -z "$OUT_DIR" ]; then
    rm -r "$OUT_DIR"
fi

mkdir -p "$OUT_DIR"

PROJ_ROOT=$(pwd)

i=ncped

n="$SRC_DIR/$i.md"
title=$(grep -E '^title = "' "$n" | sed 's/^title = "\([^"]\+\)"/\1/')
description=$(grep -E '^description = "' "$n" | sed 's/^description = "\([^"]\+\)"/\1/')
version=$(grep -E '^version = "' "$n" | sed 's/^version = "\([^"]\+\)"/\1/')
url=$(grep -E '^source = "' "$n" | sed 's/^source = "\([^"]\+\)"/\1/')

if [[ "$description" != "" ]]; then
  name="$name, $description"
fi

if [[ "$version" != "" ]]; then
  name="$name, $version"
fi

name="$title"

echo "Name: $name"

for fmt in plaintext html; do
  cd "$PROJ_ROOT"

  echo "Format: $fmt"

  dict_out="$OUT_DIR/$i-$fmt-dict"
  mkdir -p "$dict_out"

  if [[ "$fmt" == "plaintext" ]]; then
    cargo run -- markdown_to_c5 \
      --keep_entries_plaintext \
      --source_path "$SRC_DIR/$i.md" \
      --output_path "$dict_out/$i-$fmt.txt"
  else
    cargo run -- markdown_to_c5 \
      --source_path "$SRC_DIR/$i.md" \
      --output_path "$dict_out/$i-$fmt.txt"
  fi

  cd "$dict_out"

  if [[ "$fmt" == "plaintext" ]]; then
    dictfmt \
      -c5 \
      --headword-separator '; ' \
      --columns 0 \
      --utf8 \
      --allchars \
      -s "$name" \
      -u "$url" \
      "$i-$fmt" < "$i-$fmt.txt"
  else
    dictfmt \
      -c5 \
      --headword-separator '; ' \
      --columns 0 \
      --utf8 \
      --allchars \
      -s "$name" \
      -u "$url" \
      --mime-header 'Content-Type: text/html' \
      "$i-$fmt" < "$i-$fmt.txt"
  fi

  dictzip "$i-$fmt.dict"

  rm "$i-$fmt.txt"
  cd ..
  zip -r "$i-$fmt-dict.zip" "$i-$fmt-dict"
  rm "$i-$fmt-dict" -r

done
