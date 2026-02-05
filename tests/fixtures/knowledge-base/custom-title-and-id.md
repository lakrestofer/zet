---
title: "Custom Title From Frontmatter"
id: "my-custom-document-id"
tags: ["test"]
---

# This Heading Should Not Be Used As Title

This file tests that when both `title` and `id` are specified in the frontmatter, they override the default behavior:

- The title should be "Custom Title From Frontmatter" (from frontmatter), not "This Heading Should Not Be Used As Title" (from the first H1)
- The id should be "my-custom-document-id" (from frontmatter), not the slugified path

## Expected Behavior

When indexed, this document should appear in the database with:
- id: "my-custom-document-id"
- title: "Custom Title From Frontmatter"
