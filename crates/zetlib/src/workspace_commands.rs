use crate::*;

pub mod index {
    use super::*;
    use crate::db::DB;
    use crate::db::PathBufContainer;
    use ignore::{DirEntry, WalkBuilder};
    use sql_minifier::macros::minify_sql;
    use std::path::PathBuf;
    use twox_hash::XxHash3_64;
    use types::DocHashPartition;
    pub use types::DocTimestampPartition;
    use types::{CollectionDocumentStatus, DocPathPartition};

    /// given a directory of markdown files, determine the following:
    /// - are there any new documents?
    /// - are there any documents that we need to reparse?
    /// - are there any documents that have been removed?
    pub fn collection_status(conf: &ZetConfig, db: &mut DB) -> Result<CollectionDocumentStatus> {
        // collect paths of document from root
        let disk_paths: Vec<PathBuf> = workspace_paths(&conf)?;
        let db_paths: Vec<(Uuid, PathBuf)> = db
            .prepare(minify_sql!(r#"select id,path from document"#))?
            .query_map([], |r| {
                Ok((r.get(0)?, r.get::<usize, PathBufContainer>(1)?.into()))
            })?
            .map(|f| f.map_err(From::from))
            .collect::<Result<Vec<(Uuid, PathBuf)>>>()?;

        // which paths have we previously seen?
        let DocPathPartition {
            new,
            intersect,
            removed,
        } = document_path_partition(disk_paths, db_paths)?;

        // which paths have been modified according to the filesystem?
        let intersect = intersect.into_iter();
        let paths_with_timestamps: Vec<(Uuid, PathBuf, (OffsetDateTime, OffsetDateTime))> = {
            let mut query =
                db.prepare(minify_sql!(r#"select modified from document where id = ?"#))?;
            intersect
                .clone()
                .map(|(id, path)| {
                    let metadata = std::fs::metadata(&path)?;
                    let current_modified = metadata.modified()?.into(); // the current timestamp
                    let previous_modified = query.query_row([id], |r| r.get(0))?; // the previously stored timestamp
                    Ok((id, path, (current_modified, previous_modified)))
                })
                .collect::<Result<Vec<(Uuid, PathBuf, (OffsetDateTime, OffsetDateTime))>>>()?
        };

        let DocTimestampPartition {
            modified,
            unchanged: _,
        } = document_timestamp_partition(paths_with_timestamps)?;

        // which paths have been modified such that their content is different?
        let paths_with_hash: Vec<(Uuid, PathBuf, OffsetDateTime, (u64, u64))> = {
            let mut query = db.prepare(minify_sql!(r#"select hash from document where id = ?"#))?;
            modified
                .into_iter()
                .map(|(id, path, modified)| {
                    let content = std::fs::read_to_string(&path)?;
                    let current_hash = XxHash3_64::oneshot(content.as_bytes());
                    let previous_hash: u64 = query.query_row([id], |r| r.get(0))?; // the previously stored timestamp
                    Ok((id, path, modified, (current_hash, previous_hash)))
                })
                .collect::<Result<Vec<(Uuid, PathBuf, OffsetDateTime, (u64, u64))>>>()?
        };

        let DocHashPartition {
            modified,
            unchanged: _,
        } = document_hash_partition(paths_with_hash)?;

        Ok(CollectionDocumentStatus::new(new, modified, removed))
    }

    fn is_filetype(entry: &DirEntry, ext: &str) -> bool {
        entry
            .file_name()
            .to_str()
            .map_or(false, |s| s.ends_with(ext))
    }

    fn is_markdown_file(e: &DirEntry) -> bool {
        is_filetype(e, "md")
    }

    fn workspace_paths(conf: &ZetConfig) -> Result<Vec<PathBuf>> {
        let files: Vec<PathBuf> = WalkBuilder::new(&conf.root)
            .build()
            .filter_map(|e| e.ok())
            .filter(is_markdown_file)
            .map(|e| e.path().to_owned())
            .collect();
        Ok(files)
    }

    /// Compares a given set of paths with the documents stored in the db
    /// Returns three sets of paths:
    /// - The paths in the input set but not in the db set - new paths
    /// - The paths in the input set and the db set + their ids - paths that we will inspect further
    /// - The paths in the db but not in the input set -
    fn document_path_partition(
        paths: Vec<PathBuf>,
        db_paths: Vec<(Uuid, PathBuf)>,
    ) -> Result<types::DocPathPartition> {
        let mut path_set = HashSet::new();
        path_set.extend(paths.clone().into_iter());
        let mut db_path_set = HashSet::new();
        db_path_set.extend(db_paths.clone().into_iter().map(|(_id, path)| path));

        let mut info = types::DocPathPartition::default();

        // paths in input set but not in db set
        for path in paths {
            if !db_path_set.contains(&path) {
                info.new.push(path);
            }
        }

        // paths in input set and db set + paths in db set but not in input set
        for (id, path) in db_paths {
            if path_set.contains(&path) {
                info.intersect.push((id, path));
            } else {
                info.removed.push((id, path));
            }
        }

        Ok(info)
    }

    /// given a set of paths and their current modified timestamps + and the set of paths in the db
    /// return those that have changed and those that have not.
    fn document_timestamp_partition(
        paths: Vec<(Uuid, PathBuf, (OffsetDateTime, OffsetDateTime))>,
    ) -> crate::Result<DocTimestampPartition> {
        let mut partition = DocTimestampPartition::default();

        for (id, path, (current_modified, previous_modified)) in paths {
            if current_modified != previous_modified {
                partition.modified.push((id, path, current_modified));
            } else {
                partition.unchanged.push((id, path, current_modified));
            }
        }

        Ok(partition)
    }
    pub fn document_hash_partition(
        paths: Vec<(Uuid, PathBuf, OffsetDateTime, (u64, u64))>,
    ) -> crate::Result<DocHashPartition> {
        let mut partition = DocHashPartition::default();

        for (id, path, modified, (current_hash, prevous_hash)) in paths {
            if current_hash != prevous_hash {
                partition.modified.push((id, path, modified, current_hash));
            } else {
                partition.unchanged.push((id, path, modified, current_hash));
            }
        }

        Ok(partition)
    }

    pub mod types {
        use std::path::PathBuf;
        use time::OffsetDateTime;
        use uuid::Uuid;

        #[derive(Default)]
        pub struct CollectionDocumentStatus {
            pub new: Vec<PathBuf>,
            pub updated: Vec<(Uuid, PathBuf, OffsetDateTime, u64)>,
            pub removed: Vec<(Uuid, PathBuf)>,
        }

        impl CollectionDocumentStatus {
            pub fn new(
                new: Vec<PathBuf>,
                updated: Vec<(Uuid, PathBuf, OffsetDateTime, u64)>,
                removed: Vec<(Uuid, PathBuf)>,
            ) -> Self {
                Self {
                    new,
                    updated,
                    removed,
                }
            }
        }

        #[derive(Default)]
        pub struct DocPathPartition {
            pub new: Vec<PathBuf>,
            pub intersect: Vec<(Uuid, PathBuf)>,
            pub removed: Vec<(Uuid, PathBuf)>,
        }
        #[derive(Default)]
        pub struct DocTimestampPartition {
            pub modified: Vec<(Uuid, PathBuf, OffsetDateTime)>,
            pub unchanged: Vec<(Uuid, PathBuf, OffsetDateTime)>,
        }
        #[derive(Default)]
        pub struct DocHashPartition {
            pub modified: Vec<(Uuid, PathBuf, OffsetDateTime, u64)>,
            pub unchanged: Vec<(Uuid, PathBuf, OffsetDateTime, u64)>,
        }
    }
}
