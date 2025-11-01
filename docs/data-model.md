# Data Model

This document describes how zet represents and stores knowledge graph
data.

## Core Entities

### Markdown files

The fundamental unit of the knowledge graph.

**Properties:**

- **ID**: Unique identifier for linking (slugified path or frontmatter
  override)
- **Path**: File path relative to collection root
- **Title**: Display name (from H1 heading or frontmatter)
- **Aliases**: Alternative identifiers from frontmatter

**File Identification:**

1. **Default ID**: Slugified version of file path without extension
   - `notes/todo.md` → ID: `notes/todo`
   - Slugification: `.toLowerCase()` + replace whitespace with `-`
2. **Frontmatter Override**: Custom ID from `id` field
3. **Title Resolution**: First H1 heading OR frontmatter `title` field

### Links

Relationships between files, extracted from markdown link syntax.

**Link Types:**

- **Wiki Links**: `[[target]]` syntax
- **Markdown Links**: `[text](target)` syntax

Both link types can use either ID-based targets or relative path-based
targets.

### AST Elements

Structured representation of markdown content stored for querying.

**AST Storage:**

- **Granularity**: All pulldown-cmark AST nodes
- **Range Format**: Character-based offsets from pulldown-cmark
- **Hierarchy**: Parent-child relationships between AST nodes

**Key AST Element Types:**

- **Headings**: With optional attribute metadata
  `{#class .id key=value}`
- **Lists**: Including task lists and checkboxes
- **Code Blocks**: With language metadata
- **Links**: Both internal and external

## Storage Architecture

### Source of Truth

Markdown files remain the authoritative source. The database serves
purely as a cache for parsed representations and computed
relationships.

### Content Extraction

Instead of storing content in the database, use stored ranges to
extract content from source files:

1. **Query**: Find AST elements matching criteria
2. **Extract**: Use stored ranges to read content from original
   markdown
3. **Display**: Present extracted content with context

## Link Resolution Model

### ID-Based Resolution

Links without file extensions resolve using ID matching:

1. **Exact Match**: Look for file with matching ID
2. **Suffix Match**: If no exact match, find files where ID ends with
   the target
3. **Ambiguity Handling**: Log warnings for multiple matches, pick
   deterministically

### Path-Based Resolution

Links with file extensions require exact relative path matching from
the source file to target.

### Internal vs External Links

- **Internal**: Target resolves to a file in the collection
- **External**: Target points outside the collection

## Frontmatter Integration

### Standard Fields

- `id`: Override default file ID
- `title`: Override H1-derived title
- `aliases`: Array of alternative identifiers

### Custom Metadata

Any additional frontmatter fields are preserved and available for
querying.
