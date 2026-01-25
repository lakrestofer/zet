use rusqlite::params;
use serde::{Deserialize, Serialize};
use sql_minifier::macros::minify_sql as sql;

use crate::core::{
    db::DbInsert,
    types::{RangeEnd, RangeStart, document::DocumentId},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentTask {
    pub id: i64,
    pub document_id: DocumentId,
    pub checked: bool,
    pub content: String,
    pub range_start: RangeStart,
    pub range_end: RangeEnd,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewDocumentTask {
    pub document_id: DocumentId,
    pub checked: bool,
    pub content: String,
    pub range_start: RangeStart,
    pub range_end: RangeEnd,
}

impl DbInsert<NewDocumentTask, i64> for DocumentTask {
    fn insert(
        db: &mut rusqlite::Connection,
        values: &[NewDocumentTask],
    ) -> crate::result::Result<Vec<i64>> {
        let tx = db.transaction()?;
        let mut ids = Vec::with_capacity(values.len());
        {
            let mut query = tx.prepare(sql!(
                r#"
                insert into document_task (
                    document_id,
                    checked,
                    content,
                    range_start,
                    range_end
                ) values (
                    ?1,
                    ?2,
                    ?3,
                    ?4,
                    ?5
                ) returning id;
            "#
            ))?;
            for task in values {
                let id = query.query_row(
                    params![
                        task.document_id,
                        task.checked,
                        task.content,
                        task.range_start,
                        task.range_end,
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
