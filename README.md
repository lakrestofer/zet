# zet

A developer-focused zettelkasten CLI tool that treats directories of markdown files as queryable databases.

## Overview

Zet enables **discoverability** in plaintext markdown collections by parsing files into a transparent database cache. Query your notes for structural patterns ("documents with TODOs"), perform full-text searches, and extract structured elements (checkboxes, headings) - all while keeping your markdown files as the source of truth.

**Key Features:**
- **Database-like queries** on markdown directory structures
- **Structural queries** for syntax patterns (heading depth, TODO items, etc.)
- **Full-text search** across your collection
- **Link graph analysis** with wiki-style and markdown linking
- **Scriptable CLI** designed for developer workflows
- **No lock-in** - markdown files remain the authoritative source

## Target Audience

Developers who want to automate and script interactions with their markdown-based knowledge bases. Perfect for those who prefer plaintext workflows but need better discoverability than traditional file-based organization.

## Core Concepts

- **Collections**: Directories containing markdown files (your knowledge base)
- **Nodes**: Individual markdown files, each with an ID for linking
- **Links**: Both `[[wiki-style]]` and `[markdown](links)` between files
- **Cache**: SQLite database storing parsed AST and relationships (rebuilds from files)

## Architecture

Zet is built as a single Rust binary with two modes:
- **CLI mode**: For queries, management, and scripting
- **LSP mode**: Future editor integration (`zet lsp --stdin`)

The core library handles parsing (pulldown-cmark), caching (SQLite), and link resolution for both modes.

## Documentation

For detailed information, see the [docs/](./docs/) directory:

- **[Terminology](./docs/terminology.md)** - Key concepts and definitions
- **[Data Model](./docs/data-model.md)** - How nodes, links, and AST are represented
- **[Link Resolution](./docs/link-resolution.md)** - How different link types resolve to targets
- **[Cache Strategy](./docs/cache-strategy.md)** - Three-tier change detection system
- **[Technical Challenges](./docs/technical-challenges.md)** - Implementation considerations

## Current Status

This project is in early development. The basic parsing and caching infrastructure exists, with ongoing work on query interfaces and link resolution.

## Development

See [CLAUDE.md](./CLAUDE.md) for development commands and project structure information.
