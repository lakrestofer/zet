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
    frontmatter blob not null            -- frontmatter data. jsonb encoded json <https://sqlite.org/json1.html#jsonb>. Use the jsonb() function when inserting and reading from this table
) strict;

-- -- to make suffix match on the id and title fields fast, we add a virtual table
-- -- on those columns since a native "suffix"  match does not exist
-- --
-- -- the `content=''` options, makes it such that the table will not store a copy of
-- -- any data we 'insert' into it, it will only store the index.
-- --
-- -- we also need to supply the `rowid` manually
-- create virtual table document_id_title_index using fts5(
--     reversed_id,
--     reversed_title,
--     content='',
--     tokenize='unicode61'
-- );

-- -- we also make sure to keep a few
-- create trigger document_fts5_after_insert after insert on document begin
--     insert into document_id_title_index(
--         rowid,
--         id,
--         title
--     ) values (
--         new.rowid,
--         new.id,
--         new.title
--     );
-- end;



-- create trigger document_fts5_after_delete after delete on document begin
--     delete from document_id_title_index where rowid = old.rowid;
-- end;

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
    data        blob    not null, -- jsonb encoded data. Dependend on the node type
    foreign key(document_id) references document(id) on delete cascade
) strict;
