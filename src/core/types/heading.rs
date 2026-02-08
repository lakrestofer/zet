use crate::{
    core::{
        db::{DbInsert, DbList},
        types::document::DocumentId,
    },
    result::Result,
};
use rusqlite::params;
use serde::{Deserialize, Serialize};
use sql_minifier::macros::minify_sql as sql;

// a heading in a given document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentHeading {
    pub id: i64,
    pub document_id: DocumentId,
    pub content: String,
    pub metadata: serde_json::Value,
    pub range_start: usize,
    pub range_end: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewDocumentHeading {
    pub document_id: DocumentId,
    pub content: String,
    pub level: u8,
    pub metadata: serde_json::Value,
    pub range_start: usize,
    pub range_end: usize,
}

impl DbInsert<NewDocumentHeading, i64> for DocumentHeading {
    fn insert(db: &mut rusqlite::Connection, headings: &[NewDocumentHeading]) -> Result<Vec<i64>> {
        let tx = db.transaction()?;
        let mut ids = Vec::with_capacity(headings.len());
        {
            let mut query = tx.prepare(sql!(
                r#"
                insert into document_heading (
                    document_id,
                    content,
                    level,
                    metadata,
                    range_start,
                    range_end
                ) values (
                    ?1,
                    ?2,
                    ?3,
                    jsonb(?4),
                    ?5,
                    ?6
                ) returning id;
            "#
            ))?;

            for h in headings {
                let id = query.query_row(
                    params![
                        h.document_id,
                        h.content,
                        h.level,
                        h.metadata,
                        h.range_start,
                        h.range_end,
                    ],
                    |r| r.get(0),
                )?;
                ids.push(id);
            }
        }
        tx.commit()?;
        Ok(ids)
    }
}

impl DbList<DocumentHeading> for DocumentHeading {
    fn list(db: &rusqlite::Connection) -> Result<Vec<DocumentHeading>> {
        db.prepare(sql!(
            r#"
            select
                id,
                document_id,
                content,
                json(metadata) as metadata,
                range_start,
                range_end
            from
                document_heading
            "#
        ))?
        .query_map([], |r| {
            Ok(DocumentHeading {
                id: r.get(0)?,
                document_id: r.get(1)?,
                content: r.get(2)?,
                metadata: r.get(3)?,
                range_start: r.get(4)?,
                range_end: r.get(5)?,
            })
        })?
        .map(|f| f.map_err(From::from))
        .collect::<Result<Vec<DocumentHeading>>>()
    }
}
