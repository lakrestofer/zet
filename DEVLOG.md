# Devlog

This document contains an append only log documenting my work on this
tool.

## 06 Saturday

- 19:32
  - I'm currently implementing a function for retrieving all documents
    that match some prefix.
  - might actually skip it now though.
  - I need id suffix matching for "start writing document title" ->
    "full document id" autocomplete.
    - sooooo, for the LSP, and not for the cli
    - for the lsp some in-memory datastructure might be better (as
      long as it is memory efficient).
  - I shall now continue on the "Raw query" instead

## 25 Thursday

- 18:41
  - while starting on the raw query part I found an issue
    - a rather simple way to implement a raw query is to simply pass
      the query directly to the db and then iterate over every column
      returned and parse it using `serde_json`.
  - this makes me consider if I should even have a raw query
    functionality.
