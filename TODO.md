# Query Interface Implementation Plan

## Phase 1: Analyze Parser Output
**Goal**: Understand what AST information is available from pulldown-cmark

- [ ] Check pulldown-cmark `Event` enum documentation to see all available event types
- [ ] Create test script to parse sample markdown and inspect raw pulldown-cmark output
- [ ] Identify what data we can extract: headings (with depth), links (wiki/markdown), list items (checkboxes), code blocks, etc.
- [ ] Note: Current parser in `src/parser.rs` is incomplete, we need to understand the raw pulldown-cmark output first

## Phase 2: Design Database Schema
**Goal**: Model the parsed AST data in relational tables

- [ ] Based on parser capabilities, design tables for:
  - [ ] Headings (document_id, level, text, range)
  - [ ] Links (source_doc, target_doc/url, link_type, range)
  - [ ] List items (document_id, is_task, is_checked, range)
  - [ ] Other queryable elements as needed
- [ ] Keep ranges as character offsets for content extraction
- [ ] Use foreign keys to `document` table

## Phase 3: Implement SQL Migration
**Goal**: Update `001_init.sql` with new schema

- [ ] Create tables in migration file
- [ ] Add appropriate indexes for common queries
- [ ] Ensure proper cascading deletes when documents are removed

## Phase 4: AST-to-Database Persistence
**Goal**: Populate tables when parsing documents

- [ ] Extend collection parsing logic to insert AST elements
- [ ] Create query functions in `src/db/query` module for insertions
- [ ] Integrate with existing parse workflow

## Phase 5: Add Query CLI Command
**Goal**: Create `zet query <sql>` command

- [ ] Add `Query { sql: String }` variant to `Command` enum in `cli.rs`
- [ ] Wire up command to execute SQL against database

## Phase 6: SQL Sanitization
**Goal**: Ensure only SELECT queries are allowed

- [ ] Parse SQL string to detect statement type
- [ ] Allow only SELECT statements
- [ ] Reject INSERT, UPDATE, DELETE, DROP, etc.
- [ ] Return clear error messages for invalid queries

## Phase 7: Testing
**Goal**: Verify end-to-end functionality

- [ ] Test with sample markdown files
- [ ] Run various SELECT queries against parsed data
- [ ] Verify content extraction works correctly
