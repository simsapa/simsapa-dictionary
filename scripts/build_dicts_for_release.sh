#!/bin/bash

BUILD_INDIVIDUAL=1
BUILD_COMBINED=1

RUN_EPUBCHECK=0
RUN_JING=0

FORMAT_MOBI=1
FORMAT_EPUB=1
FORMAT_BABYLON=1
FORMAT_STARDICT=1
FORMAT_XLSX=1
FORMAT_DICT=1
FORMAT_TEI=1

SRC_DIR=../simsapa-dictionary-data
OUT_DIR=../simsapa-dictionary_releases/new-release

if [ -z "$OUT_DIR" ]; then
    rm -r "$OUT_DIR"
fi

mkdir -p "$OUT_DIR"

PROJ_ROOT=$(pwd)

KINDLEGEN_PATH="$HOME/lib/kindlegen/kindlegen"

EPUBCHECK_PATH="$HOME/bin/epubcheck"

STARDICT_TEXT2BIN="/usr/lib/stardict-tools/stardict-text2bin"

FREEDICT_RNG="$PROJ_ROOT/assets/freedict-P5.rng"

# === Individual ===

if [[ "$BUILD_INDIVIDUAL" -eq 1 ]]; then

    for i in dhammika dppn ncped nyana pts; do
        cd "$PROJ_ROOT"

        # === Mobi ===

        if [[ "$FORMAT_MOBI" -eq 1 ]]; then

            cargo run -- markdown_to_ebook \
                --source_path "$SRC_DIR/$i.md" \
                --dict_label "" \
                --output_format mobi \
                --output_path "$OUT_DIR/$i.mobi" \
                --mobi_compression 0 \
                --kindlegen_path "$KINDLEGEN_PATH"

        fi

        # === Epub ===

        if [[ "$FORMAT_EPUB" -eq 1 ]]; then

            cargo run -- markdown_to_ebook \
                --source_path "$SRC_DIR/$i.md" \
                --dict_label "" \
                --word_prefix "*" \
                --output_format epub \
                --output_path "$OUT_DIR/$i.epub"

        fi

        # === Babylon ===

        if [[ "$FORMAT_BABYLON" -eq 1 ]]; then

            cargo run -- markdown_to_babylon_gls \
                --source_path "$SRC_DIR/$i.md" \
                --output_path "$OUT_DIR/$i-babylon.gls"

        fi

        # === Stardict ===

        if [[ "$FORMAT_STARDICT" -eq 1 ]]; then

            stardict_out="$OUT_DIR/$i-stardict"
            mkdir -p "$stardict_out"

            cargo run -- markdown_to_stardict_xml \
                --source_path "$SRC_DIR/$i.md" \
                --output_path "$stardict_out/$i.xml"

            cd "$stardict_out"
            $STARDICT_TEXT2BIN "$i.xml" "$i.ifo"
            rm "$i.xml"
            cd ..
            zip -r "$i-stardict.zip" "$i-stardict"
            rm "$i-stardict" -r

        fi

        # === Xlsx ===

        if [[ "$FORMAT_XLSX" -eq 1 ]]; then

            cd "$PROJ_ROOT"

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

        fi

        # === Dict ===

        if [[ "$FORMAT_DICT" -eq 1 ]]; then

            n="$SRC_DIR/$i.md"
            title=$(grep -E '^title = "' "$n" | sed 's/^title = "\([^"]\+\)"/\1/')
            description=$(grep -E '^description = "' "$n" | sed 's/^description = "\([^"]\+\)"/\1/')
            version=$(grep -E '^version = "' "$n" | sed 's/^version = "\([^"]\+\)"/\1/')
            url=$(grep -E '^source = "' "$n" | sed 's/^source = "\([^"]\+\)"/\1/')

            name="$title"

            if [[ "$description" != "" ]]; then
                name="$name, $description"
            fi

            if [[ "$version" != "" ]]; then
                name="$name, $version"
            fi

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
        fi

        # === Freedict TEI ===

        if [[ "$FORMAT_TEI" -eq 1 ]]; then

            cd "$PROJ_ROOT"

            cargo run -- markdown_to_tei \
                --source_path "$SRC_DIR/$i.md" \
                --output_path "$OUT_DIR/$i.tei"

        fi

    done

