use crate::{
    core::{
        db::{DbDelete, DbInsert, DbList},
        types::{RangeEnd, RangeStart, document::DocumentId},
    },
    result::Result,
};
use rusqlite::{ToSql, params, types::FromSql};
use serde::{Deserialize, Serialize};
use sql_minifier::macros::minify_sql as sql;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentLinkId(i64);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentLinkSource(DocumentId);
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentLinkTarget(DocumentId);

/// A link from one document to another
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentLink {
    pub id: DocumentLinkId,
    pub from: DocumentLinkSource,
    pub to: Option<DocumentLinkTarget>,
    pub range_start: RangeStart,
    pub range_end: RangeEnd,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewDocumentLink {
    pub from: DocumentLinkSource,
    pub to: Option<DocumentLinkTarget>,
    pub range_start: RangeStart,
    pub range_end: RangeEnd,
}

impl DbInsert<NewDocumentLink, i64> for DocumentLink {
    fn insert(db: &mut rusqlite::Connection, values: &[NewDocumentLink]) -> Result<Vec<i64>> {
        let mut ids = Vec::with_capacity(values.len());

        let tx = db.transaction()?;
        {
            let mut query = tx.prepare(sql!(
                r#"
                insert into document_link (
                    from_id,
                    to_id,
                    range_start,
                    range_end
                ) values (
                    ?1,
                    ?2,
                    ?3,
                    ?4
                ) returning id;
            "#
            ))?;
            for NewDocumentLink {
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

impl DbList<DocumentLinkId> for DocumentLink {
    fn list(db: &rusqlite::Connection) -> Result<Vec<DocumentLinkId>> {
        db.prepare(sql!("select id from document_link"))?
            .query_map([], |r| r.get(0))?
            .map(|f| f.map_err(From::from))
            .collect::<Result<Vec<DocumentLinkId>>>()
    }
}

impl ToSql for DocumentLinkId {
    fn to_sql(&self) -> rusqlite::Result<rusqlite::types::ToSqlOutput<'_>> {
        self.0.to_sql()
    }
}

impl FromSql for DocumentLinkId {
    fn column_result(value: rusqlite::types::ValueRef<'_>) -> rusqlite::types::FromSqlResult<Self> {
        Ok(DocumentLinkId(value.as_i64()?))
    }
}

impl From<DocumentId> for DocumentLinkSource {
    fn from(value: DocumentId) -> Self {
        Self(value)
    }
}
impl From<DocumentId> for DocumentLinkTarget {
    fn from(value: DocumentId) -> Self {
        Self(value)
    }
}

impl ToSql for DocumentLinkSource {
    fn to_sql(&self) -> rusqlite::Result<rusqlite::types::ToSqlOutput<'_>> {
        self.0.to_sql()
    }
}
impl FromSql for DocumentLinkSource {
    fn column_result(value: rusqlite::types::ValueRef<'_>) -> rusqlite::types::FromSqlResult<Self> {
        Ok(DocumentLinkSource(DocumentId::column_result(value)?))
    }
}
impl ToSql for DocumentLinkTarget {
    fn to_sql(&self) -> rusqlite::Result<rusqlite::types::ToSqlOutput<'_>> {
        self.0.to_sql()
    }
}
impl FromSql for DocumentLinkTarget {
    fn column_result(value: rusqlite::types::ValueRef<'_>) -> rusqlite::types::FromSqlResult<Self> {
        Ok(DocumentLinkTarget(DocumentId::column_result(value)?))
    }
}
