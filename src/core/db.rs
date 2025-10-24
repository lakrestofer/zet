use rusqlite::Connection;
use rusqlite_migration::{M, Migrations};
use sql_minifier::macros::load_sql;
use std::{
    cell::LazyCell,
    ops::{Deref, DerefMut},
    path::Path,
};

use crate::preamble::*;

pub use conversion::*;

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

pub mod conversion {
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    pub fn init() -> Result<()> {
        DB::open(":memory:")?;
        Ok(())
    }
}
