--- ============================ document ============================
-- the unique document itself
create table document (
    -- identifiers
    id blob primary key,           -- uuid primary keys stored as 16 byte blob
    path text not null unique,     -- path to file, relative to zet root
    hash integer not null,         -- file hash, used to detect file changes
    modified text not null,        -- file modified timestamp, used to detect file changes
    created text not null          -- file created timestamp, used to detect file changes
) strict;

-- any content in the frontmatter of the document
create table document_metadata (
    id integer primary key,        -- primary key
    document_id blob not null,     -- the document foreign key
    json_data blob not null        -- jsonb encoded json <https://sqlite.org/json1.html#jsonb>. Use the jsonb() function when inserting and reading from this table
) strict
