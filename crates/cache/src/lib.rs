use std::path::{Path, PathBuf};

use db::preamble::*;
use uuid::Uuid;

pub struct DocumentInDbStatus {
    new: Vec<PathBuf>,
    in_db: Vec<(PathBuf, Uuid)>,
    removed: Vec<(PathBuf, Uuid)>,
}

/// Compares a given set of paths with the documents stored in the db
/// Returns three sets of paths:
/// - The paths in the input set but not in the db set - new paths
/// - The paths in the input set and the db set + their ids - paths that we will inspect further
/// - The paths in the db but not in the input set -
pub fn document_db_status(
    db: &mut DB,
    paths: Vec<PathBuf>,
) -> db::preamble::Result<DocumentInDbStatus> {
    todo!()
}

pub(crate) mod queries {
    use super::*;
    pub(crate) mod document {
        use std::path::PathBuf;

        use sql_minifier::macros::minify_sql;

        use super::*;
        fn list_paths(db: &mut DB) -> db::preamble::Result<Vec<PathBuf>> {
            let mut query = db.prepare(minify_sql!(r#"select path from document"#))?;
            let result = query
                .query_map([], |r| Ok(r.get::<usize, PathBufContainer>(0)?.into()))?
                .map(|f| f.map_err(From::from))
                .collect::<Result<Vec<PathBuf>>>()?;
            Ok(result)
        }
    }
}
