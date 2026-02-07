use rusqlite::params;
use serde::{Deserialize, Serialize};
use sql_minifier::macros::minify_sql as sql;

use crate::core::db::DbInsert;
use crate::core::types::document::DocumentId;
use crate::result::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tag {
    pub id: i64,
    pub tag: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewTag {
    pub tag: String,
}

#[derive(Debug, Clone)]
pub struct NewDocumentTag {
    pub document_id: DocumentId,
    pub tag: String,
}

impl DbInsert<NewDocumentTag, ()> for NewDocumentTag {
    fn insert(db: &mut rusqlite::Connection, values: &[NewDocumentTag]) -> Result<Vec<()>> {
        let tx = db.transaction()?;
        {
            let mut insert_tag = tx.prepare(sql!(
                r#"INSERT OR IGNORE INTO tag (tag) VALUES (?1)"#
            ))?;
            let mut get_tag_id = tx.prepare(sql!(
                r#"SELECT id FROM tag WHERE tag = ?1"#
            ))?;
            let mut insert_map = tx.prepare(sql!(
                r#"INSERT INTO document_tag_map (document_id, tag_id) VALUES (?1, ?2)"#
            ))?;

            for NewDocumentTag { document_id, tag } in values {
                insert_tag.execute(params![tag])?;
                let tag_id: i64 = get_tag_id.query_row(params![tag], |r| r.get(0))?;
                insert_map.execute(params![document_id, tag_id])?;
            }
        }
        tx.commit()?;

        Ok(vec![(); values.len()])
    }
}
