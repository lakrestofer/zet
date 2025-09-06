--- ==================================================================
--  Document
--- ==================================================================
-- note: some fields contain a blob field meant for json data
-- jsonb encoded json <https://sqlite.org/json1.html#jsonb>. Use the jsonb() function when inserting and reading from this table

-- the unique document itself
create table document (
    -- identifiers
    id blob primary key,           -- uuid primary keys stored as 16 byte blob
    path text not null unique,     -- path to file, relative to zet root
    hash integer not null,         -- file hash, used to detect file changes
    modified text not null,        -- file modified timestamp, used to detect file changes
    created text not null,         -- file created timestamp
    json_data blob                 -- frontmatter data. jsonb encoded json <https://sqlite.org/json1.html#jsonb>. Use the jsonb() function when inserting and reading from this table
) strict;

-- -- a link from a document in the collection to another document in the collection
-- create table document_internal_link (
--     source blob not null references document on delete cascade,
--     target blob references document on delete set null
-- ) strict;

-- -- the document headings
-- create table document_headings (
--     id integer primary key,
--     document_id blob not null references document on delete cascade -- the document foreign key
-- ) strict;

-- create table document_list_item (
--     id integer primary key,
--     document_id blob not null references document on delete cascade, -- the document foreign key
--     checklist boolean not null,
--     checked boolean,
-- ) strict;


-- -- TODO 
-- create virtual table document_fts using fts5(
-- 	path, title, body,
-- 	content = document,
-- 	content_rowid = id,
-- 	tokenize = "porter unicode61 remove_diacritics 1 tokenchars '''&/'"
-- );

