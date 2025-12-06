use rusqlite::{
    ToSql, params,
    types::{FromSql, FromSqlError, ToSqlOutput},
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sql_minifier::macros::minify_sql as sql;
use std::{path::PathBuf, str::FromStr};
use time::OffsetDateTime;

use crate::core::{db::DbCrud, parser::ast_nodes::NodeKind};
use crate::result::Result;

#[repr(transparent)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentId(pub String);

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
    pub data: JsonData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub id: i64,
    pub document_id: DocumentId,
    pub node_type: NodeKind,
    pub range_start: usize,
    pub range_end: usize,
    pub data: JsonData,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[repr(transparent)]
pub struct JsonData(pub serde_json::Value);

#[derive(Clone, Debug, Serialize, Deserialize)]
#[repr(transparent)]
pub struct DocumentPath(pub PathBuf);

impl Document {
    pub fn new(
        id: DocumentId,
        title: String,
        path: DocumentPath,
        hash: u32,
        modified: ModifiedTimestamp,
        created: CreatedTimestamp,
        data: JsonData,
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

impl Node {
    pub fn new(
        id: i64,
        document_id: DocumentId,
        node_type: NodeKind,
        range_start: usize,
        range_end: usize,
        data: JsonData,
    ) -> Self {
        Self {
            id,
            document_id,
            node_type,
            range_start,
            range_end,
            data,
        }
    }
}

impl From<String> for DocumentId {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl FromSql for NodeKind {
    fn column_result(value: rusqlite::types::ValueRef<'_>) -> rusqlite::types::FromSqlResult<Self> {
        let str_value = value.as_str()?;
        let value = serde_json::from_str(str_value).map_err(|_| FromSqlError::InvalidType)?;
        Ok(value)
    }
}

impl ToSql for NodeKind {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        match serde_json::to_string(self) {
            Ok(str) => Ok(str.into()),
            Err(_) => panic!("Could not serialize NodeKind as string"),
        }
    }
}

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

impl FromSql for JsonData {
    fn column_result(value: rusqlite::types::ValueRef<'_>) -> rusqlite::types::FromSqlResult<Self> {
        let s = value.as_str()?;
        match serde_json::from_str(s) {
            Ok(v) => Ok(JsonData(v)),
            Err(_) => Err(FromSqlError::InvalidType),
        }
    }
}

impl ToSql for JsonData {
    fn to_sql(&self) -> rusqlite::Result<rusqlite::types::ToSqlOutput<'_>> {
        Ok(self.0.to_string().into())
    }
}

impl From<JsonData> for Value {
    fn from(value: JsonData) -> Self {
        value.0
    }
}

impl DbCrud<Document, DocumentId> for Document {
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

    fn upsert(db: &mut rusqlite::Connection, value: Vec<Document>) -> Result<Vec<DocumentId>> {
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
                    ?2,        -- path     (text)
                    ?3,        -- hash     (integer)
                    ?4,        -- modified (text)
                    ?5,        -- created  (text)
                    jsonb(?6)
                ) on conflict(
                    id
                ) do update set
                    path        = ?2,
                    hash        = ?3,
                    modified    = ?4,
                    created     = ?5,
                    frontmatter = jsonb(?6)                    
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

    fn delete(db: &mut rusqlite::Connection, ids: Vec<DocumentId>) -> Result<()> {
        todo!()
    }
}

impl DbCrud<Node, i64> for Node {
    fn list(db: &rusqlite::Connection) -> Result<Vec<Node>> {
        db.prepare(sql!(
            r#"
                select
                    id,
                    document_id,
                    type,
                    range_start,
                    range_end,
                    json(data)
                    
                from
                    node
                "#
        ))?
        .query_map([], |r| {
            Ok(Node {
                id: r.get(0)?,
                document_id: r.get(1)?,
                node_type: r.get(2)?,
                range_start: r.get(3)?,
                range_end: r.get(4)?,
                data: r.get(5)?,
            })
        })?
        .map(|f| f.map_err(From::from))
        .collect::<Result<Vec<Node>>>()
    }

    fn get(db: &mut rusqlite::Connection, id: i64) -> Result<Node> {
        todo!()
    }

    fn upsert(db: &mut rusqlite::Connection, nodes: Vec<Node>) -> Result<Vec<i64>> {
        let mut ids = Vec::new();
        let tx = db.transaction()?;
        {
            let mut query = tx.prepare(sql!(
                r#"
                insert into
                    node
                values (
                    ?1,        -- id          (integer)
                    ?2,        -- document_id (text)
                    ?3,        -- type        (text)
                    ?4,        -- range_start (integer)
                    ?5,        -- range_end   (integer)
                    jsonb(?6)  -- data        (blob)
                ) on conflict(
                    id
                ) do update set
                     document_id = ?2,
                     type        = ?3,
                     range_start = ?4,
                     range_end   = ?5,
                     data        = jsonb(?6)
                "#
            ))?;
            for n in nodes {
                let id = query
                    .query_row(
                        params![
                            n.id,
                            n.document_id,
                            n.node_type,
                            n.range_start,
                            n.range_end,
                            n.data
                        ],
                        |r| r.get(0),
                    )
                    .unwrap();

                ids.push(id);
            }
        }
        tx.commit()?;

        Ok(ids)
    }

    fn delete(db: &mut rusqlite::Connection, ids: Vec<i64>) -> Result<()> {
        todo!()
    }
}
