#!/bin/bash

if [ ! -e ./Cargo.toml ]; then
    echo "Run this script from the project root."
    exit 2
fi

PRJ_ROOT=$(pwd)
BIN_PATH=./target/debug/simsapa_dictionary
TEST_TEMP="$PRJ_ROOT/test-temp"

STARDICT_TEXT2BIN="/usr/lib/stardict-tools/stardict-text2bin"
STARDICT_BABYLON="/usr/lib/stardict-tools/babylon"

# Compiling
cargo build

if [[ "$?" != "0" ]]; then
    echo "Failed to compile."
    exit 2
fi

if [ ! -e "$BIN_PATH" ]; then
    echo "Binary not found."
    exit 2
fi

if [ ! -e "$STARDICT_TEXT2BIN" -o ! -e "$STARDICT_BABYLON"]; then
    echo "Stardict binary tools not found. Is the 'stardict-tools' package installed?"
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

# Using --source_path without ./

# Using --output_path without ./, it should add that as a prefix.

./simsapa_dictionary markdown_to_ebook \
    --source_path "data/data with space/ncped with space.md" \
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

echo "=== Test: Build a MOBI from Markdown with first argument without path ==="

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

echo "=== Test: Build an EPUB from Markdown with first argument when kindlegen is not found ==="

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

# ===============================================
# ///////////////////////////////////////////////

echo "=== Test: Build an EPUB with subcommand, output path missing ==="

cd "$TEST_TEMP"

./simsapa_dictionary markdown_to_ebook \
    --source_path "data/data with space/ncped with space.md" \
    --ebook_format epub

if [[ "$?" != "0" ]]; then
    echo "Test Failed."
    exit 2
fi

if [ ! -e "./data/data with space/ncped with space.epub" ]; then
    echo "Test Failed."
    exit 2
else
    rm "./data/data with space/ncped with space.epub"
    echo "Test Passed."
fi

# ===============================================
# ///////////////////////////////////////////////

echo "=== Test: Build a MOBI from XLSX with first argument with path ==="

cd "$TEST_TEMP"

./simsapa_dictionary "data/data with space/ncped with space.xlsx"

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

echo "=== Test: Build a MOBI from XLSX with subcommand ==="

cd "$TEST_TEMP"

# Not using the --kindlegen_path option, it should detect that kindlegen is in PATH.

# Using --source_path without ./

# Using --output_path without ./, it should add that as a prefix.

./simsapa_dictionary xlsx_to_ebook \
    --source_path "data/data with space/ncped with space.xlsx" \
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

echo "=== Test: Build a MOBI with custom cover with path ==="

cd "$TEST_TEMP"

./simsapa_dictionary "data/data with space/ncped custom cover.md"

if [[ "$?" != "0" ]]; then
    echo "Test Failed."
    exit 2
fi

if [ ! -e "./data/data with space/ncped custom cover.mobi" ]; then
    echo "Test Failed."
    exit 2
else
    rm "./data/data with space/ncped custom cover.mobi"
    echo "Test Passed."
fi

# ===============================================
# ///////////////////////////////////////////////

echo "=== Test: Build a Babylon GLS from Markdown and generate a Stardict dictionary. ==="

cd "$TEST_TEMP"

name_space="ncped with space"
name_dash="ncped-with-space"

./simsapa_dictionary markdown_to_babylon_gls \
    --source_path "data/data with space/$name_space.md"

if [[ "$?" != "0" ]]; then
    echo "Test Failed."
    exit 2
fi

if [ ! -e "data/data with space/$name_dash.gls" ]; then
    echo "Test Failed."
    exit 2
fi

cd "data/data with space/"

$STARDICT_BABYLON "$name_dash.gls"

if [ ! -e "$name_dash.dict.dz" ] || [ ! -e "$name_dash.idx" ] || [ ! -e "$name_dash.ifo" ] || [ ! -e "$name_dash.syn" ]; then
    echo "Failed to generate Stardict files."
    exit 2
