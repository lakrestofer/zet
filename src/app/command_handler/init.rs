use color_eyre::eyre::eyre;
use normalize_path::NormalizePath;
use resolve_path::PathResolveExt;
use std::path::PathBuf;
use zet::core::paths::{app_work_dir, db_dir};
use zet::preamble::*;

pub fn handle_command(root: Option<PathBuf>, force: bool) -> Result<()> {
    let root = root.unwrap_or(std::env::current_dir()?);
    let root: PathBuf = root.try_resolve()?.into_owned().normalize();

    let work_dir = app_work_dir(&root); // .zet
    let db_dir = db_dir(&root); // .zet/db.sqlite

    // handle if the path already exists
    if work_dir.exists() {
        if !force {
            log::error!("{:?} already exists! specify --force to reinit", work_dir);
            return Err(eyre!("could not initialize {:?}", work_dir));
        }
        log::warn!("removing directory {:?} (and contents)", work_dir);
        std::fs::remove_dir_all(&work_dir)?;
    }
    log::info!("creating directory {:?} (and contents)", work_dir);
    std::fs::create_dir_all(&work_dir)?;

    if db_dir.is_file() {
        std::fs::remove_file(&db_dir)?;
    }

    // create and execute migrations on directory
    let _ = zet::core::db::DB::open(db_dir)?;

    // TODO, write default configuration file

    Ok(())
}
