pub mod collection;
pub mod db;
pub mod parser;

pub mod paths {
    use std::path::{Path, PathBuf};

    use crate::preamble::*;
    /// the directory where all documents are stored
    pub fn app_work_dir(root: &Path) -> PathBuf {
        root.to_owned().join(format!(".{APP_NAME}"))
    }

    /// .zet/db.sqlite
    pub fn db_dir(root: &Path) -> PathBuf {
        root.to_owned()
            .join(format!(".{APP_NAME}"))
            .join(DB_NAME.to_string())
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
}
