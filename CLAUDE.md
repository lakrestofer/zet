# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when
working with code in this repository.

Default to using Bun instead of Node.js.

## Project Overview

Zet is a CLI + LSP tool for interacting with markdown-based
zettelkasten systems. It's written in Rust with some TypeScript
components for development tooling.

### Core Architecture

- **Hybrid Language Setup**: Primary application in Rust, with
  TypeScript/Bun for development tooling
- **Database-Centric**: Uses SQLite as a cache/index over markdown
  files, with the markdown files being the source of truth
- **Parser Pipeline**: Built around pulldown-cmark for markdown
  parsing with custom frontmatter handling
- **CLI Interface**: Uses clap for command-line argument parsing

### Key Components

- `src/main.rs` - Entry point, handles CLI command routing
- `src/cli.rs` - CLI interface definition using clap
- `src/parser.rs` - Markdown parsing with frontmatter extraction
- `src/db.rs` - SQLite database operations and schema
- `src/workspace.rs` - Workspace initialization and management
- `src/collection.rs` - Document collection management

## Development Commands

### Rust Development

```bash
# Build the project
cargo build
# Run with debug logging
RUST_LOG=debug cargo run -- <command>
# Run tests
cargo test
# Run tests with output
cargo test -- --nocapture
# Format code
cargo fmt
# Run clippy lints
cargo clippy
```

### Testing Commands

```bash
# Run snapshot tests (uses insta)
cargo test
# Review snapshot changes
cargo insta review
# Accept all snapshot changes
cargo insta accept
```

## Architecture Notes

### Data Model Philosophy

- Markdown files are the permanent data store
- SQLite database is purely a cache/index that can be regenerated
- No data in the database that cannot be reconstructed from markdown
  files
- Uses content hashes and modification timestamps for change detection

### File Processing Pipeline

1. **Discovery**: Find markdown files using `ignore` crate (respects
   .gitignore)
2. **Change Detection**: Compare timestamps and content hashes
3. **Parsing**: Extract frontmatter (TOML format) and markdown content
4. **Database Storage**: Store extracted data for querying

### Configuration

- Configuration stored in `.zet/config.toml`
- Database stored as `.zet/db.sqlite`
- Frontmatter format: TOML (configurable)
