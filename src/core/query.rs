use jiff::Timestamp;
use rusqlite::Connection;
use rusqlite::types::Value;

use crate::result::Result;

use super::types::document::{
    CreatedTimestamp, Document, DocumentId, DocumentPath, ModifiedTimestamp,
};

#[derive(Debug, Clone, Copy)]
pub enum SortByOption {
    Modified,
    Created,
    Id,
    Path,
    Title,
}

#[derive(Debug, Clone, Copy)]
pub enum SortOrder {
    Ascending,
    Descending,
}

#[derive(Debug, Clone, Copy, Default)]
pub enum MatchStrategy {
    #[default]
    Fts,
    Exact,
}

#[derive(Debug, Default)]
pub struct DocumentQuery {
    pub ids: Vec<String>,
    pub titles: Vec<String>,
    pub paths: Vec<String>,
    pub tags: Vec<String>,
    pub tagless: bool,
    pub exclude_ids: Vec<String>,
    pub exclude_paths: Vec<String>,
    pub created: Option<Timestamp>,
    pub modified: Option<Timestamp>,
    pub created_before: Option<Timestamp>,
    pub created_after: Option<Timestamp>,
    pub modified_before: Option<Timestamp>,
    pub modified_after: Option<Timestamp>,
    pub links_to: Vec<String>,
    pub links_from: Vec<String>,
    pub match_pattern: Option<String>,
    pub match_strategy: MatchStrategy,
    pub order_by: Vec<(SortByOption, SortOrder)>,
    pub limit: Option<usize>,
}

impl DocumentQuery {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_ids(mut self, ids: Vec<String>) -> Self {
        self.ids = ids;
        self
    }

    pub fn with_titles(mut self, titles: Vec<String>) -> Self {
        self.titles = titles;
        self
    }

