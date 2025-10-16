# Zet Architecture

This documents describes implementation details and system design for
the zet tool.

This file is currently incomplete and will be expanded upon
incrementally.

## Core problems

This sections covers the core problems that we need to solve when
creating a zettelkasten tool.

### Data model

### File discovery

- we iterate from he root dir "down" recursively

- libraries handle cyclic directories for us

- how to specify files/directories that we do not want to parse?

- there exists file iterator libraries that support .ignore files.
- we'll just use one of them.
- might pass list of files to ignore through the config file.

### Change detection

- to avoid duplicate work we to figure out which documents we need to
  process.

- compare previously seen files with current ones.
  - compare set of document names
    - not seen ones will be processed
    - no longer seen ones will be deleted
    - seen and previously seen ones will be further compared.
  - compare modification timestamps
    - if timestamp is different from stored one -> has changed.
  - compare content hash
    - and if so, we can further check the hash
    - in case where we made some edit, saved and then reverted it

- this allows us to sync often without it being expensive

### Parsing

- pulldown cmark provides an event and tag stream that we may digest.

### Cli

- we use clap to specify the cli interface
  - we can add custom parsing to the arguments
  - domain specific languages may be validated before the rest of main
    is even triggered!

### File Processing Pipeline

### Configuration System

One config.toml file in .zet.

### Error Handling Strategy

- in case of database errors? Ask the user to delete the db and
  resync, then abort.
- migration issues? None. There shall ever only be one migration. We
  store the schema version and read in on db creation, in case it is
  different. Ask the user to delete their db and resync with the new
  schema.
- parse issue - try to conserve range with invalid markdown.
  - not sure if pulldown does not itself abort here?
- links that do not point to anything? Since we query for the link
  target when we write to the db, we do know on write if the link is
  valid or not.

### Sync vs async?

The cli may be synchronous, but the lsp may not block when performing
its operations.

## Components

This section outlines the components that we use to solve the above
problems. In general these are to be composed in a flat manner. I do
not want to create an "onion" of layers such that any one feature
needs modification on every layer.

For a given feature it should the pick the components it needs. How
this should be structured in terms of modules/imports I do not yet
know.

### DB

The handle around the underlying sqlite `Connection`. Can be used as
the same type that it wraps. When performing queries we do not do them
through a method on our own type, we just use the underlying
`Connection` directly.

When creating the DB, we perform any needed migrations. We also
register a few pragmas.

When dropping the connection, we once again run a few pragmas.

### Parser

Given a string, return AST of the markdown contents.

## Link resolver

- needs the db and a configuration
- depending on configuration we might need to interpret the link
  content differently.
