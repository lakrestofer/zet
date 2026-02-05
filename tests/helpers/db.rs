use rusqlite::Connection;
use std::path::Path;
use zet::core::db::DB;
use zet::core::types::document::{Document, DocumentId};
use zet::core::types::link::DocumentLink;

/// Opens the test database in the workspace
pub fn open_test_db(workspace_root: &Path) -> DB {
    let db_path = super::db_path(workspace_root);
    DB::open(db_path).expect("Failed to open test database")
}

/// Counts the number of documents in the database
pub fn count_documents(db: &DB) -> usize {
    db.prepare("SELECT COUNT(*) FROM document")
        .expect("Failed to prepare count query")
        .query_row([], |row| row.get::<_, i64>(0))
        .expect("Failed to count documents") as usize
}

/// Counts the number of tags in the database
pub fn count_tags(db: &DB) -> usize {
    db.prepare("SELECT COUNT(*) FROM tag")
        .expect("Failed to prepare count query")
        .query_row([], |row| row.get::<_, i64>(0))
        .expect("Failed to count tags") as usize
}

/// Counts the number of links in the database
pub fn count_links(db: &DB) -> usize {
    db.prepare("SELECT COUNT(*) FROM document_link")
        .expect("Failed to prepare count query")
        .query_row([], |row| row.get::<_, i64>(0))
        .expect("Failed to count links") as usize
}

/// Counts the number of headings in the database
pub fn count_headings(db: &DB) -> usize {
    db.prepare("SELECT COUNT(*) FROM document_heading")
        .expect("Failed to prepare count query")
        .query_row([], |row| row.get::<_, i64>(0))
        .expect("Failed to count headings") as usize
}

/// Counts the number of tasks in the database
pub fn count_tasks(db: &DB) -> usize {
    db.prepare("SELECT COUNT(*) FROM document_task")
        .expect("Failed to prepare count query")
        .query_row([], |row| row.get::<_, i64>(0))
        .expect("Failed to count tasks") as usize
}

/// Gets tags for a specific document
pub fn get_tags_for_document(db: &DB, doc_id: &str) -> Vec<String> {
    let mut stmt = db
        .prepare(
            "SELECT t.tag FROM tag t
             JOIN document_tag_map dtm ON t.id = dtm.tag_id
             WHERE dtm.document_id = ?",
        )
        .expect("Failed to prepare tags query");

    stmt.query_map([doc_id], |row| row.get::<_, String>(0))
        .expect("Failed to query tags")
        .map(|r| r.expect("Failed to extract tag"))
        .collect()
}

/// Gets links from a specific document
pub fn get_links_from(db: &DB, doc_id: &str) -> Vec<(String, Option<String>)> {
    let mut stmt = db
        .prepare("SELECT from_id, to_id FROM document_link WHERE from_id = ?")
        .expect("Failed to prepare links query");

    stmt.query_map([doc_id], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, Option<String>>(1)?,
        ))
    })
    .expect("Failed to query links")
    .map(|r| r.expect("Failed to extract link"))
    .collect()
}

/// Counts links with NULL to_id (broken links)
pub fn count_broken_links(db: &DB) -> usize {
    db.prepare("SELECT COUNT(*) FROM document_link WHERE to_id IS NULL")
        .expect("Failed to prepare count query")
        .query_row([], |row| row.get::<_, i64>(0))
        .expect("Failed to count broken links") as usize
}

/// Counts checked tasks
pub fn count_checked_tasks(db: &DB) -> usize {
    db.prepare("SELECT COUNT(*) FROM document_task WHERE checked = 1")
        .expect("Failed to prepare count query")
        .query_row([], |row| row.get::<_, i64>(0))
        .expect("Failed to count checked tasks") as usize
}

/// Counts unchecked tasks
pub fn count_unchecked_tasks(db: &DB) -> usize {
    db.prepare("SELECT COUNT(*) FROM document_task WHERE checked = 0")
        .expect("Failed to prepare count query")
        .query_row([], |row| row.get::<_, i64>(0))
        .expect("Failed to count unchecked tasks") as usize
}
