# Zet Development Plan

## Foundation - focus on cli tool only

We establish the foundation of our tool

- document model and db schema
  - what is our conceptual model that we make concrete as actual data?
  - how is this modeled in the db schema?
- parsing
  - how do we transform a document into this representation
- how to we handle updates?
  - implement
- configuration

### Implementation plan

- init command
  - create .zet dir
  - create initial db.sqlite
  - create initial config file

- collect all paths under root
  - [ ] all paths
  - [ ] all paths except those in .ignore files

- save all documents under root in db
  - [ ] only save path, modification timestamp, hash

- make change detection algorithm work
  - consider path changes
    - new files
    - deleted files
  - consider modification timestamps
  - consider hash

- add more tables for other data
  - [ ] links
