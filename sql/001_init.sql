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

create table document_link (
    id integer primary key,
    from_id text not null,
    to_id text,
    range_start integer not null,
    range_end integer not null,
    foreign key (from_id) references document(id) on delete cascade,
    foreign key (to_id) references document(id) on delete set null
) strict;


--- ==================================================================
--  Heading
--- ==================================================================

create table document_heading (
    id integer primary key,
    document_id text not null,
    content text not null,
    level integer not null,
    metadata blob not null,
    range_start integer not null,
    range_end integer not null,
    foreign key (document_id) references document(id) on delete cascade
) strict;

--- ==================================================================
--  Tasks
--- ==================================================================

create table document_task (
    id integer primary key,
    document_id text not null,
    checked integer not null, -- rusqlite converts booleans to integers
    content text not null,
    range_start integer not null,
    range_end integer not null,
    foreign key (document_id) references document(id) on delete cascade
) strict;


--- ==================================================================
--  Triggers - Clear linked data on document hash update
--- ==================================================================

-- Clear all links from a document when its hash changes
create trigger clear_links_on_hash_update
after update of hash on document
for each row
begin
    delete from document_link where from_id = NEW.id;
end;

-- Clear all headings from a document when its hash changes
create trigger clear_headings_on_hash_update
after update of hash on document
for each row
begin
    delete from document_heading where document_id = NEW.id;
end;

