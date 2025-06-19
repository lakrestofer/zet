pub mod preamble {
    pub use crate::error_handling::Result;
    pub use crate::tables::*;
    pub use crate::wrapper::DB;

    pub(crate) use crate::types::*;
    pub use rusqlite::Connection;
}

pub mod types {
    //! This module contains some wrapper structs for some common types that we
    //! want to write to and read from our db. It is a workaround for the typical
    //! "cannot implement trait for foreign type" issue.

    use std::{path::PathBuf, str::FromStr};

    use rusqlite::{
        ToSql,
        types::{FromSql, FromSqlError, ToSqlOutput},
    };
    use serde::{Deserialize, Serialize};
    use serde_json::Value;

    #[derive(Debug, Serialize, Deserialize)]
    #[repr(transparent)]
    pub struct JsonData(pub serde_json::Value);

    impl FromSql for JsonData {
        fn column_result(
            value: rusqlite::types::ValueRef<'_>,
        ) -> rusqlite::types::FromSqlResult<Self> {
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

    #[derive(Debug, Serialize, Deserialize)]
    #[repr(transparent)]
    pub struct PathBufContainer(pub PathBuf);

    impl FromSql for PathBufContainer {
        fn column_result(
            value: rusqlite::types::ValueRef<'_>,
        ) -> rusqlite::types::FromSqlResult<Self> {
            let s = value.as_str()?;
            let path = PathBuf::from_str(s).map_err(|_| FromSqlError::InvalidType)?;
            Ok(PathBufContainer(path))
        }
    }
    impl From<PathBufContainer> for PathBuf {
        fn from(value: PathBufContainer) -> PathBuf {
            value.0
        }
    }
    impl ToSql for PathBufContainer {
        fn to_sql(&self) -> rusqlite::Result<rusqlite::types::ToSqlOutput<'_>> {
            Ok(ToSqlOutput::Owned(rusqlite::types::Value::Text(
                self.0.to_string_lossy().into_owned(),
            )))
        }
    }
}

pub mod error_handling {
    use thiserror::Error;

    #[derive(Error, Debug)]
    pub enum Error {
        #[error("rusqlite error: {0}")]
        RusqliteError(rusqlite::Error),
        #[error("migration error: {0}")]
        MigrationError(rusqlite_migration::Error),
    }

    pub type Result<T, E = Error> = std::result::Result<T, E>;

    impl From<rusqlite::Error> for Error {
        fn from(value: rusqlite::Error) -> Self {
            Self::RusqliteError(value)
        }
    }
    impl From<rusqlite_migration::Error> for Error {
        fn from(value: rusqlite_migration::Error) -> Self {
            Self::MigrationError(value)
        }
    }
}

pub mod wrapper {
    use crate::preamble::*;
    use rusqlite::Connection;
    use sql_minifier::macros::load_sql;
    use std::{
        cell::LazyCell,
        ops::{Deref, DerefMut},
        path::Path,
    };

    use rusqlite_migration::{M, Migrations};

    const DB_OPEN: &str = load_sql!("sql/db_open.sql");
    const DB_CLOSE: &str = load_sql!("sql/db_close.sql");

    const MIGRATIONS: LazyCell<Migrations> =
        LazyCell::new(|| Migrations::new(vec![M::up(load_sql!("sql/001_init.sql"))]));

    #[repr(transparent)]
    pub struct DB(Connection);

    impl DB {
        pub fn open<P: AsRef<Path> + std::fmt::Debug>(path: P) -> Result<DB> {
            log::debug!("opening db at {:?}", path);
            // open and create a sqlite db
            let mut conn = Connection::open(path)?;

            conn.execute_batch(DB_OPEN)?;

            MIGRATIONS.to_latest(&mut conn)?;

            Ok(DB(conn))
        }
    }
    // util traits
    impl Drop for DB {
        fn drop(&mut self) {
            self.0.execute_batch(DB_CLOSE).unwrap();
        }
    }
    impl Deref for DB {
        type Target = Connection;

        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
    impl DerefMut for DB {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.0
        }
    }
}

pub mod tables {
    use std::path::PathBuf;
    use time::OffsetDateTime;
    use uuid::Uuid;

    pub struct Document {
        pub id: Uuid,
        pub path: PathBuf,
        pub hash: u64,
        pub modified: OffsetDateTime,
        pub created: OffsetDateTime,
    }

    pub struct DocumentMetadata {
        pub id: u64,
        pub document_id: Uuid,
        pub modified: OffsetDateTime,
        /// NOTE: field in the db is blob, meaning that the sqlite jsonb and
        /// json functions needs to used
        pub json_data: serde_json::Value,
    }
}

pub mod query {
    pub mod document {
        //! this module contains some simple CRUD functions for interacting with
        //! the document sqlite table.
        //! When possible, instead use the helpers in the query::utils module to
        //! implement your feature
        use rusqlite::params;
        use sql_minifier::macros::minify_sql;
        use std::path::PathBuf;
        use time::OffsetDateTime;
        use uuid::Uuid;

        use crate::{preamble::*, types::PathBufContainer};

        pub fn list(conn: &mut Connection) -> Result<Vec<Document>> {
            let mut query = conn.prepare(minify_sql!(
                r#"select id, path, hash, modified, created from document"#
            ))?;
            let result = query
                .query_map([], |r| {
                    Ok(Document {
                        id: r.get(0)?,
                        path: (r.get::<usize, String>(1)?).into(),
                        hash: r.get(2)?,
                        modified: r.get(3)?,
                        created: r.get(4)?,
                    })
                })?
                .map(|f| f.map_err(From::from))
                .collect::<Result<Vec<Document>>>()?;
            Ok(result)
        }
        pub fn get(conn: &mut Connection) -> Result<Document> {
            todo!()
        }
        pub fn create(
            conn: &mut Connection,
            path: PathBuf,
            hash: u64,
            modified: OffsetDateTime,
            created: OffsetDateTime,
        ) -> Result<Uuid> {
            let mut query = conn.prepare(minify_sql!(
                r#"insert into document (id, path, hash, modified, created) values (?,?,?,?,?) returning id"#
            ))?;
            let res = query.query_row(
                params![
                    Uuid::new_v4(),
                    PathBufContainer(path),
                    hash,
                    modified,
                    created
                ],
                |r| r.get(0),
            )?;
            Ok(res)
        }
        pub fn update(conn: &mut Connection) -> Result<Vec<Document>> {
            Ok(vec![])
        }
        pub fn delete(conn: &mut Connection) -> Result<Vec<Document>> {
            Ok(vec![])
        }

        #[cfg(test)]
        pub mod tests {
            use super::*;

            #[test]
            fn create_then_fetch() {
                let mut db = DB::open(":memory:").unwrap();
                let id = create(
                    &mut db,
                    "some/path".into(),
                    0,
                    OffsetDateTime::now_utc(),
                    OffsetDateTime::now_utc(),
                )
                .unwrap();
                let documents = list(&mut db).unwrap();
                assert!(!documents.is_empty());
            }
        }
    }

    pub mod document_metadata {
        //! this module contains some simple CRUD functions for interacting with
        //! the document_metadata sqlite table
        //! When possible, instead use the helpers in the query::utils module to
        //! implement your feature
    }

    pub mod utils {}
}

#[cfg(test)]
mod test {
    use crate::preamble::*;

    #[test]
    pub fn init() -> Result<()> {
        DB::open(":memory:")?;
        Ok(())
    }
}
