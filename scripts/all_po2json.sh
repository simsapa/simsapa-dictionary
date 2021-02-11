#!/bin/zsh

if [ ! -e "Cargo.toml" ]
then
    echo "Call this from crate root."
    echo "Usage: ./scripts/all_po2json.sh"
    exit 2
fi

SC_PATH="$1"
if [[ "$SC_PATH" == "" ]]; then
  echo "First argument must be the path to the SuttaCentral data."
  exit 2
fi

JSON_ROOT="$2"
if [[ "$JSON_ROOT" == "" ]]; then
  echo "Second argument must be the path to write the converted JSON files."
  exit 2
fi

# No trailing slash.
PO_ROOT="$SC_PATH/po_text"

if [ ! -d "$PO_ROOT" -o ! -d "$JSON_ROOT" ]
then
    echo "Some folders don't exist."
    exit 2
fi

for i in "$PO_ROOT"/**/*.po
do
    name=$(basename "$i")
    echo -n "$name -> "
    folder=$(echo -n "$i" | sed -e 's%^'"$PO_ROOT"'%%; s%'"$name"'$%%;')
    dest_folder="$JSON_ROOT$folder"
    dest_json="$dest_folder"$(basename -s .po "$i")".json"
    mkdir -p "$dest_folder"
    echo -n "$dest_json ... "
    ./scripts/convert_po.py "$i" "$dest_json"

    if [ $? -eq 0 ]
    then
        echo "OK"
    else
        echo "Error from convert_po.py. Exiting."
        exit 2
    fi
done
