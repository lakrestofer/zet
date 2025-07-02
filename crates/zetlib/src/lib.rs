use std::collections::HashSet;
use std::path::PathBuf;
use thiserror::Error;
use time::OffsetDateTime;
use uuid::Uuid;

pub mod db;
pub mod workspace_commands;

pub struct ZetConfig {
    pub root: PathBuf,
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("rusqlite error: {0}")]
    RusqliteError(rusqlite::Error),
    #[error("migration error: {0}")]
    MigrationError(rusqlite_migration::Error),
    #[error("io error: {0}")]
    IOError(std::io::Error),
}

pub type Result<T, E = Error> = std::result::Result<T, E>;

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Self::IOError(value)
    }
}
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
