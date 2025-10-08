use parser::FrontMatterFormat;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use thiserror::Error;
use time::OffsetDateTime;

pub mod cli;
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
    #[error("parse error: {0}")]
    ParseError(String),
    #[error("CollectionNotFoundError: could not find .zet folder")]
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

/// from CWD, walk up the directory tree until a directory containing .zet
/// is found or / is reached
pub fn resolve_root(dir: Option<PathBuf>) -> Result<PathBuf> {
    if let Some(dir) = dir {
        if !app_work_dir(&dir).is_dir() {
            log::error!("provided root dir does not contain a .zet directory!");
            return Err(Error::CollectionNotFoundError);
        } else {
            return Ok(dir);
        }
    }

    let mut dir = std::path::absolute(std::env::current_dir()?)?;
    log::debug!("resolving zet root directory, starting from {:?}", dir);
    // check if dir contains .zet or if / have been reached
    while !app_work_dir(&dir).is_dir() {
        dir = match dir.parent() {
            Some(p) => p.to_owned(),
            None => {
                log::error!("{:?} had no parent!", dir);
                return Err(Error::NoParentError);
            }
        }
    }

    if !app_work_dir(&dir).is_dir() {
        log::error!("no .zet directory found");
        return Err(Error::CollectionNotFoundError);
    }
    log::debug!("zet root directory resolved to {:?}", dir);

    Ok(dir)
}
