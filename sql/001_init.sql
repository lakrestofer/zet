--- ==================================================================
--  Document
--- ==================================================================
-- note: some fields contain a blob field meant for json data
-- jsonb encoded json <https://sqlite.org/json1.html#jsonb>. Use the jsonb() function when inserting and reading from this table

-- the document file itself and information on it
create table document (
    -- identifiers
    id          text    primary key,     -- a slugified version of the relative root path without the .md extension
    title       text    not null,
    path        text    not null unique, -- real relative path
    hash        integer not null,        -- file hash, used to detect file changes
    modified    text    not null,        -- file modified timestamp, used to detect file changes
    created     text    not null,        -- file created timestamp
    frontmatter blob                     -- frontmatter data. jsonb encoded json <https://sqlite.org/json1.html#jsonb>. Use the jsonb() function when inserting and reading from this table
) strict;


--- ==================================================================
--  Link
--- ==================================================================

create table link (
    node_id integer not null, -- the ast node for this link, tells us where in the document the link is
    from_id text not null,
    to_id text,
    foreign key (from_id) references document(id) on delete cascade,
    foreign key (to_id) references document(id) on delete set null,
    foreign key (node_id) references node(id) on delete cascade
) strict;

--- ==================================================================
--  Node
--- ==================================================================
create table node (
    id          integer primary key,
    document_id text    not null,
    parent_id   integer,
    type        text    not null,
    range_start integer not null,
    range_end   integer not null,
    data        blob, -- jsonb encoded data. Dependend on the node type
    foreign key(document_id) references document(id) on delete cascade
    foreign key(parent_id) references node(id) on delete cascade
) strict;
