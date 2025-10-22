use rusqlite::Connection;
use rusqlite_migration::{M, Migrations};
use sql_minifier::macros::load_sql;
use std::{
    cell::LazyCell,
    ops::{Deref, DerefMut},
    path::Path,
};

use super::*;

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

pub mod query {
    use super::*;
    pub mod document {
        use super::*;
        use rusqlite::params;
        use sql_minifier::macros::minify_sql;
        use std::path::PathBuf;
        use time::OffsetDateTime;
        use uuid::Uuid;

        use crate::db::{conversion::PathBufContainer, tables::Document};

        pub fn create(
            conn: &mut Connection,
            id: Uuid,
            path: PathBuf,
            hash: u64,
            modified: OffsetDateTime,
            created: OffsetDateTime,
        ) -> Result<Uuid> {
            let mut query = conn.prepare(minify_sql!(
            r#"insert into document (id, path, hash, modified, created) values (?,?,?,?,?) returning id"#
        ))?;
            let res: Uuid = query.query_row(
                params![id, PathBufContainer(path), hash, modified, created],
                |r| r.get(0),
            )?;

            Ok(res)
        }
        pub fn list(conn: &mut Connection) -> Result<Vec<Document>> {
            let mut query = conn.prepare(minify_sql!(r#"select * from document"#))?;
            let result = query
                .query_map([], |r| {
                    Ok(Document {
                        id: r.get(0)?,
                        path: (r.get::<usize, PathBufContainer>(1)?).into(),
                        hash: r.get(2)?,
                        modified: r.get(3)?,
                        created: r.get(4)?,
                    })
                })?
                .map(|f| f.map_err(From::from))
                .collect::<Result<Vec<Document>>>()?;
            Ok(result)
        }
        pub fn get(conn: &mut Connection, id: Uuid) -> Result<Document> {
            let mut query = conn.prepare(minify_sql!(
                r#"select * from document where id = ? limit 1"#
            ))?;
            let result = query.query_row([id], |r| {
                Ok(Document {
                    id: r.get(0)?,
                    path: (r.get::<usize, PathBufContainer>(1)?).into(),
                    hash: r.get(2)?,
                    modified: r.get(3)?,
                    created: r.get(4)?,
                })
            })?;
            Ok(result)
        }
        pub fn update(
            conn: &mut Connection,
            id: Uuid,
            path: PathBuf,
            hash: u64,
            modified: OffsetDateTime,
            created: OffsetDateTime,
        ) -> Result<()> {
            let mut query = conn.prepare(minify_sql!(
                r#"update document
                    set
                        path = ?,
                        hash = ?,
                        modified = ?,
                        created = ?
                    where id = ?"#
            ))?;
            query.execute(params![PathBufContainer(path), hash, modified, created, id])?;
            Ok(())
        }

        pub fn delete(conn: &mut Connection, id: Uuid) -> Result<()> {
            let mut query = conn.prepare(minify_sql!(r#"delete from document where id = ?"#))?;
            query.execute([id])?;
            Ok(())
        }

        #[cfg(test)]
        pub mod tests {
            use super::*;

            #[test]
            fn create_list() {
                let mut db = DB::open(":memory:").unwrap();
                let id = create(
                    &mut db,
                    Uuid::new_v4(),
                    "some/path".into(),
                    0,
                    OffsetDateTime::now_utc(),
                    OffsetDateTime::now_utc(),
                )
                .unwrap();
                let _id2 = create(
                    &mut db,
                    Uuid::new_v4(),
                    "some/path2".into(),
                    1,
                    OffsetDateTime::now_utc(),
                    OffsetDateTime::now_utc(),
                )
                .unwrap();
                let documents = list(&mut db).unwrap();
                assert!(!documents.is_empty());
                assert!(documents.len() == 2);
            }
            #[test]
            fn create_get() {
                let mut db = DB::open(":memory:").unwrap();
                let id = create(
                    &mut db,
                    Uuid::new_v4(),
                    "some/path".into(),
                    0,
                    OffsetDateTime::now_utc(),
                    OffsetDateTime::now_utc(),
                )
                .unwrap();
                let document = get(&mut db, id).unwrap();
                assert_eq!(document.id, id);
            }
            #[test]
            fn create_get_update_get() {
                let mut db = DB::open(":memory:").unwrap();
                let id = create(
                    &mut db,
                    Uuid::new_v4(),
                    "some/path".into(),
                    0,
                    OffsetDateTime::now_utc(),
                    OffsetDateTime::now_utc(),
                )
                .unwrap();
                let document_before = get(&mut db, id).unwrap();
                update(
                    &mut db,
                    id,
                    "some/new/path".into(),
                    1,
                    OffsetDateTime::now_utc(),
                    OffsetDateTime::now_utc(),
                )
                .unwrap();
                let document_after = get(&mut db, id).unwrap();
                assert_ne!(document_before.path, document_after.path);
                assert_ne!(document_before.hash, document_after.hash);
                assert_eq!(document_before.id, document_after.id);
            }
            #[test]
            fn create_get_update_get_delete_list() {
                let mut db = DB::open(":memory:").unwrap();
                let id_first = create(
                    &mut db,
                    Uuid::new_v4(),
                    "some/path".into(),
                    0,
                    OffsetDateTime::now_utc(),
                    OffsetDateTime::now_utc(),
                )
                .unwrap();
                let id = create(
                    &mut db,
                    Uuid::new_v4(),
                    "some/other/path".into(),
                    1,
                    OffsetDateTime::now_utc(),
                    OffsetDateTime::now_utc(),
                )
                .unwrap();
                let document_before = get(&mut db, id).unwrap();
                update(
                    &mut db,
                    id,
                    "some/new/path".into(),
                    2,
                    OffsetDateTime::now_utc(),
                    OffsetDateTime::now_utc(),
                )
                .unwrap();
                let document_after = get(&mut db, id).unwrap();
                assert_ne!(document_before.path, document_after.path);
                assert_ne!(document_before.hash, document_after.hash);
                assert_eq!(document_before.id, document_after.id);
                delete(&mut db, id).unwrap();
                let mut documents = list(&mut db).unwrap();
                assert!(!documents.is_empty());
                let document = documents.pop().unwrap();
                assert_eq!(document.id, id_first);
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
