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
    frontmatter blob not null,            -- frontmatter data. jsonb encoded json <https://sqlite.org/json1.html#jsonb>. Use the jsonb() function when inserting and reading from this table
    without rowid
) strict;

-- TODO, what behaviour should we have when target of a link is removed?
create table internal_link (
    node_id integer primary key,      -- the ast node. From this we can extract the location in the source document
    document_id_source text not null, -- the document we are refering from
    document_id_target text,          -- the document we are refering to
    foreign key(node_id) references node(id) on delete cascade,
    foreign key(document_id_source) references document(id) on delete cascade,
    foreign key(document_id_target) references document(id) on delete set null
) strict;

-- the contents of a document
create table node (
    id integer primary key,
    document_id text,
    type text,
    range_start integer,
    range_end integer,
    data blob, -- jsonb encoded data. Dependend on the node type
    foreign key(document_id) references document(id)
) strict;