else
    rm "$name_dash.gls" "$name_dash.dict.dz" "$name_dash.idx" "$name_dash.ifo" "$name_dash.syn"
    echo "Test Passed."
fi

# ===============================================
# ///////////////////////////////////////////////

echo "=== Test: Build a Babylon GLS from XLSX and generate a Stardict dictionary. ==="

cd "$TEST_TEMP"

name_space="ncped with space"
name_dash="ncped-with-space"

./simsapa_dictionary xlsx_to_babylon_gls \
    --source_path "data/data with space/$name_space.xlsx"

if [[ "$?" != "0" ]]; then
    echo "Test Failed."
    exit 2
fi

if [ ! -e "data/data with space/$name_dash.gls" ]; then
    echo "Test Failed."
    exit 2
fi

cd "data/data with space/"

$STARDICT_BABYLON "$name_dash.gls"

if [ ! -e "$name_dash.dict.dz" ] || [ ! -e "$name_dash.idx" ] || [ ! -e "$name_dash.ifo" ] || [ ! -e "$name_dash.syn" ]; then
    echo "Failed to generate Stardict files."
    exit 2
else
    rm "$name_dash.gls" "$name_dash.dict.dz" "$name_dash.idx" "$name_dash.ifo" "$name_dash.syn"
    echo "Test Passed."
fi

# ===============================================
# ///////////////////////////////////////////////

echo "=== Test: Build a Stardict XML from Markdown and generate a Stardict dictionary. ==="

cd "$TEST_TEMP"

name_space="ncped with space"
name_dash="ncped-with-space"

./simsapa_dictionary markdown_to_stardict_xml \
    --source_path "data/data with space/$name_space.md"

if [[ "$?" != "0" ]]; then
    echo "Test Failed."
    exit 2
fi

if [ ! -e "data/data with space/$name_dash.xml" ]; then
    echo "Test Failed."
    exit 2
fi

cd "data/data with space/"

$STARDICT_TEXT2BIN "$name_dash.xml" "$name_dash.ifo"

if [ ! -e "$name_dash.dict.dz" ] || [ ! -e "$name_dash.idx" ] || [ ! -e "$name_dash.ifo" ] || [ ! -e "$name_dash.syn" ]; then
    echo "Failed to generate Stardict files."
    exit 2
else
    rm "$name_dash.xml" "$name_dash.dict.dz" "$name_dash.idx" "$name_dash.ifo" "$name_dash.syn"
    echo "Test Passed."
fi

# ===============================================
# ///////////////////////////////////////////////

echo "=== Test: Build a Stardict XML from XLSX and generate a Stardict dictionary. ==="

cd "$TEST_TEMP"

name_space="ncped with space"
name_dash="ncped-with-space"

./simsapa_dictionary xlsx_to_stardict_xml \
    --source_path "data/data with space/$name_space.xlsx"

if [[ "$?" != "0" ]]; then
    echo "Test Failed."
    exit 2
fi

if [ ! -e "data/data with space/$name_dash.xml" ]; then
    echo "Test Failed."
    exit 2
fi

cd "data/data with space/"

$STARDICT_TEXT2BIN "$name_dash.xml" "$name_dash.ifo"

if [ ! -e "$name_dash.dict.dz" ] || [ ! -e "$name_dash.idx" ] || [ ! -e "$name_dash.ifo" ] || [ ! -e "$name_dash.syn" ]; then
    echo "Failed to generate Stardict files."
    exit 2
else
    rm "$name_dash.xml" "$name_dash.dict.dz" "$name_dash.idx" "$name_dash.ifo" "$name_dash.syn"
    echo "Test Passed."
fi

# === Clean up. ===

echo "All tests passed."
echo "Cleaning up generated test files."

cd "$PRJ_ROOT" && rm "$TEST_TEMP" -r

