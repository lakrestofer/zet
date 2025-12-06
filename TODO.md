# TODO

- what are we doing right now?

- document model
  - data stored in db
    - [ ] document
      - [x] id
      - [x] title
      - [x] path
      - [x] data necessary for change detection
    - [ ] ast
    - [ ] internal collection links
- document id resolver
  - given a written link `[[some-link]]`, we need some function that
    maps it to an actual document.
  - [ ] str -> document_id using "suffix match"
    - [ ] suffix search in db
      - [x] fts5 table for reversed id and reversed title
      - [ ] triggers for keeping it in sync
      - [ ] add function for reversing_string
    - for resolving links
  - [ ] str -> document_ids using "contains match"
    - for future lsp where we interactively want to search for
      document titles
  - [ ] document title -> id and path.
    - if the user prefers

- [ ] indexing
  - [x] parsing
  - [x] change detection
  - [x] mapping to db
  - [x] writes to db
- [ ] db modeling
- [ ] query
  - [ ] raw sql query
    - [ ] sanitize query for destructive operations
  - domain specific language
    - probably an sql dialect in the end
  - [ ] output
    - what should be the output format of a query?
    - [ ] json
    - [ ] template, is probably the way to describe the markdown
          output as well
    - [ ] markdown
      - to allow us to create "views" into other documents
      - could have a query that returns all todos over the collections
- [ ] note creation
  - [ ] create note
  - [ ] template

- tests
  - we need a good testing strategy
    - for parsing snapshot tests are quite good
    - for the application overall I want tests to have the most bang
      for the buck.

## stuff to improve

- [x] errors, mix of this_error any eyre, should probably just use
      eyre.
- [ ] right now the pipeline for taking a document and turning it into
      a representation that we insert into the db. is a bit spread out
- [ ] add `--force` flag to the index flow
- move where range and children is stored on the ast_nodes
  - lot of duplicated fields
- [ ] does not make sense for Document to be a Node type
