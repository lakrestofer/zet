use crate::{
    core::{
        db::{DbDelete, DbInsert},
        types::document::DocumentId,
    },
    result::Result,
};
use rusqlite::params;
use serde::{Deserialize, Serialize};
use sql_minifier::macros::minify_sql as sql;

/// A link from one document to another
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentLink {
    pub from: DocumentId,
    pub to: Option<DocumentId>,
    pub range_start: usize,
    pub range_end: usize,
}

impl DbInsert<DocumentLink, i64> for DocumentLink {
    fn insert(db: &mut rusqlite::Connection, values: &[DocumentLink]) -> Result<Vec<i64>> {
        let mut ids = Vec::with_capacity(values.len());

        let tx = db.transaction()?;
        {
            let mut query = tx.prepare(sql!(
                r#"
                insert into document_link (
                    from_id,
                    to_id,
                    range_start,
                    range_stop
                ) values (
                    ?1,
                    ?2,
                    ?3,
                    ?4
                ) returning id;
            "#
            ))?;
            for DocumentLink {
                from,
                to,
                range_start,
                range_end,
            } in values
            {
                ids.push(query.query_row(params![from, to, range_start, range_end], |r| r.get(0))?);
            }
        }
        tx.commit()?;

        Ok(ids)
    }
}

impl DbDelete<DocumentId> for DocumentLink {
    fn delete(db: &mut rusqlite::Connection, ids: &[DocumentId]) -> Result<()> {
        let tx = db.transaction()?;
        {
            let mut query = tx.prepare(sql!("delete from document_link where from_id = ?"))?;

            for id in ids {
                query.execute([id])?;
            }
        }
        tx.commit()?;
        Ok(())
    }
}
