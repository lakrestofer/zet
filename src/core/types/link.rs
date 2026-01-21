use crate::core::{db::DbDelete, types::document::DocumentId};
use serde::{Deserialize, Serialize};
use sql_minifier::macros::minify_sql as sql;

/// A link from one document to another
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentLink {
    pub from: DocumentId,
    pub to: DocumentId,
}

impl DbDelete<DocumentId> for DocumentLink {
    fn delete(db: &mut rusqlite::Connection, ids: Vec<DocumentId>) -> crate::result::Result<()> {
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
