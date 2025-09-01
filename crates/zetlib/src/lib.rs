use parser::FrontMatterFormat;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use thiserror::Error;
use time::OffsetDateTime;

pub mod collection;
pub mod db;
pub mod parser;

pub const APP_NAME: &str = "zet";

pub struct ZetConfig {
    pub root: PathBuf,
    pub front_matter_format: FrontMatterFormat,
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("rusqlite error: {0}")]
    RusqliteError(rusqlite::Error),
    #[error("migration error: {0}")]
    MigrationError(rusqlite_migration::Error),
    #[error("io error: {0}")]
    IOError(std::io::Error),
    #[error("CollectionNotFoundError: could not find .marks folder")]
    CollectionNotFoundError,
    #[error("NoParentError: path had no parent folder")]
    NoParentError,
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

/// the .zet directory
pub fn app_work_dir(dir: &Path) -> PathBuf {
    dir.to_owned().join(format!(".{APP_NAME}"))
}

/// from CWD walks up the directory tree until a directory containing .markz
/// is found or $HOME is reached, whichever comes first
pub fn resolve_root() -> Result<PathBuf> {
    let mut dir = std::path::absolute(std::env::current_dir()?)?;
    log::debug!("resolving markz root directory, starting from {:?}", dir);
    // check if dir contains .markz or of $HOME has been reached
    while !app_work_dir(&dir).is_dir() {
        dir = match dir.parent() {
            Some(p) => p.to_owned(),
            None => {
                log::error!("no .markz directory found");
                return Err(Error::NoParentError);
            }
        }
    }

    if !app_work_dir(&dir).is_dir() {
        log::error!("no .markz directory found");
        return Err(Error::CollectionNotFoundError);
    }

    Ok(dir)
}
