use crate::core::db::DB;
use crate::core::db::DbCrud;
use crate::core::paths::workspace_paths;
use crate::core::types::CreatedTimestamp;
use crate::core::types::Document;
use crate::core::types::DocumentId;
use crate::core::types::DocumentPath;
use crate::core::types::ModifiedTimestamp;
// use ignore::{DirEntry, WalkBuilder};
use std::collections::HashSet;
use std::path::Path;
use std::path::PathBuf;
use time::OffsetDateTime;
use twox_hash::XxHash3_64;

use crate::preamble::*;

/// given a directory of markdown files, determine the following:
/// - are there any new documents?
/// - are there any documents that we need to reparse?
/// - are there any documents that have been removed?
pub fn collection_status(
    root: &Path,
    db: &DB,
) -> (
    Vec<DocumentPath>,
    Vec<(
        DocumentId,
        DocumentPath,
        ModifiedTimestamp,
        CreatedTimestamp,
        u64,
    )>,
    Vec<DocumentId>,
) {
    // collect paths of document from root
    let disk_paths: Vec<PathBuf> = workspace_paths(root).unwrap();

    let db_documents: Vec<Document> = Document::list(&db).unwrap();

    // we start by figuring out documents that have been removed, which are new
    // and which that we need to investigate further.
    let mut path_set = HashSet::new();
    let mut db_path_set = HashSet::new();
    path_set.extend(&disk_paths);
    db_path_set.extend(db_documents.iter().map(|d| &d.path.0));

    let mut new: Vec<DocumentPath> = disk_paths
        .into_iter()
        .filter(|p| !db_path_set.contains(p))
        .map(|p| DocumentPath(p))
        .collect();
    let mut exists = Vec::new();
    let mut removed = Vec::new();

    for (i, d) in db_documents.iter().enumerate() {
        if path_set.contains(&d.path.0) {
            exists.push(i);
        } else {
            removed.push(d.id.to_owned());
        }
    }

    // out of the ones we need to check further we first compare the modified timestamps
    // then their hash
    let to_update: Vec<(
        DocumentId,
        DocumentPath,
        ModifiedTimestamp,
        CreatedTimestamp,
        u64,
    )> = exists
        .into_iter()
        .flat_map(
            |i| -> crate::result::Result<(
                usize,
                DocumentPath,
                ModifiedTimestamp,
                &ModifiedTimestamp,
                CreatedTimestamp,
                &CreatedTimestamp,
            )> {
                let path = db_documents[i].path.to_owned();
                let metadata = std::fs::metadata(path.0)?;

                let current_modified = ModifiedTimestamp(metadata.modified().map(From::from)?);
                let previous_modified: &ModifiedTimestamp = &db_documents[i].modified;

                let current_created = CreatedTimestamp(metadata.created().map(From::from)?);
                let previous_created: &CreatedTimestamp = &db_documents[i].created;
                Ok((
                    i,
                    path,
                    current_modified,
                    previous_modified,
                    current_created,
                    previous_created,
                ))
            },
        )
        .filter(
            |(_, _, current_modified, previous_modified, current_created, previous_created)| {
                *current_modified != **previous_modified || *current_created != **previous_created
            },
        )
        .map(|(index, path, current_modified, _, current_created, _)| {
            (index, path, current_modified, current_created)
        })
        .flat_map(
            |(index, path, modified, created)| -> crate::result::Result<(
                usize,
                DocumentPath,
                ModifiedTimestamp,
                CreatedTimestamp,
                u64,
                &u64,
            )> {
                let content = std::fs::read_to_string(&path.0)?;
                let current = XxHash3_64::oneshot(content.as_bytes());
                let previous = &db_documents[index].hash;
                Ok((index, path, modified, created, current, previous))
            },
        )
        .filter(|(index, path, modified, created, current, previous)| *current != **previous)
        .map(|(index, path, modified, created, current, previous)| {
            let id = db_documents[index].id.clone();
            (id, path, modified, created, current)
        })
        .collect();

    return (new, to_update, removed);
}
