---
title = "This File Tests Frontmatter Metadata"
tags = ["testing", "frontmatter", "metadata", "toml"]
created = 2024-01-15T10:30:00Z
modified = 2024-01-20T15:45:00Z
author = "Test Suite"
status = "draft"
---

# This File Contains TOML Frontmatter

The frontmatter above this heading contains metadata in TOML format that should be parsed separately from the document content.

## What This File Tests

This file verifies that the parser can:

- Extract TOML frontmatter from the document header
- Parse the frontmatter as separate metadata
- Continue parsing the markdown content after the frontmatter
- Handle various frontmatter field types (strings, arrays, dates)

## Content After Frontmatter

This content should be parsed normally as markdown, completely separate from the frontmatter metadata above.

- List item one
- List item two
- [ ] Task item
- [x] Completed task

## Links in Frontmatter File

This file also contains [[wiki-links]] and [markdown links](https://example.com) to verify that link parsing works correctly even when frontmatter is present.

The frontmatter should not interfere with any of the standard markdown parsing capabilities demonstrated throughout this knowledge base fixture.
