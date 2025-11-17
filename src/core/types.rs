use rusqlite::{
    ToSql, params,
    types::{FromSql, FromSqlError, ToSqlOutput},
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sql_minifier::macros::minify_sql as sql;
use std::{path::PathBuf, str::FromStr};
use time::OffsetDateTime;

use crate::core::db::DbCrud;
use crate::result::Result;

#[repr(transparent)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentId(pub String);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NodeKind {
    Document,
    NotImplemented,
    Heading,
    Paragraph,
    BlockQuote,
    Text,
    TextDecoration,
    Html,
    FootnoteReference,
    FootnoteDefinition,
    DefinitionList,
    DefinitionListTitle,
    DefinitionListDefinition,
    InlineLink,
    ReferenceLink,
    ShortcutLink,
    AutoLink,
    WikiLink,
    LinkReference,
    InlineImage,
    ReferenceImage,
    List,
    Item,
    TaskListMarker,
    SoftBreak,
    HardBreak,
    Code,
    CodeBlock,
    HorizontalRule,
    Table,
    TableHead,
    TableRow,
    TableCell,
    MetadataBlock,
    DisplayMath,
    InlineMath,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ModifiedTimestamp(pub OffsetDateTime);
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CreatedTimestamp(pub OffsetDateTime);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub id: DocumentId,
    pub path: DocumentPath,
    pub hash: u32,
    pub modified: ModifiedTimestamp,
    pub created: CreatedTimestamp,
    pub data: JsonData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InternalLink {
    pub node_id: u64,
    pub document_id_source: DocumentId,
    pub document_id_target: DocumentId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub id: u64,
    pub document_id: DocumentId,
    pub node_type: NodeKind,
    pub range_start: usize,
    pub range_end: usize,
    pub data: JsonData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[repr(transparent)]
pub struct JsonData(pub serde_json::Value);

#[derive(Clone, Debug, Serialize, Deserialize)]
#[repr(transparent)]
pub struct DocumentPath(pub PathBuf);

impl Document {
    pub fn new(
        id: DocumentId,
        path: DocumentPath,
        hash: u32,
        modified: ModifiedTimestamp,
        created: CreatedTimestamp,
        data: JsonData,
    ) -> Self {
        Self {
            id,
            path,
            hash,
            modified,
            created,
            data,
        }
    }
}

impl InternalLink {
    pub fn new(
        node_id: u64,
        document_id_source: DocumentId,
        document_id_target: DocumentId,
    ) -> Self {
        Self {
            node_id,
            document_id_source,
            document_id_target,
        }
    }
}

impl Node {
    pub fn new(
        id: u64,
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
                ))
            })?
            .map(|f| f.map_err(From::from))
            .collect::<Result<Vec<Document>>>()?)
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
                    ?1,
                    ?2,
                    ?3,
                    ?4,
                    ?5,
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

    fn delete(db: &mut rusqlite::Connection, id: DocumentId) -> Result<()> {
        todo!()
    }
}
