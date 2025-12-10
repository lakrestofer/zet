pub mod db;
pub mod parser;
pub mod slug;
pub mod types;


use crate::core::parser::ast_nodes::{self};

use crate::core::db::{DB, DbList};
use crate::core::types::DocumentId;
use crate::preamble::*;
use std::path::Path;
use std::path::PathBuf;

use crate::core::types::CreatedTimestamp;
use crate::core::types::Document;
use crate::core::types::DocumentPath;
use crate::core::types::ModifiedTimestamp;
// use ignore::{DirEntry, WalkBuilder};
use std::collections::HashSet;

use twox_hash::XxHash32;

use color_eyre::eyre::eyre;
use ignore::{DirEntry, WalkBuilder};

////////////////////////////////////////////////////////////
// Paths
////////////////////////////////////////////////////////////

/// the directory where all documents are stored
pub fn app_work_dir(root: &Path) -> PathBuf {
    root.to_owned().join(format!(".{APP_NAME}"))
}

/// .zet/db.sqlite
pub fn db_dir(root: &Path) -> PathBuf {
    root.to_owned().join(format!(".{APP_NAME}")).join(DB_NAME)
}

/// from CWD, walk up the directory tree until a directory containing .zet
/// is found or / is reached
pub fn resolve_root(dir: Option<PathBuf>) -> Result<PathBuf> {
    if let Some(dir) = dir {
        if !app_work_dir(&dir).is_dir() {
            log::error!("provided root dir does not contain a .zet directory!");
            return Err(eyre!("collection not found!"));
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
                return Err(eyre!("no parent found"));
            }
        }
    }

    if !app_work_dir(&dir).is_dir() {
        log::error!("no .zet directory found");
        return Err(eyre!("collection not found!"));
    }
    log::debug!("zet root directory resolved to {:?}", dir);

    Ok(dir)
}

pub fn is_filetype(entry: &DirEntry, ext: &str) -> bool {
    entry.file_name().to_str().is_some_and(|s| s.ends_with(ext))
}

pub fn is_markdown_file(e: &DirEntry) -> bool {
    is_filetype(e, "md")
}

pub fn workspace_paths(root: &Path) -> Result<Vec<PathBuf>> {
    let files: Vec<PathBuf> = WalkBuilder::new(root)
        .build()
        .filter_map(|e| e.ok())
        .filter(is_markdown_file)
        .map(|e| e.path().to_owned())
        .collect();
    Ok(files)
}

////////////////////////////////////////////////////////////
// change detection
////////////////////////////////////////////////////////////

const HASH_SEED: u32 = 42;

pub fn hash(content: &str) -> u32 {
    XxHash32::oneshot(HASH_SEED, content.as_bytes())
}

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
        u32,
    )>,
    Vec<DocumentId>,
) {
    // collect paths of document from root
    let disk_paths: Vec<PathBuf> = workspace_paths(root).unwrap();

    let db_documents: Vec<Document> = Document::list(db).unwrap();

    // we start by figuring out documents that have been removed, which are new
    // and which that we need to investigate further.
    let mut path_set = HashSet::new();
    let mut db_path_set = HashSet::new();
    path_set.extend(disk_paths.clone());
    db_path_set.extend(db_documents.iter().map(|d| &d.path.0));

    let new: Vec<DocumentPath> = disk_paths
        .into_iter()
        .filter(|p| !db_path_set.contains(p))
        .map(DocumentPath)
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
        u32,
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
                let metadata = std::fs::metadata(&path.0)?;

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
                u32,
                &u32,
            )> {
                let content = std::fs::read_to_string(&path.0)?;
                let current = crate::core::hash(&content);
                let previous = &db_documents[index].hash;
                Ok((index, path, modified, created, current, previous))
            },
        )
        .filter(|(_, _, _, _, current, previous)| *current != **previous)
        .map(|(index, path, modified, created, current, _)| {
            let id = db_documents[index].id.clone();
            (id, path, modified, created, current)
        })
        .collect();

    (new, to_update, removed)
}

////////////////////////////////////////////////////////////
// Path manipulation functions
////////////////////////////////////////////////////////////

/// Given a path to a document within the collection, we compute its id.
pub fn path_to_id(root: &Path, path: &Path) -> DocumentId {
    let mut path = path.to_owned();
    path.set_extension("");
    let path = path.strip_prefix(root).unwrap();
    let id = crate::core::slug::slugify(path.to_str().unwrap());
    DocumentId(id)
}

/// given a string, we check if there exists any document in the database
/// whose id ends in that string.
// pub fn resolve_id(db: &DB, suffix: &str) -> Result<Vec<DocumentId>> {
//     let query = db.prepare(sql!(
//         r#"
//             select
//                 id
//             from
//                 document
//             where

//         "#
//     ))?;
//     todo!()
// }

////////////////////////////////////////////////////////////
// Parsing
////////////////////////////////////////////////////////////

pub const TITLE_KEY: &str = "title";

pub fn extract_title_from_frontmatter(data: &serde_json::Value) -> Option<String> {
    let res = data.get("title")?;

    if let serde_json::Value::String(s) = res {
        return Some(s.to_owned());
    }

    None
}

/// TODO write documentation for how we retrieve the title
pub fn extract_title_from_ast(ast: &[ast_nodes::Node]) -> Option<String> {
    // find the first heading
    let heading = ast.iter().find(|n| {
        if let ast_nodes::Node::Heading(heading) = n {
            heading.level == 1
        } else {
            false
        }
    })?;
    let mut title = String::new();

    // take all the text content
    if let ast_nodes::Node::Heading(heading) = heading {
        let children = &heading.children;

        for child in children {
            if let ast_nodes::Node::Text(text) = child {
                title.push_str(text.text.as_str());
            }
        }
    } else {
        unreachable!()
    }

    Some(title)
}
