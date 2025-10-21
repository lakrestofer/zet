# Technical Challenges

This document outlines the key technical challenges that need to be
considered during the development of zet.

## AST Storage and Querying

### Challenge: AST Granularity vs Performance

- **Problem**: Storing every pulldown-cmark AST node provides maximum
  query flexibility but may impact performance and storage size
- **Considerations**:
  - Balance between query capabilities and database size
  - Efficient queries for structural patterns (e.g., "documents with
    headings more than 4 levels deep")
  - Memory usage when loading large document ASTs

### Challenge: SQL Query Interface for AST

- **Problem**: Exposing AST structure through SQL requires careful
  schema design
- **Considerations**:
  - How to represent hierarchical AST relationships in relational
    tables
  - Efficient queries for extracting elements (e.g., "all checkmarks
    grouped by document")
  - Query performance for common structural queries

## Link Resolution and Graph Management

### Challenge: Ambiguous Link Resolution

- **Problem**: Multiple nodes may match the same wiki link target
  (e.g., `[[todo]]` matching both `work/todo.md` and
  `personal/todo.md`)
- **Considerations**:
  - Deterministic resolution algorithms for suffix matching
  - Warning users about ambiguous links
  - Performance of suffix matching across collections

## Cache Invalidation Strategy

### Challenge: Three-Tier Change Detection

- **Problem**: Efficiently detecting when files need re-parsing using
  the path/timestamp/hash approach
- **Considerations**:
  - Performance of hash computation for large files
  - Handling timestamp changes that don't reflect content changes
  - Balancing detection accuracy with computational cost

### Challenge: AST Range Invalidation

- **Problem**: When files change, stored character-based ranges become
  invalid
- **Considerations**:
  - Detecting which ranges are affected by file changes
  - Re-parsing vs incremental range updates
  - Maintaining range accuracy for content extraction

## Node Identification System

### Challenge: ID vs Path-Based Linking

- **Problem**: Supporting both automatic path-based IDs and
  frontmatter ID overrides
- **Considerations**:
  - Handling conflicts between slugified paths and custom IDs
  - Maintaining link integrity when IDs change
  - Migration behavior when files don't follow ID conventions

### Challenge: Alias Resolution

- **Problem**: Multiple aliases pointing to the same node complicates
  link resolution
- **Considerations**:
  - Preventing alias conflicts across nodes
  - Performance impact of checking multiple aliases during resolution
  - User feedback when aliases create ambiguity

## Full-Text + Structural Query Integration

### Challenge: Combining Query Types

- **Problem**: Users want to combine full-text search with structural
  queries (e.g., "find 'machine learning' in documents with TODO
  items")
- **Considerations**:
  - Query execution order optimization
  - How to efficiently intersect results from different search types
  - Unclear how these two systems will interact effectively
