# Terminology

This document defines the key terms used throughout the zet project to
ensure consistent understanding and implementation.

## Core Concepts

### Collection

A directory containing markdown files (with nested folders) that
represents a user's knowledge base. Collections are the top-level
organizational unit and define the scope of a workspace.

### Node

The basic unit of the knowledge graph. Currently, a node corresponds
to a single markdown file. Nodes are the entities that can be linked
to and queried.

### Node Path

The file path of a node relative to the collection root, including the
file extension.

- Example: `notes/todo.md`, `projects/web-app/architecture.md`

### Node Title

The human-readable name of a node, determined by:

1. The `title` field in frontmatter (if present)
2. The content of the first H1 heading (if present)
3. Used primarily for display purposes in interactive interfaces

### Node ID

The unique identifier used for linking to nodes, determined by:

1. The `id` field in frontmatter (if present)
2. The slugified version of the node path without extension (default)

- Example: `notes/todo.md` → ID: `notes/todo`
- Slugification rules: `.toLowerCase()` and replace whitespace with
  `-`

### Aliases

Additional identifiers for a node defined in frontmatter that resolve
to the same node. Allows for shorter or alternative references in
links.

- Example: A node with ID `research/machine-learning/neural-networks`
  might have alias `nn`

## Link Types

### Wiki Links

Links using double bracket syntax: `[[target]]`

- Target resolution uses suffix matching against node IDs
- Example: `[[todo]]` matches node with ID `notes/todo`

### Markdown Links

Standard markdown links: `[text](target.md)`

- When target includes extension, requires exact relative path match
- Example: `[link](../other/file.md)` must be exact relative path

### Internal vs External Links

- **Internal Link**: Target resolves to another node in the collection
- **External Link**: Target points outside the collection (URLs,
  non-existent files, etc.)

## Storage Concepts

### AST (Abstract Syntax Tree)

The parsed representation of markdown content stored in the database
cache. Uses granularity and format from the pulldown-cmark parsing
library.

### Range

Character-based offsets (from pulldown-cmark) that define the location
of AST elements within the source markdown file. Used to extract
content without re-parsing.

### Cache

The SQLite database that stores parsed AST, node metadata, and link
relationships. The source of truth remains the markdown files
themselves.

## Frontmatter

YAML or TOML metadata blocks at the beginning of markdown files that
can override default behavior:

```yaml
---
id: custom-identifier
title: Custom Display Title
aliases: [short-name, alt-name]
tags: [research, important]
---
```

## Metadata Syntax Extensions

### Heading Attributes

Metadata can be attached to headings using attribute syntax:

```markdown
# Some Heading {#class .id attr key=value}
```

This allows for rich metadata on structural elements beyond just
frontmatter.
