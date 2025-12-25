use crate::core::{
    db::{DbDelete, DbGet, DbInsert, DbList, DbUpdate},
    parser::ast_nodes::NodeKind,
};
use std::{path::PathBuf, str::FromStr};

use crate::result::Result;
use rusqlite::{
    ToSql, params,
    types::{FromSql, FromSqlError, ToSqlOutput},
};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

use sql_minifier::macros::minify_sql as sql;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[repr(transparent)]
pub struct DocumentPath(pub PathBuf);

impl From<String> for DocumentId {
    fn from(value: String) -> Self {
        Self(value)
    }
}

#[repr(transparent)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentId(pub String);

impl FromSql for DocumentId {
    fn column_result(value: rusqlite::types::ValueRef<'_>) -> rusqlite::types::FromSqlResult<Self> {
        let s: String = value.as_str()?.to_owned();
        Ok(DocumentId(s))
    }
}

impl ToSql for DocumentId {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        Ok(self.0.to_owned().into())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ModifiedTimestamp(pub OffsetDateTime);
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CreatedTimestamp(pub OffsetDateTime);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub id: DocumentId,
    pub title: String,
    pub path: DocumentPath,
    pub hash: u32,
    pub modified: ModifiedTimestamp,
    pub created: CreatedTimestamp,
    pub data: serde_json::Value,
}

impl Document {
    pub fn new(
        id: DocumentId,
        title: String,
        path: DocumentPath,
        hash: u32,
        modified: ModifiedTimestamp,
        created: CreatedTimestamp,
        data: serde_json::Value,
    ) -> Self {
        Self {
            id,
            title,
            path,
            hash,
            modified,
            created,
            data,
        }
    }
}

impl DbList<Document> for Document {
    fn list(db: &rusqlite::Connection) -> Result<Vec<Document>> {
        db.prepare(sql!(
            r#"
                select
                    id,
                    title,
                    path,
                    hash,
                    modified,
                    created,
                    json(frontmatter) as frontmatter
                from
                    document
                "#
        ))?
        .query_map([], |r| {
            Ok(Document::new(
                r.get(0)?,
                r.get(1)?,
                r.get(2)?,
                r.get(3)?,
                r.get(4)?,
                r.get(5)?,
                r.get(6)?,
            ))
        })?
        .map(|f| f.map_err(From::from))
        .collect::<Result<Vec<Document>>>()
    }
}
impl DbGet<DocumentId, Document> for Document {
    fn get(db: &mut rusqlite::Connection, id: DocumentId) -> Result<Document> {
        Ok(db
            .prepare(sql!(
                r#"
            select
                id,
                path,
                hash,
                modified,
                created,
                json(frontmatter) as frontmatter
            from
                document
            where
                id = ?
        "#
            ))?
            .query_row([id], |r| {
                Ok(Document::new(
                    r.get(0)?,
                    r.get(1)?,
                    r.get(2)?,
                    r.get(3)?,
                    r.get(4)?,
                    r.get(5)?,
                    r.get(6)?,
                ))
            })?)
    }
}

impl DbInsert<Document, DocumentId> for Document {
    fn insert(db: &mut rusqlite::Connection, value: Vec<Document>) -> Result<Vec<DocumentId>> {
        log::debug!("inserting {} documents", value.len());
        let ids = Vec::with_capacity(value.len());
        let tx = db.transaction()?;
        {
            let query_str = sql!(
                r#"
                insert into
                    document
                values (
                    ?1,        -- id       (text)
                    ?2,        -- title    (text)
                    ?3,        -- path     (text)
                    ?4,        -- hash     (integer)
                    ?5,        -- modified (text)
                    ?6,        -- created  (text)
                    jsonb(?7)
                );
                "#
            );
            let mut query = tx.prepare(query_str)?;

            for d in value {
                query.execute(params![d.id, d.path, d.hash, d.modified, d.created, d.data])?;
            }
        }
        tx.commit()?;
        Ok(ids)
    }
}

impl DbUpdate<Document, DocumentId> for Document {
    fn update(db: &mut rusqlite::Connection, value: Vec<Document>) -> Result<Vec<DocumentId>> {
        log::debug!("upserting {} documents", value.len());
        let ids = Vec::with_capacity(value.len());
        let tx = db.transaction()?;
        {
            let query_str = sql!(
                r#"
                insert into
                    document
                values (
                    ?1,        -- id       (text)
                    ?2,        -- title    (text)
                    ?3,        -- path     (text)
                    ?4,        -- hash     (integer)
                    ?5,        -- modified (text)
                    ?6,        -- created  (text)
                    jsonb(?7)
                ) on conflict(
                    id
                ) do update set
                    title       = ?2,
                    path        = ?3,
                    hash        = ?4,
                    modified    = ?5,
                    created     = ?6,
                    frontmatter = jsonb(?7)                    
                "#
            );
            let mut query = tx.prepare(query_str)?;
            for d in value {
                query.execute(params![d.id, d.path, d.hash, d.modified, d.created, d.data])?;
            }
        }
        tx.commit()?;
        Ok(ids)
    }
}

impl DbDelete<DocumentId> for Document {
    fn delete(db: &mut rusqlite::Connection, ids: Vec<DocumentId>) -> Result<()> {
        let tx = db.transaction()?;
        {
            let query_str = sql!(r#"delete from document where id = ?1"#);
            let mut query = tx.prepare(query_str)?;
            for id in ids {
                query.execute(params![id])?;
            }
        }
        tx.commit()?;
        Ok(())
    }
}

impl FromSql for ModifiedTimestamp {
    fn column_result(value: rusqlite::types::ValueRef<'_>) -> rusqlite::types::FromSqlResult<Self> {
        Ok(ModifiedTimestamp(OffsetDateTime::column_result(value)?))
    }
}
impl FromSql for CreatedTimestamp {
    fn column_result(value: rusqlite::types::ValueRef<'_>) -> rusqlite::types::FromSqlResult<Self> {
        Ok(CreatedTimestamp(OffsetDateTime::column_result(value)?))
    }
}
impl ToSql for ModifiedTimestamp {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        self.0.to_sql()
    }
}
impl ToSql for CreatedTimestamp {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        self.0.to_sql()
    }
}

impl FromSql for DocumentPath {
    fn column_result(value: rusqlite::types::ValueRef<'_>) -> rusqlite::types::FromSqlResult<Self> {
        let s = value.as_str()?;
        let path = PathBuf::from_str(s).map_err(|_| FromSqlError::InvalidType)?;
        Ok(DocumentPath(path))
    }
}

impl From<DocumentPath> for PathBuf {
    fn from(value: DocumentPath) -> PathBuf {
        value.0
    }
}
impl ToSql for DocumentPath {
    fn to_sql(&self) -> rusqlite::Result<rusqlite::types::ToSqlOutput<'_>> {
        Ok(self.0.to_string_lossy().into_owned().into())
    }
}
