#!/bin/bash

# The first argument is the input file on the command line. Such as:
# ./xlsx_to_stardict ncped.xlsx

SRC=$(basename $1 .xlsx)

STARDICT_TEXT2BIN="/usr/lib/stardict-tools/stardict-text2bin"

stardict_out="./$SRC-stardict"
mkdir -p "$stardict_out"

echo -n "Running simsapa_dictionary_linux ... "

./simsapa_dictionary_linux xlsx_to_stardict_xml \
    --source_path "$1" \
    --output_path "$stardict_out/$SRC.xml"

if [[ "$?" != "0" ]]; then
    "Error."
    exit 2
else
    echo "OK"
fi

echo -n "Running stardict-text2bin ..."

cd "$stardict_out"
$STARDICT_TEXT2BIN "$SRC.xml" "$SRC.ifo"
rm "$SRC.xml"
cd ..

echo -n "Creating zip archive ... "

zip -r "$SRC-stardict.zip" "$SRC-stardict"
rm "$SRC-stardict" -r

if [[ "$?" != "0" ]]; then
    "Error."
    exit 2
else
    echo "OK"
fi
