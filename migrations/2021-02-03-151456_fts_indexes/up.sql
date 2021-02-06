-- === Index tables ===

CREATE VIRTUAL TABLE fts_root_texts USING fts5 (
  content=root_texts,
  content_rowid=id,
  content_plain
);

CREATE VIRTUAL TABLE fts_translated_texts USING fts5 (
  content=translated_texts,
  content_rowid=id,
  content_plain
);

CREATE VIRTUAL TABLE fts_meanings USING fts5 (
  content=meanings,
  content_rowid=id,
  definition_md,
  summary
);

-- === Triggers to keep content synced ===

-- root_texts

CREATE TRIGGER root_texts_ai AFTER INSERT ON root_texts BEGIN
  INSERT INTO fts_root_texts
    (rowid, content_plain)
    VALUES
    (new.id, new.content_plain);
END;

CREATE TRIGGER root_texts_ad AFTER DELETE ON root_texts BEGIN
  INSERT INTO fts_root_texts
    (fts_root_texts, rowid, content_plain)
    VALUES
    ('delete', old.id, old.content_plain);
END;

CREATE TRIGGER root_texts_au AFTER UPDATE ON root_texts BEGIN
  INSERT INTO fts_root_texts
    (fts_root_texts, rowid, content_plain)
    VALUES
    ('delete', old.id, old.content_plain);
  INSERT INTO fts_root_texts
    (rowid, content_plain)
    VALUES
    (new.id, new.content_plain);
END;

-- translated_texts

CREATE TRIGGER translated_texts_ai AFTER INSERT ON translated_texts BEGIN
  INSERT INTO fts_translated_texts
    (rowid, content_plain)
    VALUES
    (new.id, new.content_plain);
END;

CREATE TRIGGER translated_texts_ad AFTER DELETE ON translated_texts BEGIN
  INSERT INTO fts_translated_texts
    (fts_translated_texts, rowid, content_plain)
    VALUES
    ('delete', old.id, old.content_plain);
END;

CREATE TRIGGER translated_texts_au AFTER UPDATE ON translated_texts BEGIN
  INSERT INTO fts_translated_texts
    (fts_translated_texts, rowid, content_plain)
    VALUES
    ('delete', old.id, old.content_plain);
  INSERT INTO fts_translated_texts
    (rowid, content_plain)
    VALUES
    (new.id, new.content_plain);
END;

-- meanings

CREATE TRIGGER meanings_ai AFTER INSERT ON meanings BEGIN
  INSERT INTO fts_meanings
    (rowid, definition_md, summary)
    VALUES
    (new.id, new.definition_md, new.summary);
END;

CREATE TRIGGER meanings_ad AFTER DELETE ON meanings BEGIN
  INSERT INTO fts_meanings
    (fts_meanings, rowid, definition_md, summary)
    VALUES
    ('delete', old.id, old.definition_md, old.summary);
END;

CREATE TRIGGER meanings_au AFTER UPDATE ON meanings BEGIN
  INSERT INTO fts_meanings
    (fts_meanings, rowid, definition_md, summary)
    VALUES
    ('delete', old.id, old.definition_md, old.summary);
  INSERT INTO fts_meanings
    (rowid, definition_md, summary)
    VALUES
    (new.id, new.definition_md, new.summary);
END;

