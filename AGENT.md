# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when
working with code in this repository.

## Common Development Commands

### Building and Running

- `cargo build` - Build the project
- `cargo run -- --help` - Show CLI help
- `cargo run -- init [path]` - Initialize a new zet workspace
- `cargo run -- parse <file>` - Parse a markdown file

### Testing

- `cargo test` - Run all tests

### Code Quality

- `cargo fmt` - Format code
- `cargo clippy` - Run linter
- `cargo check` - Quick syntax check

### Documentation and Formatting

- `dprint fmt` - Format markdown files (configured in
  .helix/languages.toml)

## Architecture Overview

This is a Personal Knowledge Management (PKM) CLI tool written in Rust
that parses markdown files and stores them in a SQLite database.

### Core Modules

- **parser/** - Markdown parsing with frontmatter support (TOML
  format)
  - `parser/ast_nodes.rs` - AST node definitions
  - `parser.rs` - Main parsing logic using pulldown-cmark
- **db.rs** - SQLite database operations with rusqlite
- **workspace.rs** - Workspace initialization and management
- **collection.rs** - Document collection management
- **cli.rs** - Command-line interface using clap

### Key Dependencies

- `clap` - CLI argument parsing with derive macros
- `rusqlite` - SQLite database with time/uuid features
- `pulldown-cmark` - Markdown parsing with SIMD acceleration
- `gray_matter` - Frontmatter parsing
- `insta` - Snapshot testing for parser verification
- `color-eyre` - Enhanced error handling

### Workspace Structure

- `.zet/` - Hidden directory containing workspace data
- `.zet/db.sqlite` - SQLite database for storing parsed documents
- Workspace root is resolved by walking up directory tree to find
  `.zet/`

### Testing Strategy

Uses snapshot testing with `insta` crate for parser verification. Test
files are in `tests/input_files/` with corresponding snapshots in
`tests/snapshots/`.

## important-instruction-reminders

Do what has been asked; nothing more, nothing less. NEVER create files
unless they're absolutely necessary for achieving your goal. ALWAYS
prefer editing an existing file to creating a new one. NEVER
proactively create documentation files (*.md) or README files. Only
create documentation files if explicitly requested by the User.
