pub mod document;

pub mod node {
    use rusqlite::{
        ToSql, params, params_from_iter,
        types::{FromSql, FromSqlError, ToSqlOutput},
    };
    use serde::{Deserialize, Serialize};
    use sql_minifier::macros::minify_sql as sql;
    use std::{path::PathBuf, str::FromStr};
    use time::OffsetDateTime;

    use crate::core::{
        db::{DbDelete, DbGet, DbInsert, DbList, DbUpdate},
        parser::ast_nodes::NodeKind,
        types::document::DocumentId,
    };
    use crate::result::Result;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Node {
        pub id: i64,
        pub document_id: DocumentId,
        pub node_type: NodeKind,
        pub range_start: usize,
        pub range_end: usize,
        pub data: serde_json::Value,
    }

    impl Node {
        pub fn new(
            id: i64,
            document_id: DocumentId,
            node_type: NodeKind,
            range_start: usize,
            range_end: usize,
            data: serde_json::Value,
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

    impl FromSql for NodeKind {
        fn column_result(
            value: rusqlite::types::ValueRef<'_>,
        ) -> rusqlite::types::FromSqlResult<Self> {
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
}
