--- ==================================================================
--  FTS5 Full-Text Search Support
--- ==================================================================

-- Add body column to store document text for FTS
ALTER TABLE document ADD COLUMN body TEXT NOT NULL DEFAULT '';

-- Create FTS5 virtual table with external content
CREATE VIRTUAL TABLE document_fts USING fts5(
    title,
    body,
    content='document',
    content_rowid='rowid',
    tokenize='unicode61 remove_diacritics 1'
);

-- Triggers for automatic sync between document table and FTS index

-- Insert trigger: add new document to FTS index
CREATE TRIGGER document_fts_insert AFTER INSERT ON document BEGIN
    INSERT INTO document_fts(rowid, title, body) VALUES (NEW.rowid, NEW.title, NEW.body);
END;

-- Delete trigger: remove document from FTS index
CREATE TRIGGER document_fts_delete AFTER DELETE ON document BEGIN
    INSERT INTO document_fts(document_fts, rowid, title, body) VALUES ('delete', OLD.rowid, OLD.title, OLD.body);
END;

-- Update trigger: update document in FTS index
CREATE TRIGGER document_fts_update AFTER UPDATE ON document BEGIN
    INSERT INTO document_fts(document_fts, rowid, title, body) VALUES ('delete', OLD.rowid, OLD.title, OLD.body);
    INSERT INTO document_fts(rowid, title, body) VALUES (NEW.rowid, NEW.title, NEW.body);
END;
