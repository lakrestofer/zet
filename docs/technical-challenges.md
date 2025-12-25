# Technical Challenges

This document outlines the key technical challenges that need to be
considered during the development of zet. They are listed in the order
that they have come up during development. If they had a solution that
is included as well.

## How to store data given that we must be able to construct dynamic queries.

Since we want to support the ability to perform arbitrary queries that
should return data in json format, we need any field of any column of
any table to be interpretable as json. This means that we should only
ever read and write columns to the db that are serialized by the
serde_json crate.

To understand this issue, consider this table describing the
conversion, retrieved from the `serde_json` module of the `rusqlite`
crate.

```
/// Deserialize SQLite value to JSON `Value`:
///
/// | SQLite     | JSON    |
/// |------------|---------|
/// | NULL       | Null    |
/// | 'null'     | Null    |
/// | 'true'     | Bool    |
/// | 1          | Number  |
/// | 0.1        | Number  |
/// | '"text"'   | String  |
/// | 'text'     | _Error_ |
/// | '[0, 1]'   | Array   |
/// | '{"x": 1}' | Object  |
```

This means that we cannot use any of our own serialization code if we
ever want the target type of `r.get(...)?` to be a
`serde_json::Value`. **If we were to write a string directly to a
field, it would not include the required quotation marks, and
therefore not be interpretable as a valid json string**.

A way to solve this issue could be to have any dynamic query be aware
of what fields we are accessing. This feels more complex though.

## Resolve ID

How to generate an ID from the markdown files themselves? Should be no
requirement to have an uuid in the filename, or some enforced
convention. Should the user be able to define their own id?

### Solution

Use a slugified version of the document relative path as the id. When
resolving the target of a link we use suffix matching.

The user should not be able to define their own id (through an id
field in the frontmatter), since this would meant hat link resolution
could not longer be a simple suffix match on the sluggified relative
path of the target.

## Link resolution

how do we match from markdown link syntax to some other file in the
workspace.

Ordinary relative path syntax is common, but can become too long if
the link is in the middle of a paragraph.

Want the ability for a shorter target to resolve to a document that
might be identified by a longer path or id.

Current thoughts on solution in [[./link-resolution.md]]

## Ambiguous Link Resolution

Multiple files may match the same wiki link target (e.g., `[[todo]]`
matching both `work/todo.md` and `personal/todo.md`) because of the
link resolution mechanism.

I think in these cases we should just resolve the target to one of the
files (using some tiebreaker), and somehow log a warning to the user.

## Cache Invalidation Strategy

We need to detect when a file has changed, such that we can reindex
its contents.

We solve this by making comparison on three levels.

- we compare the the currently seen files with the ones we have seen
  before.
  - from this we know which are new, deleted
  - the rest we need to change for modifications
- then we compare file timestamps with the ones stored in the db
  - the ones that differ needs to be reindexed.
- then we compare the hashes
  - since it is cheap to hash the contents, we also compare the hash.

More can be seen in [[./cache-strategy.md]]
