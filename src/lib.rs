use std::collections::HashSet;
use std::path::{Path, PathBuf};
use thiserror::Error;
use time::OffsetDateTime;

pub mod core;

pub const APP_NAME: &str = "zet";
pub const DB_NAME: &str = "db.sqlite";

pub mod preamble {
    pub use crate::error::*;
    pub use crate::result::*;
    pub use crate::{APP_NAME, DB_NAME};
}

pub mod error {
    use thiserror::Error;

    #[derive(Error, Debug)]
    pub enum Error {
        #[error("rusqlite error: {0}")]
        RusqliteError(#[from] rusqlite::Error),
        #[error("migration error: {0}")]
        MigrationError(#[from] rusqlite_migration::Error),
        #[error("io error: {0}")]
        IOError(#[from] std::io::Error),
        #[error("parse error: {0}")]
        ParseError(String),
        #[error("CollectionNotFoundError: could not find .zet folder")]
        CollectionNotFoundError,
        #[error("NoParentError: path had no parent folder")]
        NoParentError,
        #[error("InitError")]
        InitError,
    }
}

pub mod result {
    use crate::error::*;

    pub type Result<T, E = Error> = std::result::Result<T, E>;
}

pub mod config {
    use std::path::PathBuf;

    use crate::core::parser::FrontMatterFormat;

    pub struct Config {
        pub root: PathBuf,
        pub front_matter_format: FrontMatterFormat,
    }
}
