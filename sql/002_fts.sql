--- ==================================================================
--  FTS5 Full-Text Search Support (Contentless)
--- ==================================================================

-- Create contentless FTS5 table
-- content='' means only the index is stored, not the original text
-- contentless_delete=1 allows deletion from the index
CREATE VIRTUAL TABLE document_fts USING fts5(
    title,
    body,
    content='',
    contentless_delete=1,
    tokenize='unicode61 remove_diacritics 1'
);
