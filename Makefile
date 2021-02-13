SHELL=/bin/bash

include .env

all:
	@echo "There is no default target."

# ==== Main tasks ===

# Re-generate appdata.sqlite3 for the Simsapa desktop app.
bootstrap_database: db_reset db_po_texts db_html_texts db_dict_words db_reindex_fts

# Build simsapa-data dictionaries in all possible formats for Github release uploads.
simsapa_dicts_release:
	./scripts/simsapa_dicts_release.sh $(SIMSAPA_DATA) ../simsapa-dictionary_releases/new-release

# Convert sc-data dictionaries to Markdown for simsapa-data.
sc_dicts_to_md:
	./scripts/sc_dicts_to_md.sh $(SC_PATH) $(SIMSAPA_DATA)

# === Helper tasks ===

po2json:
	./scripts/all_po2json.sh $(SC_PATH) $(PO_TEXT_JSON_PATH)

db_reset:
	rm -f $(DB_PATH) && diesel database --database-url $(DB_PATH) setup

db_reindex_fts:
	cat ./scripts/reindex_fts.sql | sqlite3 $(DB_PATH)

db_po_texts:
	cargo run -- suttacentral_po_texts_to_sqlite \
		--source_path $(SC_PATH) \
		--po_text_json_path $(PO_TEXT_JSON_PATH) \
		--output_path $(DB_PATH) 2>&1 | tee db_po_texts.log

db_html_texts:
	cargo run -- suttacentral_html_texts_to_sqlite \
		--source_path $(SC_PATH) \
		--output_path $(DB_PATH) 2>&1 | tee db_html_texts.log

db_dict_words:
	for i in dhammika dppn ncped nyana pts; do \
		cargo run -- markdown_to_sqlite \
			--source_path "$(SIMSAPA_DATA)/$$i.md" \
			--dict_label $$i \
			--output_path $(DB_PATH) 2>&1 | tee db_dict_words_$$i.log; \
	done; \
	cargo run -- xlsx_to_sqlite \
		--source_path $(SIMSAPA_DATA)/bodhirasa.xlsx \
		--dict_label bodhirasa \
		--output_path $(DB_PATH) 2>&1 | tee db_dict_words_bodhirasa.log

