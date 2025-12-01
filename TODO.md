# TODO

- I'm currently working on the indexing feature
  - I have the change detection working right now

Now I should start on the basic query implementation

- [ ] take the list of nodes and insert them into the db
  - [ ] map them to format expected by db
- [ ] RawQuery command that we sanitize for any destructive operations

## stuff to improve

- [ ] errors, mix of this_error any eyre, should probably just use
      eyre.

- [ ] right now the pipeline for taking a document and turning it into
      a representation that we insert into the db.
