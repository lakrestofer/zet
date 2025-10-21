# Link Resolution

This document details how zet resolves different types of links to
target nodes.

## Link Types

Both wiki links and markdown links support the same resolution
mechanisms:

- **Wiki Links**: `[[target]]`
- **Markdown Links**: `[text](target)`

## Resolution Rules

### Extension-Based Distinction

The presence or absence of a file extension determines the resolution
strategy:

#### Links Without Extensions

Links that don't specify a file extension use **ID-based resolution**:

**Examples:**

- `[[todo]]`
- `[link text](todo)`
- `[[notes/project]]`

**Resolution Process:**

1. **Exact Match**: Look for a node with exactly matching ID
2. **Suffix Match**: If no exact match, find nodes where the ID ends
   with the target
3. **Multiple Matches**: Log warning, pick deterministically
   (implementation-defined)

**Suffix Matching Examples:**

- `[[todo]]` can match node with ID `work/todo` (from file
  `work/todo.md`)
- `[[todo]]` can match node with ID `personal/todo` (from file
  `personal/todo.md`)
- If both exist, zet warns about ambiguity and chooses one
  consistently

#### Links With Extensions

Links that include a file extension use **path-based resolution**:

**Examples:**

- `[[../other/file.md]]`
- `[link text](./subfolder/note.md)`
- `[link text](../../parent-dir/note.md)`

**Resolution Process:**

1. **Exact Path Match**: Target must be the correct relative path from
   linking document to target document
2. **No Suffix Matching**: Path must be precisely correct
3. **Validation**: File must exist at the specified relative location

## Node ID System

### Default ID Generation

- File path without extension, relative to collection root
- Slugified using: `.toLowerCase()` + replace whitespace with `-`
- Examples:
  - `notes/todo.md` → ID: `notes/todo`
  - `Projects/Web App.md` → ID: `projects/web-app`

### Frontmatter Override

The `id` field in frontmatter can override the default:

```yaml
---
id: custom-identifier
---
```

### Aliases

Additional identifiers can be defined in frontmatter:

```yaml
---
aliases: [short-name, alt-name]
---
```

Aliases participate in the same resolution process as the primary ID.

## Resolution Examples

### Scenario 1: Simple ID Resolution

**Files:**

- `work/todo.md` (ID: `work/todo`)
- `personal/todo.md` (ID: `personal/todo`)

**Link:** `[[todo]]` **Result:** Matches both, zet warns about
ambiguity and picks one deterministically

### Scenario 2: Specific Path Resolution

**Files:**

- `work/todo.md`
- `personal/todo.md`

**Link from `work/notes.md`:** `[link](todo.md)` **Result:** Resolves
to `work/todo.md` (exact relative path)

### Scenario 3: Alias Resolution

**File:** `research/machine-learning.md`

```yaml
---
id: research/machine-learning
aliases: [ml, ai-research]
---
```

**Links:**

- `[[ml]]` → resolves to `research/machine-learning`
- `[[ai-research]]` → resolves to `research/machine-learning`
- `[[machine-learning]]` → resolves to `research/machine-learning`
  (suffix match)

### Scenario 4: Custom ID Override

**File:** `very/deep/nested/path.md`

```yaml
---
id: important-doc
---
```

**Link:** `[[important-doc]]` **Result:** Resolves to
`very/deep/nested/path.md`

## Internal vs External Links

### Internal Links

Links that successfully resolve to a node within the collection:

- Tracked in the link graph
- Available for graph-based queries
- Validated during parsing

### External Links

Links that don't resolve to internal nodes:

- URLs (e.g., `https://example.com`)
- Non-existent files
- Files outside the collection
- Not tracked in the internal link graph

## Ambiguity Handling

When multiple nodes could match a link target:

1. **Log Warning**: Inform user about the ambiguous link
2. **Deterministic Choice**: Pick one match consistently (e.g.,
   alphabetically first)
3. **Future Enhancement**: LSP could offer "jump point" file creation
   as a code action

This approach allows existing markdown collections to work while
encouraging migration to more specific linking patterns.
