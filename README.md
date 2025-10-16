# zet

Your markdown zettelkasten assistant.

## Description

Zet aims to be a cli + lsp for interacting with your markdown based
zettelkasten.

## Goals

- improve discoverability of your markdown documents
  - it is very easy to accidentally create several small notes, that
    are not really engaged with anymore after creation.
  - If if create a note, I want the act of creating that note to be a
    measure of me engaging in some particular thought, idea, fact more
    in the future.
  - When a note exists for the purpose of being expanded upon, one
    needs to be reminded that it exists.
  - solution?
    - a better querying interface over all documents
      - MOOC exist but require certain conventions.
        - maintenance
        - "top down" structuring
- the only persistent data is the markdown files themselves
  - this means no links to other documents using data that cannot be
    fully reconstructed after deleting `db.sqlite`.
  - one could have the requirement that every document needs a
    front-matter that stores their id.
    - unsure if i want that requirement.
- db only as a caching mechanism, api surface.
  - the db is not meant to be stored forever, but the collection of
    markdown files is.
- the directory of markdown files as a db.
  - the markdown files has content that is is interesting to extract
    and display to the user.
- do not force the user to format their documents in a certain way
- lsp integration for common everyday task such as following links,
  creating new notes from selections etc.
  - follow definition -> follow link to some other note
    - create new note if selected text does not have a def
  - actions
    - rename note
- query language over markdown documents
  - simple first version would be allowing direct sql queries over the
    db.
  - sync -> information about documents exist in queryable format ->
    query

- cli tools
  - initialize new directory
  - create new note
  - sync directory

## Example queries

- plaintext content
  - full text search
    - "which documents mention 'obsidian'"
- information about links
  - are there any dead links?
  - which notes reference a certain other note
    - incoming links
    - outgoing links
  - which external sites are mentioned?
- information about file metadata
  - last edited file?
  - sort documents by access date
- information about metadata (internal to document)
  - frontmatter
    - which documents have the "#project" tag? (find thoughts about
      some project)
    - which documents do not have a certain tag? (find notes that have
      not been categorized)
  - heading metadata (requires extension in pulldown-cmark)
    - any heading with "#flashcard" tag

## Possible feature list

- templating for note creation
  - with some builtin variables `{{}}`
    - title
    - created

- rename note
  - as a command
  - as a automatic step after sync
    - how can we detect that a given file has only been renamed?

- lsp features

  - code actions
    - create note from selection
    - goto definition (follow link)
    - goto references (follow backlinks)
      - cursor anywhere inside file