fi

# === Combined ===

if [[ "$BUILD_COMBINED" -eq 1 ]]; then

    cd "$PROJ_ROOT"

    i="combined-dictionary"

    # === Mobi ===

    if [[ "$FORMAT_MOBI" -eq 1 ]]; then

        cargo run -- markdown_to_ebook \
            --title "Combined Pali - English Dictionary" \
            --source_paths_list ./scripts/combined_dict_md_paths.txt \
            --output_format mobi \
            --output_path "$OUT_DIR/$i.mobi" \
            --mobi_compression 0 \
            --kindlegen_path "$KINDLEGEN_PATH"

    fi

    # === Epub ===

    if [[ "$FORMAT_EPUB" -eq 1 ]]; then

        cargo run -- markdown_to_ebook \
            --title "Combined Pali - English Dictionary" \
            --source_paths_list ./scripts/combined_dict_md_paths.txt \
            --word_prefix "*" \
            --output_format epub \
            --output_path "$OUT_DIR/$i.epub"

    fi

    # === Babylon ===

    if [[ "$FORMAT_BABYLON" -eq 1 ]]; then

        cargo run -- markdown_to_babylon_gls \
            --title "Combined Pali - English Dictionary" \
            --source_paths_list ./scripts/combined_dict_md_paths.txt \
            --output_path "$OUT_DIR/$i-babylon.gls"

    fi

    # === Stardict ===

    if [[ "$FORMAT_STARDICT" -eq 1 ]]; then

        stardict_out="$OUT_DIR/$i-stardict"
        mkdir -p "$stardict_out"

        cargo run -- markdown_to_stardict_xml \
            --title "Combined Pali - English Dictionary" \
            --source_paths_list ./scripts/combined_dict_md_paths.txt \
            --output_path "$stardict_out/$i.xml"

        cd "$stardict_out"
        $STARDICT_TEXT2BIN "$i.xml" "$i.ifo"
        rm "$i.xml"
        cd ..
        zip -r "$i-stardict.zip" "$i-stardict"
        rm "$i-stardict" -r

    fi

    # === Dict ===

    if [[ "$FORMAT_DICT" -eq 1 ]]; then

        name="Combined Pali - English Dictionary"
        url="https://simsapa.github.io"

        echo "Name: $name"

        for fmt in plaintext html; do
            cd "$PROJ_ROOT"

            echo "Format: $fmt"

            dict_out="$OUT_DIR/$i-$fmt-dict"
            mkdir -p "$dict_out"

            if [[ "$fmt" == "plaintext" ]]; then
                cargo run -- markdown_to_c5 \
                    --keep_entries_plaintext \
                    --title "$title" \
                    --source_paths_list ./scripts/combined_dict_md_paths.txt \
                    --output_path "$dict_out/$i-$fmt.txt"
            else
                cargo run -- markdown_to_c5 \
                    --title "Combined Pali - English Dictionary" \
                    --source_paths_list ./scripts/combined_dict_md_paths.txt \
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
    fi

    # === Freedict TEI ===

    if [[ "$FORMAT_TEI" -eq 1 ]]; then

        cd "$PROJ_ROOT"

        cargo run -- markdown_to_tei \
            --title "Combined Pali - English Dictionary" \
            --source_paths_list ./scripts/combined_dict_md_paths.txt \
            --output_path "$OUT_DIR/$i.tei"

    fi

fi

# === Epubcheck ===

if [[ "$RUN_EPUBCHECK" -eq 1 ]]; then
    cd "$PROJ_ROOT"
    cd "$OUT_DIR"

    for i in ./*.epub; do
        echo "=== epubcheck $i ==="
        $EPUBCHECK_PATH $i
    done
fi

# === Jing ===

if [[ "$RUN_JING" -eq 1 ]]; then
    cd "$PROJ_ROOT"
    cd "$OUT_DIR"

    for i in ./*.tei; do
        echo "=== jing $i ==="
        jing "$FREEDICT_RNG" $i
    done
fi
