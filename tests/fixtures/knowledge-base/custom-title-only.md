---
title: "Title Only From Frontmatter"
tags: ["test"]
---

# This Heading Should Be Ignored

This file tests that when only `title` is specified in the frontmatter:

- The title should be "Title Only From Frontmatter" (from frontmatter)
- The id should be the slugified path (default behavior)

## Expected Behavior

The title comes from frontmatter but the id is generated from the file path.
