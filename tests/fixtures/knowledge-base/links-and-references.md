# This File Tests Links and References

This document contains various types of markdown links to test the parser's link extraction capabilities.

## Internal Wiki-Style Links

This section demonstrates internal wiki-style links that are commonly used in personal knowledge bases:

- Link to [[index]] page
- Link to [[paragraphs-and-lists]] for list examples
- Link to [[heading-hierarchy]] for heading examples
- Link to a [[non-existent-page]] that doesn't exist yet

## Standard Markdown Links

This paragraph contains a [standard markdown link](https://example.com) that points to an external URL.

Here is another paragraph with [multiple](https://first.com) [different](https://second.com) [links](https://third.com) in sequence.

## Reference-Style Links

This section uses reference-style links: [reference link][ref1] and [another reference][ref2].

[ref1]: https://reference-example.com
[ref2]: https://another-reference.com

## Mixed Link Types in Lists

- Item with [[wiki-link]]
- Item with [markdown link](https://example.org)
- Item with [[another-wiki-link]] and [markdown link](https://test.com) together
  - Nested item with [[nested-wiki-link]]
  - [ ] Task with link to [[task-related-page]]
  - [x] Completed task with [external link](https://completed.com)

## Links in Paragraphs

This paragraph demonstrates links within flowing text. You might reference the [[index]] page, then link to [an external resource](https://resource.com), and then mention [[another-internal-page]] all in the same paragraph.
