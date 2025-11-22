use rusqlite::Connection;
use rusqlite_migration::{M, Migrations};
use sql_minifier::macros::load_sql;
use std::{
    cell::LazyCell,
    ops::{Deref, DerefMut},
    path::Path,
};

use crate::preamble::*;

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

pub trait DbCrud<T, I> {
    fn list(db: &Connection) -> Result<Vec<T>>;
    fn get(db: &mut Connection, id: I) -> Result<T>;

    fn upsert(db: &mut Connection, values: Vec<T>) -> Result<Vec<I>>;

    fn delete(db: &mut Connection, ids: Vec<I>) -> Result<()>;
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
