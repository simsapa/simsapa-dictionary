#!/bin/bash

if [ ! -e ./Cargo.toml ]; then
    echo "Run this script from the project root."
    exit 2
fi

PRJ_ROOT=$(pwd)
BIN_PATH=./target/debug/simsapa_dictionary
TEST_TEMP="$PRJ_ROOT/test-temp"

if [ ! -e "$BIN_PATH" ]; then
    echo "Binary not found. Did run 'cargo build'?"
    exit 2
fi

# Remove previous test-temp, create it again, and copy in the data.

if [[ -d "$TEST_TEMP" ]]; then
    rm "$TEST_TEMP" -r
fi

mkdir -p "$TEST_TEMP"
cp -r ./tests/data "$TEST_TEMP"

# Setup a symlink in the test folder.

if [[ -e "$TEST_TEMP/simsapa_dictionary" || -L "$TEST_TEMP/simsapa_dictionary" ]]; then
    rm "$TEST_TEMP/simsapa_dictionary"
fi

# Symlink target is relative to where the symlink is.
ln -s ../target/debug/simsapa_dictionary "$TEST_TEMP/simsapa_dictionary"

export RUST_LOG=info

# === Tests ===

# ===============================================
# ///////////////////////////////////////////////

echo "=== Test: Build a MOBI with subcommand ==="

cd "$TEST_TEMP"

# Not using the --kindlegen_path option, it should detect that kindlegen is in PATH.

# Using --markdown_path without ./

# Using --output_path without ./, it should add that as a prefix.

./simsapa_dictionary markdown_to_ebook \
    --markdown_path "data/data with space/ncped with space.md" \
    --dict_label "" \
    --ebook_format mobi \
    --output_path "ncped here.mobi" \
    --mobi_compression 0 2>&1 | tee output.log

if [[ "$?" != "0" ]]; then
    echo "Test Failed."
    exit 2
fi

# Check for Kindlegen warnings.
warns=$(grep -E '^Warning' output.log | grep -vE 'Index not supported for enhanced mobi.')

if [[ "$warns" != "" ]]; then
    echo "Kindlegen warnings:"
    echo "$warns"
    echo "Test Failed."
    exit 2
fi

if [ ! -e "./ncped here.mobi" ]; then
    echo "Test Failed."
    exit 2
else
    rm "./ncped here.mobi"
    echo "Test Passed."
fi

# ===============================================
# ///////////////////////////////////////////////

echo "=== Test: Build a MOBI with first argument with path ==="

cd "$TEST_TEMP"

./simsapa_dictionary "data/data with space/ncped with space.md"

if [[ "$?" != "0" ]]; then
    echo "Test Failed."
    exit 2
fi

if [ ! -e "./data/data with space/ncped with space.mobi" ]; then
    echo "Test Failed."
    exit 2
else
    rm "./data/data with space/ncped with space.mobi"
    echo "Test Passed."
fi

# ===============================================
# ///////////////////////////////////////////////

echo "=== Test: Build a MOBI with first argument without path ==="

cd "$TEST_TEMP/data/data with space"

if [[ -L ./simsapa_dictionary ]]; then
    rm ./simsapa_dictionary
fi
ln -s ../../../target/debug/simsapa_dictionary ./simsapa_dictionary

./simsapa_dictionary "ncped with space.md"

if [[ "$?" != "0" ]]; then
    echo "Test Failed."
    exit 2
fi

if [ ! -e "./ncped with space.mobi" ]; then
    echo "Test Failed."
    exit 2
else
    rm "./ncped with space.mobi"
    echo "Test Passed."
fi

# ===============================================
# ///////////////////////////////////////////////

echo "=== Test: Build an EPUB with first argument when kindlegen is not found ==="

PATH_ORIG="$PATH"
export PATH="/usr/local/bin:/usr/bin:/bin:"

cd "$TEST_TEMP/data/data with space"

if [[ -L ./simsapa_dictionary ]]; then
    rm ./simsapa_dictionary
fi
ln -s ../../../target/debug/simsapa_dictionary ./simsapa_dictionary

./simsapa_dictionary "ncped with space.md"

if [[ "$?" != "0" ]]; then
    echo "Test Failed."
    exit 2
fi

if [ ! -e "./ncped with space.epub" ]; then
    echo "Test Failed."
    exit 2
else
    rm "./ncped with space.epub"
    echo "Test Passed."
fi

export PATH="$PATH_ORIG"

# === Clean up. ===

cd "$PRJ_ROOT" && rm "$TEST_TEMP" -r

