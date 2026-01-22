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
pub struct DocumentHeading {
    pub document_id: DocumentId,
    pub meta_id: Option<String>,
    pub range_start: usize,
    pub range_end: usize,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentHeadingMetaClass {
    pub heading_id: DocumentId,
    pub meta_id: Option<String>,
    pub range_start: usize,
    pub range_end: usize,
    pub content: String,
}
