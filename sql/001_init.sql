--- ==================================================================
--  Document
--- ==================================================================
-- note: some fields contain a blob field meant for json data
-- jsonb encoded json <https://sqlite.org/json1.html#jsonb>. Use the jsonb() function when inserting and reading from this table

-- the document file itself and information on it
create table document (
    -- identifiers
    id          text    primary key,     -- a slugified version of the relative root path without the .md extension
    path        text    not null unique, -- real relative path
    hash        integer not null,        -- file hash, used to detect file changes
    modified    text    not null,        -- file modified timestamp, used to detect file changes
    created     text    not null,        -- file created timestamp
    frontmatter blob not null            -- frontmatter data. jsonb encoded json <https://sqlite.org/json1.html#jsonb>. Use the jsonb() function when inserting and reading from this table
) strict;

-- the contents of a document
create table node (
    id          integer primary key,
    document_id text    not null,
    parent_id   integer,
    type        text    not null,
    range_start integer not null,
    range_end   integer not null,
    data        blob    not null, -- jsonb encoded data. Dependend on the node type
    foreign key(document_id) references document(id) on delete cascade
) strict;