    pub fn with_paths(mut self, paths: Vec<String>) -> Self {
        self.paths = paths;
        self
    }

    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }

    pub fn tagless(mut self) -> Self {
        self.tagless = true;
        self
    }

    pub fn exclude_ids(mut self, ids: Vec<String>) -> Self {
        self.exclude_ids = ids;
        self
    }

    pub fn exclude_paths(mut self, paths: Vec<String>) -> Self {
        self.exclude_paths = paths;
        self
    }

    pub fn created(mut self, ts: Timestamp) -> Self {
        self.created = Some(ts);
        self
    }

    pub fn modified(mut self, ts: Timestamp) -> Self {
        self.modified = Some(ts);
        self
    }

    pub fn created_before(mut self, ts: Timestamp) -> Self {
        self.created_before = Some(ts);
        self
    }

    pub fn created_after(mut self, ts: Timestamp) -> Self {
        self.created_after = Some(ts);
        self
    }

    pub fn modified_before(mut self, ts: Timestamp) -> Self {
        self.modified_before = Some(ts);
        self
    }

    pub fn modified_after(mut self, ts: Timestamp) -> Self {
        self.modified_after = Some(ts);
        self
    }

    pub fn links_to(mut self, ids: Vec<String>) -> Self {
        self.links_to = ids;
        self
    }

    pub fn links_from(mut self, ids: Vec<String>) -> Self {
        self.links_from = ids;
        self
    }

    pub fn with_match(mut self, pattern: String, strategy: MatchStrategy) -> Self {
        self.match_pattern = Some(pattern);
        self.match_strategy = strategy;
        self
    }

    pub fn order_by(mut self, by: SortByOption, order: SortOrder) -> Self {
        self.order_by.push((by, order));
        self
    }

    pub fn limit(mut self, n: usize) -> Self {
        self.limit = Some(n);
        self
    }

    pub fn execute(self, db: &Connection) -> Result<Vec<Document>> {
        let mut sql = String::from(
            r#"SELECT DISTINCT d.id, d.title, d.path, d.hash, d.modified, d.created, json(d.frontmatter), d.body
FROM document d
WHERE 1=1"#,
        );
        let mut params: Vec<Value> = Vec::new();

        // --id filter
        if !self.ids.is_empty() {
            let placeholders = generate_placeholders(self.ids.len());
            sql.push_str(&format!(" AND d.id IN ({placeholders})"));
            params.extend(self.ids.into_iter().map(Value::from));
        }

        // --title filter
        if !self.titles.is_empty() {
            let placeholders = generate_placeholders(self.titles.len());
            sql.push_str(&format!(" AND d.title IN ({placeholders})"));
            params.extend(self.titles.into_iter().map(Value::from));
        }

        // --path filter (suffix match)
        for path in &self.paths {
            sql.push_str(" AND d.path LIKE ?");
            params.push(Value::from(format!("%{}", path)));
        }

        // --tag filter (AND semantics: document must have ALL specified tags)
        for tag in &self.tags {
            sql.push_str(
                " AND EXISTS (SELECT 1 FROM document_tag_map m JOIN tag t ON m.tag_id = t.id WHERE m.document_id = d.id AND LOWER(t.tag) = LOWER(?))",
            );
            params.push(Value::from(tag.clone()));
        }

        // --tagless filter
        if self.tagless {
            sql.push_str(
                " AND NOT EXISTS (SELECT 1 FROM document_tag_map m WHERE m.document_id = d.id)",
            );
        }

        // --exclude filter
        if !self.exclude_ids.is_empty() {
            let placeholders = generate_placeholders(self.exclude_ids.len());
            sql.push_str(&format!(" AND d.id NOT IN ({placeholders})"));
            params.extend(self.exclude_ids.into_iter().map(Value::from));
        }

        // --exclude-by-path filter (suffix match)
        for path in &self.exclude_paths {
            sql.push_str(" AND d.path NOT LIKE ?");
            params.push(Value::from(format!("%{}", path)));
        }

        // --created filter (exact date match)
        if let Some(ts) = self.created {
            sql.push_str(" AND date(d.created) = date(?)");
            params.push(Value::from(ts.to_string()));
        }

        // --modified filter (exact date match)
        if let Some(ts) = self.modified {
            sql.push_str(" AND date(d.modified) = date(?)");
            params.push(Value::from(ts.to_string()));
        }

        // --created-before filter
        if let Some(ts) = self.created_before {
            sql.push_str(" AND d.created < ?");
            params.push(Value::from(ts.to_string()));
        }

        // --created-after filter
        if let Some(ts) = self.created_after {
            sql.push_str(" AND d.created > ?");
            params.push(Value::from(ts.to_string()));
        }

        // --modified-before filter
        if let Some(ts) = self.modified_before {
            sql.push_str(" AND d.modified < ?");
            params.push(Value::from(ts.to_string()));
        }

        // --modified-after filter
        if let Some(ts) = self.modified_after {
            sql.push_str(" AND d.modified > ?");
            params.push(Value::from(ts.to_string()));
        }

        // --links-to filter (documents that link TO any of these IDs)
        if !self.links_to.is_empty() {
            let placeholders = generate_placeholders(self.links_to.len());
            sql.push_str(&format!(
                " AND EXISTS (SELECT 1 FROM document_link l WHERE l.from_id = d.id AND l.to_id IN ({placeholders}))"
            ));
            params.extend(self.links_to.into_iter().map(Value::from));
        }

        // --links-from filter (documents that are linked FROM any of these IDs)
        if !self.links_from.is_empty() {
            let placeholders = generate_placeholders(self.links_from.len());
            sql.push_str(&format!(
                " AND EXISTS (SELECT 1 FROM document_link l WHERE l.to_id = d.id AND l.from_id IN ({placeholders}))"
            ));
            params.extend(self.links_from.into_iter().map(Value::from));
        }

        // --match filter (full-text search or exact match)
        if let Some(pattern) = &self.match_pattern {
            match self.match_strategy {
                MatchStrategy::Fts => {
                    sql.push_str(
                        " AND d.rowid IN (SELECT rowid FROM document_fts WHERE document_fts MATCH ?)",
                    );
                    params.push(Value::from(pattern.clone()));
                }
                MatchStrategy::Exact => {
                    sql.push_str(" AND (d.title LIKE ? OR d.body LIKE ?)");
                    let like_pattern = format!("%{}%", pattern);
                    params.push(Value::from(like_pattern.clone()));
                    params.push(Value::from(like_pattern));
                }
            }
        }

        // ORDER BY
        if !self.order_by.is_empty() {
            sql.push_str(" ORDER BY ");
            let order_clauses: Vec<String> = self
                .order_by
                .iter()
                .map(|(by, order)| {
                    let col = match by {
                        SortByOption::Modified => "d.modified",
                        SortByOption::Created => "d.created",
                        SortByOption::Id => "d.id",
                        SortByOption::Path => "d.path",
                        SortByOption::Title => "d.title",
                    };
                    let dir = match order {
                        SortOrder::Ascending => "ASC",
                        SortOrder::Descending => "DESC",
                    };
                    format!("{col} {dir}")
                })
                .collect();
            sql.push_str(&order_clauses.join(", "));
        }

        // LIMIT
        if let Some(n) = self.limit {
            sql.push_str(&format!(" LIMIT {n}"));
        }

        log::debug!("executing query: {}", sql);
        log::debug!("with params: {:?}", params);

        let mut stmt = db.prepare(&sql)?;
        let params_slice: Vec<&dyn rusqlite::ToSql> =
            params.iter().map(|v| v as &dyn rusqlite::ToSql).collect();

        let documents = stmt
            .query_map(params_slice.as_slice(), |r| {
                Ok(Document::new(
                    r.get::<_, DocumentId>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, DocumentPath>(2)?,
                    r.get::<_, u32>(3)?,
                    r.get::<_, ModifiedTimestamp>(4)?,
                    r.get::<_, CreatedTimestamp>(5)?,
                    r.get::<_, serde_json::Value>(6)?,
                    r.get::<_, String>(7)?,
                ))
            })?
            .map(|r| r.map_err(From::from))
            .collect::<Result<Vec<Document>>>()?;

        Ok(documents)
    }
}

fn generate_placeholders(count: usize) -> String {
    vec!["?"; count].join(", ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_placeholders() {
        assert_eq!(generate_placeholders(1), "?");
        assert_eq!(generate_placeholders(3), "?, ?, ?");
        assert_eq!(generate_placeholders(0), "");
    }
}
