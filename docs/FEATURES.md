# Zet Features

User-facing features and capabilities of the zettelkasten tool.

## CLI Commands

### Collection Management
- `zet init` - Initialize a new zettelkasten collection in current directory
- `zet sync` - Index/reindex all markdown files in the collection
- `zet status` - Show collection statistics and health

### Note Operations
- `zet create <title>` - Create a new note with optional template
- `zet create --template=<name> <title>` - Create note with specific template
- `zet rename <old-name> <new-name>` - Rename note and update all references

### Querying & Search
- `zet search <text>` - Full-text search across all notes
- `zet query --sql "<query>"` - Direct SQL queries against the indexed database
- `zet links <note>` - Show incoming and outgoing links for a note
- `zet orphans` - List notes with no incoming links
- `zet broken` - List broken/dead links in the collection

### Content Analysis
- `zet tags` - List all tags found in frontmatter across the collection
- `zet stats` - Collection statistics (note count, word count, etc.)

## LSP Features

### Navigation
- **Go to Definition**: Follow wikilinks `[[note-name]]` and markdown links to target notes
- **Hover**: Preview linked note content on hover
- **Auto-completion**: Suggest note names when typing `[[` for wikilinks

### Code Actions
- **Create Note from Selection**: Select text and create a new note with that content
- **Rename Note**: Rename a note and automatically update all references
- **Convert Link Format**: Convert between wikilinks and markdown links

### Diagnostics
- **Dead Link Detection**: Highlight links that point to non-existent notes
- **Orphaned Notes**: Warning for notes with no incoming references
- **Malformed Frontmatter**: Errors for invalid YAML frontmatter

## Link Formats

### Wikilinks
- `[[note-title]]` - Link by note title (preferred)
- `[[note-title|display text]]` - Link with custom display text

### Markdown Links
- `[text](path/to/note.md)` - Relative path links
- `[text](note-title.md)` - File name links (resolved across collection)

### Link Resolution
1. Exact title match (case-sensitive)
2. Slugified title match (`"Some Note"` → `some-note.md`)
3. Relative file path resolution
4. Collection-wide file name search

## Templates

### Template Variables
- `{{title}}` - Note title
- `{{date}}` - Current date (ISO format)
- `{{date:YYYY-MM-DD}}` - Formatted date
- `{{uuid}}` - Generated UUID for the note

### Template Selection
- **Automatic**: Based on note location and collection config
- **Manual**: `zet create --template=meeting "Weekly Standup"`
- **Default**: Falls back to basic template if none specified

## Metadata Support

### Frontmatter
```yaml
---
title: "Custom Title"
tags: ["project", "important"]
aliases: ["alt-name", "another-name"]
date: 2024-01-01
---
```

### Heading Metadata
```markdown
# Project Planning { #planning .important priority=high }
```

### Special Fields
- `title`: Overrides filename/first heading for link resolution
- `aliases`: Alternative names for linking to this note
- `tags`: Categorization and filtering
- Custom fields: Stored as-is for SQL querying

## Query Examples

### Simple CLI Queries
```bash
# Find notes mentioning "obsidian"
zet search "obsidian"

# Show all project-related notes
zet query --sql "SELECT title FROM notes WHERE json_extract(frontmatter, '$.tags') LIKE '%project%'"

# Find orphaned notes
zet orphans

# Show broken links
zet broken
```

### Advanced SQL Queries
```sql
-- Notes created in the last week
SELECT title, created_at FROM notes
WHERE created_at > datetime('now', '-7 days');

-- Most linked-to notes
SELECT target_title, COUNT(*) as link_count
FROM links
GROUP BY target_title
ORDER BY link_count DESC;

-- Notes without tags
SELECT title FROM notes
WHERE json_extract(frontmatter, '$.tags') IS NULL;
```

## Configuration

### Collection Config (`config.toml`)
```toml
[collection]
root = "."
templates_dir = "templates"
default_template = "basic"

[templates]
meeting = "templates/meeting.md"
project = "templates/project.md"

[indexing]
extensions = [".md", ".markdown"]
ignore_patterns = ["_drafts/*", "archive/*"]
```

### Template Directory Structure
```
templates/
├── basic.md
├── meeting.md
├── project.md
└── daily.md
```